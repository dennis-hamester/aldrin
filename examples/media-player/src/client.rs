use crate::media_player::{
    MediaPlayerEvent, MediaPlayerPlayArgs, MediaPlayerProxy, Metadata, State,
};
use crate::{Play, ServerArg};
use aldrin::core::ObjectUuid;
use aldrin::{Event, Handle, Property};
use anyhow::{anyhow, Result};
use tokio::signal;

pub async fn listen(args: ServerArg, bus: &Handle) -> Result<()> {
    Listen::new(bus, args.server).await?.run().await
}

struct Listen {
    media_player: MediaPlayerProxy,

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
        let state = Property::from_reply(&media_player.get_state().await?)??;
        let metadata = Property::from_reply(&media_player.get_metadata().await?)??;
        let position = Property::from_reply(&media_player.get_position().await?)??;
        let last_metadata = Property::from_reply(&media_player.get_last_metadata().await?)??;

        Ok(Self {
            media_player,
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

        self.print_state();

        loop {
            tokio::select! {
                ev = self.media_player.next_event() => {
                    match ev {
                        Some(Ok(ev)) => {
                            if self.handle_event(ev) {
                                self.print_state();
                            }
                        }

                        Some(Err(e)) => println!("Received an invalid event: {e}."),
                        None => break,
                    }
                }

                _ = signal::ctrl_c() => break,
            }
        }

        Ok(())
    }

    fn print_state(&self) {
        // We ignore all state changes while the media player is transitioning. This resembles the
        // common begin() / end() idiom to some extend. Even when transitioning, state changes are
        // tracked, but not acted upon.
        if *self.state.get() == State::Transitioning {
            return;
        }

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

    // This function returns `true` if the event caused some state to change its value. To achieve
    // that, we use the `check_*` family of methods on `Property`.
    fn handle_event(&mut self, ev: MediaPlayerEvent) -> bool {
        match ev {
            MediaPlayerEvent::StateChanged(ev) => self.state_changed(ev),
            MediaPlayerEvent::MetadataChanged(ev) => self.metadata_changed(ev),
            MediaPlayerEvent::PositionChanged(ev) => self.position_changed(ev),
            MediaPlayerEvent::LastMetadataChanged(ev) => self.last_metadata_changed(ev),
        }
    }

    fn state_changed(&mut self, ev: Event<State>) -> bool {
        self.state.check_event(ev).is_some()
    }

    fn metadata_changed(&mut self, ev: Event<Option<Metadata>>) -> bool {
        self.metadata.check_event(ev).is_some()
    }

    fn position_changed(&mut self, ev: Event<Option<u32>>) -> bool {
        self.position.check_event(ev).is_some()
    }

    fn last_metadata_changed(&mut self, ev: Event<Metadata>) -> bool {
        // The last metadata property is special in that it starts out as `None`, then goes to
        // `Some` and will never be reset back to `None`. In the schema, this is expressed by the
        // fact that the getter returns an `Option`, but the event does not.
        self.last_metadata.check_event_some(ev).is_some()
    }
}

pub async fn pause(args: ServerArg, bus: &Handle) -> Result<()> {
    get_media_player(bus, args.server)
        .await?
        .pause()
        .await?
        .deserialize()??;

    Ok(())
}

pub async fn play(args: Play, bus: &Handle) -> Result<()> {
    get_media_player(bus, args.server.server)
        .await?
        .play(MediaPlayerPlayArgs {
            title: args.title,
            duration: args.duration,
            paused: Some(args.paused),
        })
        .await?
        .deserialize()??;

    Ok(())
}

pub async fn resume(args: ServerArg, bus: &Handle) -> Result<()> {
    get_media_player(bus, args.server)
        .await?
        .resume()
        .await?
        .deserialize()??;

    Ok(())
}

pub async fn stop(args: ServerArg, bus: &Handle) -> Result<()> {
    get_media_player(bus, args.server)
        .await?
        .stop()
        .await?
        .deserialize()??;

    Ok(())
}

async fn get_media_player(bus: &Handle, uuid: Option<ObjectUuid>) -> Result<MediaPlayerProxy> {
    let (_, [id]) = bus
        .find_object(uuid, &[MediaPlayerProxy::UUID])
        .await?
        .ok_or_else(|| anyhow!("media player not found"))?;

    let media_player = MediaPlayerProxy::new(bus, id).await?;
    Ok(media_player)
}
