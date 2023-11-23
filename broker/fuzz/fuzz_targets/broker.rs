#![no_main]

mod context;
mod message_le;
mod runtime;
mod serial_le;
mod uuid_le;

use aldrin_broker::core::channel::{self, Unbounded};
use aldrin_broker::{Broker, BrokerHandle};
use arbitrary::Arbitrary;
use context::Context;
use libfuzzer_sys::fuzz_target;
use message_le::{MessageLe, UpdateContext};
use runtime::Runtime;

#[derive(Debug, Arbitrary)]
enum Step {
    Connect,
    Disconnect(u8),
    Send { client: u8, msg: MessageLe },
}

struct Fuzzer {
    runtime: Runtime,
    broker: BrokerHandle,
    clients: Vec<Option<Unbounded>>,
    context: Context,
}

impl Fuzzer {
    fn new() -> Self {
        let mut runtime = Runtime::new();

        let broker = Broker::new();
        let handle = broker.handle().clone();
        runtime.spawn(broker.run());

        Fuzzer {
            runtime,
            broker: handle,
            clients: Vec::new(),
            context: Context::new(),
        }
    }

    fn run(mut self, steps: Vec<Step>) {
        for step in steps {
            self.execute_step(step);
            self.drain_clients();
            self.runtime.poll_all_tasks();
        }
    }

    fn execute_step(&mut self, step: Step) {
        match step {
            Step::Connect => self.connect(),
            Step::Disconnect(client) => self.disconnect(client),
            Step::Send { client, msg } => self.send(client, msg),
        }
    }

    fn connect(&mut self) {
        let (channel1, channel2) = channel::unbounded();

        self.clients.push(Some(channel1));

        let mut broker = self.broker.clone();
        self.runtime.spawn(async move {
            if let Ok(conn) = broker.connect(channel2).await {
                conn.run().await.ok();
            }
        });
    }

    fn disconnect(&mut self, client: u8) {
        if let Some(client) = self.clients.get_mut(client as usize) {
            *client = None;
        }
    }

    fn send(&mut self, client: u8, msg: MessageLe) {
        let Some(client) = self.clients.get_mut(client as usize) else {
            return;
        };

        let Some(ref mut inner) = client else {
            return;
        };

        let msg = msg.to_core(&self.context);

        if self.runtime.send(inner, msg).is_err() {
            *client = None;
        }
    }

    fn drain_clients(&mut self) {
        for client in &mut self.clients {
            let Some(ref mut inner) = client else {
                continue;
            };

            loop {
                match self.runtime.receive(inner) {
                    Ok(Some(msg)) => {
                        msg.update_context(&mut self.context);
                    }

                    Ok(None) => break,

                    Err(()) => {
                        *client = None;
                        break;
                    }
                }
            }
        }
    }
}

fuzz_target!(|steps: Vec<Step>| {
    Fuzzer::new().run(steps);
});
