use crate::{Broker, BrokerError, BrokerHandle};
use aldrin_client::{Client, Handle as ClientHandle, RunError};
use aldrin_util::channel::{self, Disconnected};
use std::ops::Deref;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time;

const JOIN_TIMEOUT: Duration = Duration::from_millis(1000);

pub struct TestBroker {
    handle: BrokerHandle,
    join: JoinHandle<Result<(), BrokerError>>,
}

impl TestBroker {
    pub fn new() -> Self {
        let broker = Broker::new();
        let handle = broker.handle().clone();
        let join = tokio::spawn(broker.run());

        TestBroker { handle, join }
    }

    pub async fn join_idle(self) {
        self.handle.shutdown_idle();
        time::timeout(JOIN_TIMEOUT, self.join)
            .await
            .unwrap() // timeout
            .unwrap() // join
            .unwrap(); // broker
    }

    pub async fn add_client(&self) -> TestClient {
        let (t1, t2) = channel::unbounded();
        let client = tokio::spawn(Client::connect(t2));
        let conn = self.handle.add_connection(t1).await.unwrap();
        let client = client.await.unwrap().unwrap();
        tokio::spawn(conn.run());

        let handle = client.handle().clone();
        let join = tokio::spawn(client.run());

        TestClient { handle, join }
    }
}

impl Deref for TestBroker {
    type Target = BrokerHandle;

    fn deref(&self) -> &BrokerHandle {
        &self.handle
    }
}

pub struct TestClient {
    handle: ClientHandle,
    join: JoinHandle<Result<(), RunError<Disconnected>>>,
}

impl TestClient {
    pub async fn join(self) {
        self.join.await.unwrap().unwrap()
    }
}

impl Deref for TestClient {
    type Target = ClientHandle;

    fn deref(&self) -> &ClientHandle {
        &self.handle
    }
}
