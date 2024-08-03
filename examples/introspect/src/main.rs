#[cfg(feature = "introspection")]
use aldrin::core::introspection::{Enum, Introspection, Layout, Service, Struct};
use aldrin::core::tokio::TokioTransport;
use aldrin::core::{BusEvent, BusListenerFilter, BusListenerScope, TypeId};
use aldrin::low_level::Proxy;
use aldrin::{Client, Handle};
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use std::collections::{BTreeMap, BTreeSet};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::TcpStream;

const BUS_DEFAULT: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 24940);

/// Introspection example.
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
    /// List all objects and their services.
    List,

    /// Query an introspection of a specific type.
    Query {
        /// The type id to query.
        type_id: TypeId,

        /// Also query all referenced types.
        #[clap(short, long)]
        full: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let stream = TcpStream::connect(&args.bus)
        .await
        .with_context(|| anyhow!("failed to connect to broker at {}", args.bus))?;

    // Setting nodelay on the TCP socket can vastly improve latencies as Aldrin messages are
    // typically small.
    stream.set_nodelay(true)?;

    let transport = TokioTransport::new(stream);

    let client = Client::connect(transport)
        .await
        .with_context(|| anyhow!("failed to connect to broker at {}", args.bus))?;
    let handle = client.handle().clone();
    let join = tokio::spawn(client.run());

    match args.cmd {
        Command::List => list(&handle).await?,

        Command::Query { type_id, full } => {
            if full {
                query_full(&handle, type_id).await?
            } else {
                query(&handle, type_id).await?
            }
        }
    }

    handle.shutdown();
    join.await
        .with_context(|| anyhow!("failed to shut down client"))?
        .with_context(|| anyhow!("failed to shut down client"))?;

    Ok(())
}

async fn list(bus: &Handle) -> Result<()> {
    let mut bus_listener = bus.create_bus_listener().await?;
    bus_listener.add_filter(BusListenerFilter::any_object())?;
    bus_listener.add_filter(BusListenerFilter::any_object_any_service())?;
    bus_listener.start(BusListenerScope::Current).await?;

    let mut objects: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();

    while let Some(event) = bus_listener.next_event().await {
        match event {
            BusEvent::ObjectCreated(object_id) => {
                objects.entry(object_id).or_default();
            }

            BusEvent::ServiceCreated(service_id) => {
                objects
                    .entry(service_id.object_id)
                    .or_default()
                    .insert(service_id);
            }

            BusEvent::ObjectDestroyed(_) | BusEvent::ServiceDestroyed(_) => unreachable!(),
        }
    }

    if objects.is_empty() {
        println!("No objects found.");
    }

    for (object_id, services) in objects {
        println!("Object {}: {} service(s)", object_id.uuid, services.len());

        for service_id in services {
            let proxy = Proxy::new(bus, service_id).await?;

            println!("|- Service {}", service_id.uuid);

            #[cfg(feature = "introspection")]
            if let Some(introspection) = proxy.query_introspection().await? {
                println!(
                    "|    Name:    {}::{}",
                    introspection.schema(),
                    introspection.name()
                );
            } else {
                println!("|    Name:    N/A");
            }

            #[cfg(not(feature = "introspection"))]
            println!("|    Name:    N/A");

            if let Some(type_id) = proxy.type_id() {
                println!("|    Type id: {}", type_id);
            } else {
                println!("|    Type id: N/A");
            }
        }

        println!();
    }

    Ok(())
}

#[cfg(feature = "introspection")]
async fn query_full(bus: &Handle, type_id: TypeId) -> Result<()> {
    let mut pending = vec![type_id];
    let mut available = BTreeMap::new();
    let mut unavailable = BTreeSet::new();

    while let Some(type_id) = pending.pop() {
        if available.contains_key(&type_id) || unavailable.contains(&type_id) {
            continue;
        }

        match bus.query_introspection(type_id).await? {
            Some(introspection) => {
                pending.extend(introspection.inner_type_ids());
                available.insert(type_id, introspection);
            }

            None => {
                unavailable.insert(type_id);
            }
        }
    }

    let mut first = true;

    for introspection in available.values() {
        if let Layout::Service(service) = introspection.layout() {
            print_service(service, introspection);
            first = false;
            break;
        }
    }

    for (_, introspection) in available {
        if let Layout::Service(_) = introspection.layout() {
            continue;
        }

        if first {
            first = false;
        } else {
            println!();
        }

        match introspection.layout() {
            Layout::Struct(struct_) => print_struct(struct_, &introspection),
            Layout::Enum(enum_) => print_enum(enum_, &introspection),
            Layout::Service(_) => {}
        }
    }

    if !unavailable.is_empty() {
        if !first {
            println!();
        }

        println!("Unavailable type ids:");

        for type_id in unavailable {
            println!("|- {type_id}");
        }
    }

    Ok(())
}

#[cfg(not(feature = "introspection"))]
async fn query_full(_bus: &Handle, type_id: TypeId) -> Result<()> {
    println!("Unavailable type ids:");
    println!("|- {type_id}");

    Ok(())
}

#[cfg(feature = "introspection")]
async fn query(bus: &Handle, type_id: TypeId) -> Result<()> {
    let Some(introspection) = bus.query_introspection(type_id).await? else {
        println!("No introspection available for {type_id}.");
        return Ok(());
    };

    match introspection.layout() {
        Layout::Service(service) => print_service(service, &introspection),
        Layout::Struct(struct_) => print_struct(struct_, &introspection),
        Layout::Enum(enum_) => print_enum(enum_, &introspection),
    }

    let mut inner_types = introspection.iter_inner_types().peekable();
    if inner_types.peek().is_some() {
        println!();
        println!("Inner types:");

        for (schema, name, type_id) in inner_types {
            println!("|- {schema}::{name}: {type_id}");
        }
    }

    Ok(())
}

#[cfg(not(feature = "introspection"))]
async fn query(_bus: &Handle, type_id: TypeId) -> Result<()> {
    println!("No introspection available for {type_id}.");
    Ok(())
}

#[cfg(feature = "introspection")]
fn print_service(service: &Service, introspection: &Introspection) {
    println!("service {}::{} {{", introspection.schema(), service.name());
    println!("    uuid = {};", service.uuid());
    println!("    version = {};", service.version());

    let mut first = true;
    for function in service.functions().values() {
        if first {
            println!();
        }

        if function.args().is_some() || function.ok().is_some() || function.err().is_some() {
            if !first {
                println!();
            }

            println!("    fn {} @ {} {{", function.name(), function.id());

            if let Some(args) = function.args() {
                println!("        args = {args};");
            }

            if let Some(ok) = function.ok() {
                println!("        ok = {ok};");
            }

            if let Some(err) = function.err() {
                println!("        err = {err};");
            }

            println!("    }}");
            first = true;
        } else {
            println!("    fn {} @ {};", function.name(), function.id());
            first = false;
        }
    }

    let mut first = true;
    for event in service.events().values() {
        if first {
            println!();
        }

        if let Some(event_type) = event.event_type() {
            println!(
                "    event {} @ {} = {};",
                event.name(),
                event.id(),
                event_type
            );
        } else {
            println!("    event {} @ {};", event.name(), event.id());
        }

        first = false;
    }

    println!("}}");
}

#[cfg(feature = "introspection")]
fn print_struct(struct_: &Struct, introspection: &Introspection) {
    println!("struct {}::{} {{", introspection.schema(), struct_.name());

    for field in struct_.fields().values() {
        if field.is_required() {
            println!(
                "    required {} @ {} = {};",
                field.name(),
                field.id(),
                field.field_type()
            );
        } else {
            println!(
                "    {} @ {} = {};",
                field.name(),
                field.id(),
                field.field_type()
            );
        }
    }

    println!("}}");
}

#[cfg(feature = "introspection")]
fn print_enum(enum_: &Enum, introspection: &Introspection) {
    println!("enum {}::{} {{", introspection.schema(), enum_.name());

    for variant in enum_.variants().values() {
        print!("    {} @ {}", variant.name(), variant.id());

        if let Some(variant_type) = variant.variant_type() {
            print!(" = {variant_type}");
        }

        println!(";");
    }

    println!("}}");
}
