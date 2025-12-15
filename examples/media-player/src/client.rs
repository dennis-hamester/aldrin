use crate::media_player::{
    MediaPlayerEventHandler, MediaPlayerPlayArgs, MediaPlayerProxy, Metadata, State,
};
use crate::{Play, ServerArg};
use aldrin::core::ObjectUuid;
use aldrin::{Event, Handle, Property};
use anyhow::{anyhow, Error, Result};
use async_trait::async_trait;
use tokio::signal;

pub(crate) async fn listen(args: ServerArg, bus: &Handle) -> Result<()> {
    Listen::new(bus, args.server).await?.run().await
}

struct Listen {
    media_player: MediaPlayerProxy,
    print_state: bool,

    // The entire state of the media player is kept track of using `Property`.
    state: Property<State>,
    metadata: Property<Option<Metadata>>,
    position: Property<Option<u32>>,
    last_metadata: Property<Option<Metadata>>,
}

impl Listen {
    async fn new(bus: &Handle, uuid: Option<ObjectUuid>) -> Result<Self> {
        let media_player = get_media_player(bus, uuid).await?;
        media_player.subscribe_all().await?;

        // State is initialized by reading it from the media player. `Property` can be constructed
        // directly from `Reply`.
        let state = Property::from_reply(media_player.get_state().await?)?;
        let metadata = Property::from_reply(media_player.get_metadata().await?)?;
        let position = Property::from_reply(media_player.get_position().await?)?;
        let last_metadata = Property::from_reply(media_player.get_last_metadata().await?)?;

        Ok(Self {
            media_player,
            print_state: true,
            state,
            metadata,
            position,
            last_metadata,
        })
    }

    async fn run(mut self) -> Result<()> {
        println!(
            "Listening for events on media player {}.",
            self.media_player.id().object_id.uuid
        );

        loop {
            self.print_state();

            tokio::select! {
                ev = self.media_player.next_event() => {
                    match ev {
                        Some(ev) => MediaPlayerProxy::dispatch_event(ev, &mut self).await?,
                        None => break,
                    }
                }

                _ = signal::ctrl_c() => break,
            }
        }

        Ok(())
    }

    fn print_state(&mut self) {
        // We ignore all state changes while the media player is transitioning. This resembles the
        // common begin() / end() idiom to some extend. Even when transitioning, state changes are
        // tracked, but not acted upon.
        if !self.print_state || (*self.state.get() == State::Transitioning) {
            return;
        }

        self.print_state = false;

        print!("State: ");
        match self.state.get() {
            State::Stopped => print!("stopped;"),
            State::Playing => print!("playing;"),
            State::Paused => print!("paused; "),
            State::Transitioning => unreachable!(),
        }

        if let Some(metadata) = self.metadata.get() {
            print!(
                " Title: `{}`; Duration: {}s;",
                metadata.title, metadata.duration
            );
        } else {
            print!(" no metadata;");
        }

        if let Some(position) = self.position.get() {
            print!(" Position: {position}s;");
        } else {
            print!(" no position;");
        }

        if let Some(metadata) = self.last_metadata.get() {
            print!(
                " Last Title: `{}`; Last Duration: {}s",
                metadata.title, metadata.duration
            );
        } else {
            print!(" no last metadata");
        }

        println!();
    }
}

#[async_trait]
impl MediaPlayerEventHandler for Listen {
    type Error = Error;

    async fn state_changed(&mut self, ev: Event<State>) -> Result<()> {
        self.print_state |= self.state.check_event(ev).is_some();
        Ok(())
    }

    async fn metadata_changed(&mut self, ev: Event<Option<Metadata>>) -> Result<()> {
        self.print_state |= self.metadata.check_event(ev).is_some();
        Ok(())
    }

    async fn position_changed(&mut self, ev: Event<Option<u32>>) -> Result<()> {
        self.print_state |= self.position.check_event(ev).is_some();
        Ok(())
    }

    async fn last_metadata_changed(&mut self, ev: Event<Metadata>) -> Result<()> {
        // The last metadata property is special in that it starts out as `None`, then goes to
        // `Some` and will never be reset back to `None`. In the schema, this is expressed by the
        // fact that the getter returns an `Option`, but the event does not.
        self.print_state |= self.last_metadata.check_event_some(ev).is_some();
        Ok(())
    }

    async fn invalid_event(&mut self, error: aldrin::Error) -> Result<()> {
        println!("Received an invalid event: {error}.");
        Ok(())
    }
}

pub(crate) async fn pause(args: ServerArg, bus: &Handle) -> Result<()> {
    get_media_player(bus, args.server)
        .await?
        .pause()
        .await?
        .into_args()?;

    Ok(())
}

pub(crate) async fn play(args: Play, bus: &Handle) -> Result<()> {
    get_media_player(bus, args.server.server)
        .await?
        .play(MediaPlayerPlayArgs {
            title: args.title,
            duration: args.duration,
            paused: Some(args.paused),
        })
        .await?
        .into_args()?;

    Ok(())
}

pub(crate) async fn resume(args: ServerArg, bus: &Handle) -> Result<()> {
    get_media_player(bus, args.server)
        .await?
        .resume()
        .await?
        .into_args()?;

    Ok(())
}

pub(crate) async fn stop(args: ServerArg, bus: &Handle) -> Result<()> {
    get_media_player(bus, args.server)
        .await?
        .stop()
        .await?
        .into_args()?;

    Ok(())
}

async fn get_media_player(bus: &Handle, uuid: Option<ObjectUuid>) -> Result<MediaPlayerProxy> {
    let (_, [id]) = bus
        .find_object_n(uuid, &[MediaPlayerProxy::UUID])
        .await?
        .ok_or_else(|| anyhow!("media player not found"))?;

    let media_player = MediaPlayerProxy::new(bus, id).await?;
    Ok(media_player)
}
