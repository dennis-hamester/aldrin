#[cfg(feature = "introspection")]
use aldrin::core::introspection::{
    BuiltInType, Enum, Event, Function, Introspection, Layout, Newtype, Service, Struct,
};
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
            if let Some(svc) = proxy
                .query_introspection()
                .await?
                .as_ref()
                .and_then(Introspection::as_service_layout)
            {
                println!("|    Name:    {}::{}", svc.schema(), svc.name());
            } else {
                println!("|    Name:    N/A");
            }

            #[cfg(not(feature = "introspection"))]
            println!("|    Name:    N/A");

            if let Some(type_id) = proxy.type_id() {
                println!("|    Type id: {type_id}");
            } else {
                println!("|    Type id: N/A");
            }
        }

        println!();
    }

    Ok(())
}

#[cfg(feature = "introspection")]
async fn query(bus: &Handle, type_id: TypeId) -> Result<()> {
    match bus.query_introspection(type_id).await? {
        Some(introspection) => print_introspection(&introspection, &BTreeMap::new(), false),
        None => println!("No introspection available for {type_id}."),
    }

    Ok(())
}

#[cfg(not(feature = "introspection"))]
async fn query(_bus: &Handle, type_id: TypeId) -> Result<()> {
    println!("No introspection available for {type_id}.");
    Ok(())
}

#[cfg(feature = "introspection")]
async fn query_full(bus: &Handle, type_id: TypeId) -> Result<()> {
    use std::collections::HashSet;

    let Some(introspection) = bus.query_introspection(type_id).await? else {
        println!("No introspection available for {type_id}.");
        return Ok(());
    };

    let mut db = BTreeMap::new();
    let mut unavailable = HashSet::new();
    let mut pending = vec![type_id];

    while let Some(type_id) = pending.pop() {
        let Some(introspection) = bus.query_introspection(type_id).await? else {
            unavailable.insert(type_id);
            continue;
        };

        for type_id in introspection.iter_references() {
            if !db.contains_key(&type_id) && !unavailable.contains(&type_id) {
                pending.push(type_id);
            }
        }

        db.insert(type_id, introspection);
    }

    print_introspection(&introspection, &db, true);

    for introspection in db.values() {
        if (introspection.type_id() != type_id) && introspection.as_built_in_layout().is_none() {
            print_introspection(introspection, &db, true);
        }
    }

    Ok(())
}

#[cfg(not(feature = "introspection"))]
async fn query_full(_bus: &Handle, type_id: TypeId) -> Result<()> {
    println!("No introspection available for {type_id}.");
    Ok(())
}

#[cfg(feature = "introspection")]
fn print_introspection(
    introspection: &Introspection,
    db: &BTreeMap<TypeId, Introspection>,
    full: bool,
) {
    match introspection.layout() {
        Layout::BuiltIn(ty) => print_built_in(*ty, db, full),
        Layout::Struct(ty) => print_struct(ty, db, full),
        Layout::Enum(ty) => print_enum(ty, db, full),
        Layout::Service(ty) => print_service(ty, db, full),
        Layout::Newtype(ty) => print_newtype(ty, db, full),
    }
}

#[cfg(feature = "introspection")]
fn print_built_in(ty: BuiltInType, db: &BTreeMap<TypeId, Introspection>, full: bool) {
    print_built_in_type_name(ty, db, full);
    println!();
    println!();
}

#[cfg(feature = "introspection")]
fn print_struct(ty: &Struct, db: &BTreeMap<TypeId, Introspection>, full: bool) {
    println!("struct {}::{} {{", ty.schema(), ty.name());

    for field in ty.fields().values() {
        if field.is_required() {
            print!("    required ");
        } else {
            print!("    ");
        }

        print!("{} @ {} = ", field.name(), field.id());
        print_type_name(field.field_type(), db, full);
        println!(";");
    }

    if let Some(fallback) = ty.fallback() {
        println!("    {fallback} = fallback;");
    }

    println!("}}");
    println!();
}

#[cfg(feature = "introspection")]
fn print_enum(ty: &Enum, db: &BTreeMap<TypeId, Introspection>, full: bool) {
    println!("enum {}::{} {{", ty.schema(), ty.name());

    for var in ty.variants().values() {
        print!("    {} @ {}", var.name(), var.id());

        if let Some(var_type) = var.variant_type() {
            print!(" = ");
            print_type_name(var_type, db, full);
        }

        println!(";");
    }

    if let Some(fallback) = ty.fallback() {
        println!("    {fallback} = fallback;");
    }

    println!("}}");
    println!();
}

#[cfg(feature = "introspection")]
fn print_service(ty: &Service, db: &BTreeMap<TypeId, Introspection>, full: bool) {
    println!("service {}::{} {{", ty.schema(), ty.name());
    println!("    uuid = {};", ty.uuid());
    println!("    version = {};", ty.version());

    for func in ty.functions().values() {
        println!();
        print_function(func, db, full);
    }

    for ev in ty.events().values() {
        println!();
        print_event(ev, db, full);
    }

    if let Some(fallback) = ty.function_fallback() {
        println!();
        println!("    {fallback} = fallback;");
    }

    if let Some(fallback) = ty.event_fallback() {
        println!();
        println!("    {fallback} = fallback;");
    }

    println!("}}");
    println!();
}

#[cfg(feature = "introspection")]
fn print_function(func: &Function, db: &BTreeMap<TypeId, Introspection>, full: bool) {
    print!("    fn {} @ {}", func.name(), func.id());

    if let (None, Some(ty), None) = (func.args(), func.ok(), func.err()) {
        print!(" = ");
        print_type_name(ty, db, full);
        println!(";");
    } else if func.args().is_some() || func.ok().is_some() || func.err().is_some() {
        println!(" {{");

        if let Some(ty) = func.args() {
            print!("        args = ");
            print_type_name(ty, db, full);
            println!(";");
        }

        if let Some(ty) = func.ok() {
            print!("        ok = ");
            print_type_name(ty, db, full);
            println!(";");
        }

        if let Some(ty) = func.err() {
            print!("        err = ");
            print_type_name(ty, db, full);
            println!(";");
        }

        println!("    }}");
    } else {
        println!(";");
    }
}

#[cfg(feature = "introspection")]
fn print_event(ev: &Event, db: &BTreeMap<TypeId, Introspection>, full: bool) {
    print!("    event {} @ {}", ev.name(), ev.id());

    if let Some(ty) = ev.event_type() {
        print!(" = ");
        print_type_name(ty, db, full);
    }

    println!(";");
}

#[cfg(feature = "introspection")]
fn print_newtype(ty: &Newtype, db: &BTreeMap<TypeId, Introspection>, full: bool) {
    print!("newtype {}::{} = ", ty.schema(), ty.name());
    print_type_name(ty.target_type(), db, full);
    println!(";");
    println!();
}

#[cfg(feature = "introspection")]
fn print_type_name(ty: TypeId, db: &BTreeMap<TypeId, Introspection>, full: bool) {
    if !full {
        print!("type_id({ty})");
        return;
    }

    let Some(introspection) = db.get(&ty) else {
        print!("type_id({ty})");
        return;
    };

    match introspection.layout() {
        Layout::BuiltIn(ty) => print_built_in_type_name(*ty, db, full),
        Layout::Struct(ty) => print!("{}::{}", ty.schema(), ty.name()),
        Layout::Enum(ty) => print!("{}::{}", ty.schema(), ty.name()),
        Layout::Service(ty) => print!("{}::{}", ty.schema(), ty.name()),
        Layout::Newtype(ty) => print!("{}::{}", ty.schema(), ty.name()),
    }
}

#[cfg(feature = "introspection")]
fn print_built_in_type_name(ty: BuiltInType, db: &BTreeMap<TypeId, Introspection>, full: bool) {
    match ty {
        BuiltInType::Bool => print!("bool"),
        BuiltInType::U8 => print!("u8"),
        BuiltInType::I8 => print!("i8"),
        BuiltInType::U16 => print!("u16"),
        BuiltInType::I16 => print!("i16"),
        BuiltInType::U32 => print!("u32"),
        BuiltInType::I32 => print!("i32"),
        BuiltInType::U64 => print!("u64"),
        BuiltInType::I64 => print!("i64"),
        BuiltInType::F32 => print!("f32"),
        BuiltInType::F64 => print!("f64"),
        BuiltInType::String => print!("string"),
        BuiltInType::Uuid => print!("uuid"),
        BuiltInType::ObjectId => print!("object_id"),
        BuiltInType::ServiceId => print!("service_id"),
        BuiltInType::Value => print!("value"),

        BuiltInType::Option(ty) => {
            print!("option<");
            print_type_name(ty, db, full);
            print!(">");
        }

        BuiltInType::Box(ty) => {
            print!("box<");
            print_type_name(ty, db, full);
            print!(">");
        }

        BuiltInType::Vec(ty) => {
            print!("vec<");
            print_type_name(ty, db, full);
            print!(">");
        }

        BuiltInType::Bytes => print!("bytes"),

        BuiltInType::Map(ty) => {
            print!("map<");
            print_type_name(ty.key(), db, full);
            print!(" -> ");
            print_type_name(ty.value(), db, full);
            print!(">");
        }

        BuiltInType::Set(ty) => {
            print!("set<");
            print_type_name(ty, db, full);
            print!(">");
        }

        BuiltInType::Sender(ty) => {
            print!("sender<");
            print_type_name(ty, db, full);
            print!(">");
        }

        BuiltInType::Receiver(ty) => {
            print!("receiver<");
            print_type_name(ty, db, full);
            print!(">");
        }

        BuiltInType::Lifetime => print!("lifetime"),
        BuiltInType::Unit => print!("unit"),

        BuiltInType::Result(ty) => {
            print!("result<");
            print_type_name(ty.ok(), db, full);
            print!(", ");
            print_type_name(ty.err(), db, full);
            print!(">");
        }

        BuiltInType::Array(ty) => {
            print!("[");
            print_type_name(ty.elem_type(), db, full);
            print!("; {}]", ty.len());
        }
    }
}
