#![no_main]

mod context;
mod message_le;
mod runtime;
mod serial_le;
mod uuid_le;
mod value_epoch;

use aldrin_broker::core::channel::{self, Unbounded};
use aldrin_broker::core::message::MessageOps;
use aldrin_broker::{Broker, BrokerHandle};
use arbitrary::Arbitrary;
use context::Context;
use libfuzzer_sys::fuzz_target;
use message_le::{MessageLe, UpdateContext};
use runtime::Runtime;
use value_epoch::ValueEpoch;

#[derive(Debug, Arbitrary)]
enum Step {
    Connect,
    Disconnect(u8),
    Send(Send),
}

#[derive(Debug, Arbitrary)]
struct Send {
    client: u8,
    msg: MessageLe,
    value_epoch: Option<ValueEpoch>,
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
            Step::Send(send) => self.send(send),
        }
    }

    fn connect(&mut self) {
        let (channel1, channel2) = channel::unbounded();

        self.clients.push(Some(channel1));

        let mut broker = self.broker.clone();
        self.runtime.spawn(async move {
            if let Ok(conn) = broker.connect(channel2).await {
                let _ = conn.run().await;
            }
        });
    }

    fn disconnect(&mut self, client: u8) {
        if let Some(client) = self.clients.get_mut(client as usize) {
            *client = None;
        }
    }

    fn send(&mut self, send: Send) {
        let Some(client) = self.clients.get_mut(send.client as usize) else {
            return;
        };

        let Some(inner) = client else {
            return;
        };

        let mut msg = send.msg.to_core(&self.context);

        if let Some(value_epoch) = send.value_epoch {
            let _ = msg.convert_value(None, value_epoch.into());
        }

        if self.runtime.send(inner, msg).is_err() {
            *client = None;
        }
    }

    fn drain_clients(&mut self) {
        for client in &mut self.clients {
            let Some(inner) = client else {
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
