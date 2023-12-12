use aldrin::core::tokio::TokioTransport;
use aldrin::core::ObjectUuid;
use aldrin::{Client, Handle};
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use echo::{Echo, EchoEchoAllError, EchoEchoError, EchoEvent, EchoFunction, EchoProxy, ECHO_UUID};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::TcpStream;
use tokio::signal;

const BUS_DEFAULT: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9999);

// The generated code can be inspected using rustdoc. Just run
// `cargo doc --document-private-items --open` and look at the `echo` module.
aldrin::generate!("src/echo.aldrin");

/// Echo example.
#[derive(Parser)]
struct Args {
    /// Address of the broker to connect to.
    #[clap(short, long, default_value_t = BUS_DEFAULT)]
    bus: SocketAddr,

    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Parser)]
enum Command {
    /// Run an echo server.
    Server,

    /// List available echo servers.
    List,

    /// Send something to the server and have it echoed back.
    Echo(EchoArgs),

    /// Send something to the server and have it echoed back to all listeners.
    EchoAll(EchoArgs),

    /// Listen for events from a server.
    Listen(Listen),
}

#[derive(Parser)]
struct EchoArgs {
    /// UUID of the server to use.
    ///
    /// If this is not specified, then the first server is used that is found.
    #[clap(short, long)]
    server: Option<ObjectUuid>,

    /// String to send to the server.
    echo: String,
}

#[derive(Parser)]
struct Listen {
    /// UUID of the server to use.
    ///
    /// If this is not specified, then the first server is used that is found.
    #[clap(short, long)]
    server: Option<ObjectUuid>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("Connection to broker at {}.", args.bus);

    let stream = TcpStream::connect(&args.bus)
        .await
        .with_context(|| anyhow!("failed to connect to broker at {}", args.bus))?;
    let transport = TokioTransport::new(stream);

    let client = Client::connect(transport)
        .await
        .with_context(|| anyhow!("failed to connect to broker at {}", args.bus))?;
    let handle = client.handle().clone();
    let join = tokio::spawn(client.run());

    match args.cmd {
        Command::Server => server(&handle).await?,
        Command::List => list(&handle).await?,
        Command::Echo(args) => echo(&handle, args).await?,
        Command::EchoAll(args) => echo_all(&handle, args).await?,
        Command::Listen(args) => listen(&handle, args).await?,
    }

    log::info!("Shutting down connection to the broker.");

    handle.shutdown();
    join.await
        .with_context(|| anyhow!("failed to shut down client"))?
        .with_context(|| anyhow!("failed to shut down client"))?;

    Ok(())
}

async fn server(bus: &Handle) -> Result<()> {
    let object = bus.create_object(ObjectUuid::new_v4()).await?;
    let mut echo = Echo::create(&object).await?;
    let events = echo.event_emitter();

    log::info!("Starting echo server {}.", object.id().uuid);

    loop {
        let function = tokio::select! {
            // Services are essentially asynchronous streams of incoming function calls. They also
            // implement the `Stream` trait from the `future` crate, which is however not used here.
            function = echo.next_function_call() => {
                match function {
                    Ok(Some(function)) => function,

                    // `None` is returned when the service is destroyed or when the connection to
                    // the broker shuts down.
                    Ok(None) => return Err(anyhow!("broker shut down")),

                    // Function calls can be invalid either because their ID is unknown or because
                    // the arguments to the call failed to deserialize.
                    Err(e) => {
                        log::error!("Received an invalid function call: {e}.");
                        continue;
                    }
                }
            }

            signal = signal::ctrl_c() => {
                signal.with_context(|| anyhow!("failed to listen for CTRL-C"))?;
                break Ok(());
            }
        };

        // `EchoFunction` is a generated enum, that has one variant for each defined
        // function. Function arguments (if any) and a reply object are contained in each
        // variant. Aldrin does not impose any restrictions on when the reply is sent. If
        // e.g. processing the function call takes some time, then it is fine to hang onto the reply
        // object until then. If the reply object is dropped, then the caller will be notified that
        // the call has been aborted.
        match function {
            EchoFunction::Echo(echo, reply) => {
                log::info!("echo(\"{echo}\") called.");

                if !echo.is_empty() {
                    // Here, we echo the same value back to the caller.
                    reply.ok(&echo)?;
                } else {
                    reply.err(&EchoEchoError::EmptyString)?;
                }
            }

            EchoFunction::EchoAll(echo, reply) => {
                log::info!("echo_all(\"{echo}\") called.");

                if !echo.is_empty() {
                    // This emits an event to all subscribed clients. If there is no such client,
                    // then the event will not even be sent to the broker. In any case, the event
                    // will be sent at most once. The broker then dispatches it further to other
                    // clients.
                    events.echoed_to_all(&echo)?;

                    // No value is sent back to the caller.
                    reply.ok()?;
                } else {
                    reply.err(&EchoEchoAllError::EmptyString)?;
                }
            }
        }
    }
}

async fn list(bus: &Handle) -> Result<()> {
    // `Discoverer`s are used to find objects on the bus that implement some specific set of
    // services. Every call to `add_object` registers interest in one type of object.
    let mut discoverer = bus
        .create_discoverer()
        .any((), [ECHO_UUID])
        .build_current_only()
        .await?;

    let mut found = false;

    // The discoverer will emit events for every object it has found on the bus that matches the
    // criteria. Through the event, we can get the object's ID and the relevant service IDs.
    while let Some(event) = discoverer.next_event().await {
        log::info!("Found echo server {}.", event.object_id().uuid);
        found = true;
    }

    if !found {
        log::warn!("No echo servers found.");
    }

    Ok(())
}

async fn echo(bus: &Handle, args: EchoArgs) -> Result<()> {
    let echo = get_echo(bus, args.server).await?;
    log::info!("Calling echo(\"{}\") on {}.", args.echo, echo.id().uuid);

    // This is what a function call in Aldrin looks like on the client's side. Errors can happen
    // before and after sending the request to the broker. Both `?`s handle errors on the protocol
    // level, such as when e.g. the server shuts down during any of these calls. The `match` then
    // operates on the reply from the echo server (`Result<String, EchoEchoError>`).
    match echo.echo(&args.echo)?.await? {
        Ok(reply) => {
            log::info!("Server replied with: {reply}");
            Ok(())
        }

        Err(EchoEchoError::EmptyString) => Err(anyhow!("empty string")),
    }
}

async fn echo_all(bus: &Handle, args: EchoArgs) -> Result<()> {
    let echo = get_echo(bus, args.server).await?;
    log::info!("Calling echo_all(\"{}\") on {}.", args.echo, echo.id().uuid);

    // This is just like `echo` above, except that the server returns no data. Instead, it will emit
    // an event with the value we sent. The event can be seen by having another instance of this
    // example running with the `listen` subcommand.
    match echo.echo_all(&args.echo)?.await? {
        Ok(()) => Ok(()),
        Err(EchoEchoAllError::EmptyString) => Err(anyhow!("empty string")),
    }
}

async fn listen(bus: &Handle, args: Listen) -> Result<()> {
    let echo = get_echo(bus, args.server).await?;

    // Events are delivered through the type `EchoEvents` that is part of the generated
    // code. Specific events need to be subscribed to, or else the broker will not forward them to
    // this client. In fact, when there are no subscribers to an event at all, they will not even be
    // sent to the broker in the first place. As the name suggests, `subscribe_all` will subscribe
    // to all events of the Echo service.
    let mut events = echo.events();
    events.subscribe_all().await?;

    log::info!("Listen to events from {}.", echo.id().uuid);

    loop {
        let event = tokio::select! {
            event = events.next_event() => {
                match event {
                    Ok(Some(event)) => event,

                    // Happens when either the service is destroyed or the connection to the broker
                    // shuts down.
                    Ok(None) => break Ok(()),

                    // As with functions, events can be invalid, either because they have an unknown
                    // ID or because their associated value failed to deserialize.
                    Err(e) => {
                        log::error!("Received an invalid event: {e}.");
                        continue;
                    }
                }
            }

            signal = signal::ctrl_c() => {
                signal.with_context(|| anyhow!("failed to listen for CTRL-C"))?;
                break Ok(());
            }
        };

        // `EchoEvent` is a generated enum that has one variant for each event.
        match event {
            EchoEvent::EchoedToAll(echo) => log::info!("Received event: EchoedToAll(\"{echo}\")."),
        }
    }
}

async fn get_echo(bus: &Handle, object_uuid: Option<ObjectUuid>) -> Result<EchoProxy> {
    // Find either a specific object (when `object_uuid` is `Some`) or any object implementing the
    // Echo service. The `find_object` function is a convenience wrapper for `Discoverer`, which is
    // more concise when looking for a single object as a one-shot operation.
    let (_, [service_id]) = bus
        .find_object(object_uuid, &[ECHO_UUID])
        .await?
        .ok_or_else(|| anyhow!("echo server not found"))?;

    let echo = EchoProxy::bind(bus.clone(), service_id).await?;
    Ok(echo)
}
