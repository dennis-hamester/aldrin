use crate::media_player::{
    Error, MediaPlayer, MediaPlayerFunction, MediaPlayerPlayArgs, Metadata, State,
};
use aldrin::core::ObjectUuid;
use aldrin::{Call, Handle, Object};
use anyhow::{anyhow, Result};
use std::convert::Infallible;
use std::time::Duration;
use tokio::signal;
use tokio::time::{self, Instant, Interval, MissedTickBehavior};

pub async fn run(bus: &Handle) -> Result<()> {
    Server::new(bus).await?.run().await
}

struct Server {
    obj: Object,
    svc: MediaPlayer,
    state: State,
    metadata: Option<Metadata>,
    position: Option<u32>,
    last_metadata: Option<Metadata>,
    timer: Option<Interval>,
}

impl Server {
    async fn new(bus: &Handle) -> Result<Self> {
        let obj = bus.create_object(ObjectUuid::new_v4()).await?;
        let svc = MediaPlayer::new(&obj).await?;

        Ok(Self {
            obj,
            svc,
            state: State::Stopped,
            metadata: None,
            position: None,
            last_metadata: None,
            timer: None,
        })
    }

    async fn run(mut self) -> Result<()> {
        println!("Starting media player {}.", self.obj.id().uuid);

        loop {
            tokio::select! {
                func = self.svc.next_call() => {
                    match func {
                        Some(Ok(func)) => self.handle_call(func)?,
                        Some(Err(e)) => println!("Received an invalid call: {e}."),
                        None => return Err(anyhow!("broker shut down")),
                    }
                }

                _ = Self::tick(self.timer.as_mut()), if self.timer.is_some() => {
                    self.timer_elapsed()?;
                }

                _ = signal::ctrl_c() => break,
            }
        }

        println!("Stopping media player {}.", self.obj.id().uuid);
        Ok(())
    }

    async fn tick(timer: Option<&mut Interval>) {
        timer.unwrap().tick().await;
    }

    fn handle_call(&mut self, func: MediaPlayerFunction) -> Result<()> {
        match func {
            MediaPlayerFunction::GetState(call) => self.get_state(call),
            MediaPlayerFunction::GetMetadata(call) => self.get_metadata(call),
            MediaPlayerFunction::GetPosition(call) => self.get_position(call),
            MediaPlayerFunction::GetLastMetadata(call) => self.get_last_metadata(call),
            MediaPlayerFunction::Play(call) => self.play(call),
            MediaPlayerFunction::Stop(call) => self.stop(call),
            MediaPlayerFunction::Pause(call) => self.pause(call),
            MediaPlayerFunction::Resume(call) => self.resume(call),
        }
    }

    fn get_state(&self, call: Call<(), State, Infallible>) -> Result<()> {
        call.ok(&self.state)?;
        Ok(())
    }

    fn get_metadata(&self, call: Call<(), Option<Metadata>, Infallible>) -> Result<()> {
        call.ok_ref(&self.metadata)?;
        Ok(())
    }

    fn get_position(&self, call: Call<(), Option<u32>, Infallible>) -> Result<()> {
        call.ok_ref(&self.position)?;
        Ok(())
    }

    fn get_last_metadata(&self, call: Call<(), Option<Metadata>, Infallible>) -> Result<()> {
        call.ok_ref(&self.last_metadata)?;
        Ok(())
    }

    fn play(&mut self, call: Call<MediaPlayerPlayArgs, (), Error>) -> Result<()> {
        let (args, promise) = call.into_args_and_promise();
        let duration = args.duration.unwrap_or(10);

        println!("Starting `{}` with a duration of {duration}s.", args.title);

        if args.title.is_empty() {
            println!("Rejecting play command with empty title.");
            promise.err(&Error::InvalidTitle)?;
            return Ok(());
        }

        self.svc.state_changed(&State::Transitioning)?;

        if let Some(metadata) = self.metadata.take() {
            self.svc.last_metadata_changed(&metadata)?;
            self.last_metadata = Some(metadata);
        }

        self.metadata = Some(Metadata {
            title: args.title,
            duration,
        });
        self.svc.metadata_changed_ref(&self.metadata)?;

        self.position = Some(0);
        self.svc.position_changed(self.position)?;

        if args.paused.unwrap_or(false) {
            self.state = State::Paused;
        } else {
            self.state = State::Playing;
            self.start_timer(duration > 0);
        }
        self.svc.state_changed(&self.state)?;

        promise.done()?;
        Ok(())
    }

    fn stop(&mut self, call: Call<(), (), Infallible>) -> Result<()> {
        if self.state == State::Stopped {
            println!("Playback is already stopped.");
            call.done()?;
            return Ok(());
        }

        println!("Stopping playback.");

        self.svc.state_changed(&State::Transitioning)?;

        if let Some(metadata) = self.metadata.take() {
            self.svc.metadata_changed(None)?;
            self.svc.last_metadata_changed(&metadata)?;
            self.last_metadata = Some(metadata);
        }

        self.position = None;
        self.svc.position_changed(self.position)?;

        self.position = None;
        self.svc.position_changed(self.position)?;

        self.state = State::Stopped;
        self.svc.state_changed(&self.state)?;

        self.timer = None;

        call.done()?;
        Ok(())
    }

    fn pause(&mut self, call: Call<(), (), Error>) -> Result<()> {
        if self.state == State::Playing {
            println!("Pausing playback.");
            self.state = State::Paused;
            self.svc.state_changed(&self.state)?;
            self.timer = None;
            call.done()?;
        } else {
            println!("Cannot pause playback.");
            call.err(&Error::NotPlaying)?;
        }

        Ok(())
    }

    fn resume(&mut self, call: Call<(), (), Error>) -> Result<()> {
        if self.state == State::Paused {
            println!("Resuming playback.");
            self.state = State::Playing;
            self.svc.state_changed(&self.state)?;
            self.start_timer(true);
            call.done()?;
        } else {
            println!("Cannot resume playback.");
            call.err(&Error::NotPaused)?;
        }

        Ok(())
    }

    fn start_timer(&mut self, delay: bool) {
        let mut timer = time::interval_at(
            Instant::now() + Duration::from_secs(delay as _),
            Duration::from_secs(1),
        );

        timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

        self.timer = Some(timer);
    }

    fn timer_elapsed(&mut self) -> Result<()> {
        let Some(ref mut position) = self.position else {
            return Ok(());
        };

        let Some(ref mut metadata) = self.metadata else {
            return Ok(());
        };

        *position += 1;

        if *position <= metadata.duration {
            self.svc.position_changed(Some(*position))?;
        }

        if *position >= metadata.duration {
            println!("Track `{}` finished.", metadata.title);

            self.svc.state_changed(&State::Transitioning)?;

            self.svc.last_metadata_changed(metadata)?;
            self.last_metadata = self.metadata.take();

            self.svc.metadata_changed_ref(&self.metadata)?;

            self.position = None;
            self.svc.position_changed(self.position)?;

            self.state = State::Stopped;
            self.svc.state_changed(&self.state)?;

            self.timer = None;
        }

        Ok(())
    }
}
