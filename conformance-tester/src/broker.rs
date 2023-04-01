use crate::util::FutureExt;
use anyhow::{anyhow, Context, Error, Result};
use std::ffi::OsStr;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::task::JoinHandle;
use tokio::time::Instant;

#[derive(Debug)]
pub struct Broker {
    child: Child,
    stderr: JoinHandle<Vec<u8>>,
    _port: u16,
}

impl Broker {
    pub async fn new(broker: &OsStr, startup_timeout: Duration) -> Result<Self> {
        let mut child = Command::new(broker)
            .kill_on_drop(true)
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| anyhow!("failed to spawn broker process"))?;

        let mut port = String::new();
        BufReader::new(child.stdout.take().unwrap())
            .read_line(&mut port)
            .timeout(startup_timeout)
            .await
            .map_err(|_| anyhow!("timeout while reading the port from broker's stdout"))?
            .with_context(|| anyhow!("failed to read the port from broker's stdout"))?;
        let port = port
            .trim_end()
            .parse()
            .with_context(|| anyhow!("failed to parse the port from `{port}`"))?;

        let mut stderr = child.stderr.take().unwrap();
        let stderr = tokio::spawn(async move {
            let mut buf = Vec::new();
            stderr.read_to_end(&mut buf).await.ok();
            buf
        });

        Ok(Self {
            child,
            stderr,
            _port: port,
        })
    }

    pub async fn shut_down(&mut self, timeout: Instant) -> Result<()> {
        match self.child.wait().timeout_at(timeout).await {
            Ok(Ok(status)) => {
                if status.success() {
                    Ok(())
                } else if let Some(code) = status.code() {
                    Err(anyhow!("broker shut down with code {}", code))
                } else {
                    Err(anyhow!("broker didn't shut down successfully"))
                }
            }

            Ok(Err(e)) => Err(Error::new(e).context(anyhow!("failed to shut down broker"))),
            Err(_) => Err(anyhow!("timeout while shutting down broker")),
        }
    }

    pub async fn take_stderr(&mut self, timeout: Instant) -> Result<Vec<u8>> {
        match (&mut self.stderr).timeout_at(timeout).await {
            Ok(Ok(stderr)) => Ok(stderr),
            Ok(Err(e)) => Err(Error::new(e).context(anyhow!("failed to capture stderr"))),
            Err(_) => Err(anyhow!("timeout while capturing stderr")),
        }
    }
}
