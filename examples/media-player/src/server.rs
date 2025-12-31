use crate::media_player::{
    Error, MediaPlayer, MediaPlayerCallHandler, MediaPlayerPlayArgs, Metadata, State,
};
use aldrin::core::ObjectUuid;
use aldrin::{Handle, Object, Promise};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use std::convert::Infallible;
use std::time::Duration;
use tokio::signal;
use tokio::time::{self, Instant, Interval, MissedTickBehavior};

pub(crate) async fn run(bus: &Handle) -> Result<()> {
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
                call = self.svc.next_call() => {
                    match call {
                        Some(call) => MediaPlayer::dispatch_call(call, &mut self).await?,
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
            self.svc.position_changed(Some(&position))?;
        }

        if *position >= metadata.duration {
            println!("Track `{}` finished.", metadata.title);

            self.svc.state_changed(State::Transitioning)?;

            self.svc.last_metadata_changed(metadata)?;
            self.last_metadata = self.metadata.take();

            self.svc.metadata_changed(&self.metadata)?;

            self.position = None;
            self.svc.position_changed(self.position)?;

            self.state = State::Stopped;
            self.svc.state_changed(&self.state)?;

            self.timer = None;
        }

        Ok(())
    }
}

#[async_trait]
impl MediaPlayerCallHandler for Server {
    type Error = anyhow::Error;

    async fn get_state(&mut self, promise: Promise<State, Infallible>) -> Result<()> {
        promise.ok(&self.state)?;
        Ok(())
    }

    async fn get_metadata(&mut self, promise: Promise<Option<Metadata>, Infallible>) -> Result<()> {
        promise.ok(&self.metadata)?;
        Ok(())
    }

    async fn get_position(&mut self, promise: Promise<Option<u32>, Infallible>) -> Result<()> {
        promise.ok(self.position)?;
        Ok(())
    }

    async fn get_last_metadata(
        &mut self,
        promise: Promise<Option<Metadata>, Infallible>,
    ) -> Result<()> {
        promise.ok(&self.last_metadata)?;
        Ok(())
    }

    async fn play(&mut self, args: MediaPlayerPlayArgs, promise: Promise<(), Error>) -> Result<()> {
        let duration = args.duration.unwrap_or(10);

        println!("Starting `{}` with a duration of {duration}s.", args.title);

        if args.title.is_empty() {
            println!("Rejecting play command with empty title.");
            promise.err(Error::InvalidTitle)?;
            return Ok(());
        }

        self.svc.state_changed(State::Transitioning)?;

        if let Some(metadata) = self.metadata.take() {
            self.svc.last_metadata_changed(&metadata)?;
            self.last_metadata = Some(metadata);
        }

        self.metadata = Some(Metadata {
            title: args.title,
            duration,
        });
        self.svc.metadata_changed(&self.metadata)?;

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

    async fn stop(&mut self, promise: Promise<(), Infallible>) -> Result<()> {
        if self.state == State::Stopped {
            println!("Playback is already stopped.");
            promise.done()?;
            return Ok(());
        }

        println!("Stopping playback.");

        self.svc.state_changed(State::Transitioning)?;

        if let Some(metadata) = self.metadata.take() {
            self.svc.metadata_changed(())?;
            self.svc.last_metadata_changed(&metadata)?;
            self.last_metadata = Some(metadata);
        }

        self.position = None;
        self.svc.position_changed(self.position)?;

        self.state = State::Stopped;
        self.svc.state_changed(&self.state)?;

        self.timer = None;

        promise.done()?;
        Ok(())
    }

    async fn pause(&mut self, promise: Promise<(), Error>) -> Result<()> {
        if self.state == State::Playing {
            println!("Pausing playback.");
            self.state = State::Paused;
            self.svc.state_changed(&self.state)?;
            self.timer = None;
            promise.done()?;
        } else {
            println!("Cannot pause playback.");
            promise.err(Error::NotPlaying)?;
        }

        Ok(())
    }

    async fn resume(&mut self, promise: Promise<(), Error>) -> Result<()> {
        if self.state == State::Paused {
            println!("Resuming playback.");
            self.state = State::Playing;
            self.svc.state_changed(&self.state)?;
            self.start_timer(true);
            promise.done()?;
        } else {
            println!("Cannot resume playback.");
            promise.err(Error::NotPaused)?;
        }

        Ok(())
    }

    async fn invalid_call(&mut self, error: aldrin::Error) -> Result<()> {
        println!("Received an invalid call: {error}.");
        Ok(())
    }
}
