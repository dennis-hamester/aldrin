use aldrin_broker::core::channel::Unbounded;
use aldrin_broker::core::message::Message as ProtoMessage;
use aldrin_broker::core::transport::AsyncTransport;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};

pub struct Runtime {
    tasks: Vec<Option<SpawnedTask>>,
}

impl Runtime {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn spawn(&mut self, fut: impl Future<Output = ()> + 'static) {
        self.tasks.push(Some(SpawnedTask::new(fut)));
    }

    pub fn receive(&mut self, mut client: &mut Unbounded) -> Result<Option<ProtoMessage>, ()> {
        let ready = ReadyFlag::new();
        let waker = ready.clone().into();
        let mut context = Context::from_waker(&waker);

        let mut progress = true;

        while progress {
            progress = false;

            if ready.is_ready() {
                ready.reset();

                match Pin::new(&mut client).receive_poll(&mut context) {
                    Poll::Ready(Ok(msg)) => return Ok(Some(msg)),
                    Poll::Ready(Err(_)) => return Err(()),
                    Poll::Pending => progress |= ready.is_ready(),
                }
            }

            progress |= self.poll_all_tasks();
        }

        Ok(None)
    }

    pub fn send(&mut self, mut client: &mut Unbounded, msg: ProtoMessage) -> Result<(), ()> {
        self.send_ready(client)?;
        Pin::new(&mut client).send_start(msg).map_err(|_| ())?;
        self.flush(client)?;
        Ok(())
    }

    fn send_ready(&mut self, mut client: &mut Unbounded) -> Result<(), ()> {
        let ready = ReadyFlag::new();
        let waker = ready.clone().into();
        let mut context = Context::from_waker(&waker);

        let mut progress;

        loop {
            progress = false;

            if ready.is_ready() {
                ready.reset();

                match Pin::new(&mut client).send_poll_ready(&mut context) {
                    Poll::Ready(Ok(())) => break Ok(()),
                    Poll::Ready(Err(_)) => return Err(()),
                    Poll::Pending => progress |= ready.is_ready(),
                }
            }

            progress |= self.poll_all_tasks();
            if !progress {
                panic!("cannot send message");
            }
        }
    }

    fn flush(&mut self, mut client: &mut Unbounded) -> Result<(), ()> {
        let ready = ReadyFlag::new();
        let waker = ready.clone().into();
        let mut context = Context::from_waker(&waker);

        let mut progress;

        loop {
            progress = false;

            if ready.is_ready() {
                ready.reset();

                match Pin::new(&mut client).send_poll_flush(&mut context) {
                    Poll::Ready(Ok(())) => break Ok(()),
                    Poll::Ready(Err(_)) => return Err(()),
                    Poll::Pending => progress |= ready.is_ready(),
                }
            }

            progress |= self.poll_all_tasks();
            if !progress {
                panic!("cannot flush client");
            }
        }
    }

    pub fn poll_all_tasks(&mut self) -> bool {
        for task in &mut self.tasks {
            let Some(ref mut inner) = task else {
                continue;
            };

            if inner.poll() {
                *task = None;
            }
        }

        self.tasks
            .iter()
            .filter_map(|task| task.as_ref())
            .any(|task| task.is_ready())
    }
}

struct SpawnedTask {
    fut: Pin<Box<dyn Future<Output = ()> + 'static>>,
    ready: Arc<ReadyFlag>,
    waker: Waker,
}

impl SpawnedTask {
    fn new(fut: impl Future<Output = ()> + 'static) -> Self {
        let ready = ReadyFlag::new();
        let waker = ready.clone().into();

        Self {
            fut: Box::pin(fut),
            ready,
            waker,
        }
    }

    fn is_ready(&self) -> bool {
        self.ready.is_ready()
    }

    fn poll(&mut self) -> bool {
        self.ready.reset();

        self.fut
            .as_mut()
            .poll(&mut Context::from_waker(&self.waker))
            .is_ready()
    }
}

struct ReadyFlag {
    flag: AtomicBool,
}

impl ReadyFlag {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            flag: AtomicBool::new(true),
        })
    }

    fn set(&self) {
        self.flag.store(true, Ordering::Relaxed);
    }

    fn reset(&self) {
        self.flag.store(false, Ordering::Relaxed);
    }

    fn is_ready(&self) -> bool {
        self.flag.load(Ordering::Relaxed)
    }
}

impl Wake for ReadyFlag {
    fn wake(self: Arc<Self>) {
        self.set();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.set();
    }
}
