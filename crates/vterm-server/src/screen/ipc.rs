//! Blocking ctl IPC client (coordinator side) with an async façade.
//!
//! One connection = one pipe/UDS stream. A pump thread moves inbound frames
//! off the wire — responses are demultiplexed to their waiting
//! [`IpcClient::request`] caller by id, events go to a broadcast channel.
//! On Windows the pump polls with [`ctl::PipeFrameReader`] (a blocking read
//! on a synchronous pipe would serialize the shared file object and freeze
//! writes); on unix a plain blocking read is fine.
//!
//! When the wire breaks the client marks itself dead and fails every
//! pending request; the supervisor re-attaches (or respawns) on the next
//! use.

use std::collections::HashMap;
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};
use tokio::sync::{broadcast, oneshot};
use tracing::debug;
use crate::screen::ctl as ctl;

/// Per-request timeout (the control plane is low-rate and local).
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
/// Timeout for requests that wait on a human: `WarmupInput` blocks while
/// the user answers the OS consent dialog (the screen-server side waits
/// up to 120 s), which the default 5 s would cut off as a failure.
pub(super) const SLOW_REQUEST_TIMEOUT: Duration = Duration::from_secs(130);
/// How long `connect` waits for the server-side Hello frame.
const HELLO_TIMEOUT: Duration = Duration::from_secs(3);
/// Poll tick of the Windows peek-pump (also the event latency bound).
#[cfg(target_os = "windows")]
const PUMP_TICK: Duration = Duration::from_millis(20);

pub struct IpcClient {
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    pending: Arc<Mutex<HashMap<u64, oneshot::Sender<ctl::ResponsePayload>>>>,
    next_id: AtomicU64,
    dead: Arc<AtomicBool>,
    /// The Hello received at connect time (server identity + udp_port).
    pub hello: ctl::Hello,
}

impl IpcClient {
    /// Connect to the ctl endpoint, read the Hello and start the pump
    /// thread. Blocking; call via `spawn_blocking`.
    pub fn connect(endpoint: &str, events_tx: broadcast::Sender<ctl::Event>) -> Result<Arc<Self>> {
        let (reader, writer) = open_endpoint(endpoint)?;
        // The Hello precedes any request/response traffic; read it inline
        // (deadline-bounded), then hand the reader to the pump thread.
        let (hello, pump_reader) = read_hello(reader)?;
        let client = Arc::new(IpcClient {
            writer: Arc::new(Mutex::new(writer)),
            pending: Arc::new(Mutex::new(HashMap::new())),
            next_id: AtomicU64::new(1),
            dead: Arc::new(AtomicBool::new(false)),
            hello,
        });
        let pending = Arc::clone(&client.pending);
        let dead = Arc::clone(&client.dead);
        std::thread::Builder::new()
            .name("vterm-screen-ctl-pump".into())
            .spawn(move || pump(pump_reader, pending, dead, events_tx))
            .expect("failed to spawn ctl pump thread");
        Ok(client)
    }

    pub fn is_dead(&self) -> bool {
        self.dead.load(Ordering::Relaxed)
    }

    /// Send a request and await the correlated response.
    pub async fn request(&self, cmd: ctl::Cmd) -> Result<ctl::ResponsePayload> {
        self.request_with_timeout(cmd, REQUEST_TIMEOUT).await
    }

    /// [`Self::request`] with an explicit response timeout (see
    /// [`SLOW_REQUEST_TIMEOUT`]).
    pub async fn request_with_timeout(
        &self,
        cmd: ctl::Cmd,
        timeout: Duration,
    ) -> Result<ctl::ResponsePayload> {
        if self.is_dead() {
            bail!("ctl connection is dead");
        }
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let command = format!("{cmd:?}");
        debug!(request_id = id, command, ?timeout, "ctl request sent");
        let (tx, rx) = oneshot::channel();
        self.pending.lock().unwrap().insert(id, tx);
        let frame = ctl::Frame::Request(ctl::Request { id, cmd });
        {
            let mut w = self.writer.lock().unwrap();
            if let Err(e) = ctl::write_frame(&mut *w, &frame) {
                self.pending.lock().unwrap().remove(&id);
                return Err(anyhow!("ctl write failed: {e}"));
            }
        }
        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(payload)) => Ok(payload),
            Ok(Err(_)) => Err(anyhow!("ctl connection lost mid-request")),
            Err(_) => {
                self.pending.lock().unwrap().remove(&id);
                tracing::warn!(request_id = id, command, ?timeout, "ctl request timed out");
                Err(anyhow!("ctl request {id} timed out"))
            }
        }
    }
}

/// The platform pump source: a peek-polling pipe reader on Windows, a plain
/// blocking reader on unix.
#[cfg(target_os = "windows")]
type PumpReader = ctl::PipeFrameReader;
#[cfg(not(target_os = "windows"))]
type PumpReader = Box<dyn std::io::Read + Send>;

/// Open the endpoint, returning (reader, writer) halves.
#[cfg(target_os = "windows")]
fn open_endpoint(endpoint: &str) -> Result<(std::fs::File, Box<dyn Write + Send>)> {
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(endpoint)
        .with_context(|| format!("connect ctl pipe {endpoint}"))?;
    let reader = file.try_clone().context("clone pipe handle")?;
    Ok((reader, Box::new(file)))
}

#[cfg(not(target_os = "windows"))]
fn open_endpoint(
    endpoint: &str,
) -> Result<(std::os::unix::net::UnixStream, Box<dyn Write + Send>)> {
    let stream = std::os::unix::net::UnixStream::connect(endpoint)
        .with_context(|| format!("connect ctl socket {endpoint}"))?;
    let reader = stream.try_clone().context("clone socket")?;
    Ok((reader, Box::new(stream)))
}

/// The Hello read happens before the pump exists; on Windows it is a
/// peek-poll with a deadline (the pipe handle cannot block-read while
/// writes happen on the same file object).
#[cfg(target_os = "windows")]
fn read_hello(reader: std::fs::File) -> Result<(ctl::Hello, PumpReader)> {
    let mut reader = ctl::PipeFrameReader::new(reader);
    let deadline = Instant::now() + HELLO_TIMEOUT;
    loop {
        match reader.poll_frame::<ctl::Frame>() {
            Ok(Some(ctl::Frame::Hello { hello })) => return Ok((hello, reader)),
            Ok(Some(_)) => return Err(anyhow!("expected Hello as the first ctl frame")),
            Ok(None) => {
                if Instant::now() >= deadline {
                    return Err(anyhow!("timed out waiting for the ctl Hello"));
                }
                std::thread::sleep(PUMP_TICK);
            }
            Err(e) => return Err(anyhow!("ctl Hello read failed: {e}")),
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn read_hello(mut reader: std::os::unix::net::UnixStream) -> Result<(ctl::Hello, PumpReader)> {
    use std::io::Read;
    reader
        .set_read_timeout(Some(HELLO_TIMEOUT))
        .context("set read timeout")?;
    let frame = ctl::read_frame::<ctl::Frame>(&mut reader).context("read ctl Hello")?;
    reader
        .set_read_timeout(None)
        .context("clear read timeout")?;
    match frame {
        Some(ctl::Frame::Hello { hello }) => {
            let boxed: Box<dyn Read + Send> = Box::new(reader);
            Ok((hello, boxed))
        }
        Some(_) => Err(anyhow!("expected Hello as the first ctl frame")),
        None => Err(anyhow!("ctl endpoint closed before Hello")),
    }
}

/// The pump: demultiplex inbound frames to pending requests / the event
/// broadcast. Any terminal error marks the client dead and drops every
/// pending waiter (their receivers error out).
#[cfg(target_os = "windows")]
fn pump(
    mut reader: PumpReader,
    pending: Arc<Mutex<HashMap<u64, oneshot::Sender<ctl::ResponsePayload>>>>,
    dead: Arc<AtomicBool>,
    events_tx: broadcast::Sender<ctl::Event>,
) {
    loop {
        match reader.poll_frame::<ctl::Frame>() {
            Ok(Some(frame)) => handle_inbound(frame, &pending, &events_tx),
            Ok(None) => std::thread::sleep(PUMP_TICK),
            Err(_) => break,
        }
    }
    dead.store(true, Ordering::Relaxed);
    pending.lock().unwrap().clear();
}

#[cfg(not(target_os = "windows"))]
fn pump(
    mut reader: PumpReader,
    pending: Arc<Mutex<HashMap<u64, oneshot::Sender<ctl::ResponsePayload>>>>,
    dead: Arc<AtomicBool>,
    events_tx: broadcast::Sender<ctl::Event>,
) {
    loop {
        match ctl::read_frame::<ctl::Frame>(&mut reader) {
            Ok(Some(frame)) => handle_inbound(frame, &pending, &events_tx),
            Ok(None) => {
                debug!("ctl pump: connection closed by peer");
                break;
            }
            Err(e) => {
                debug!("ctl pump: read error: {e}");
                break;
            }
        }
    }
    dead.store(true, Ordering::Relaxed);
    pending.lock().unwrap().clear();
}

fn handle_inbound(
    frame: ctl::Frame,
    pending: &Arc<Mutex<HashMap<u64, oneshot::Sender<ctl::ResponsePayload>>>>,
    events_tx: &broadcast::Sender<ctl::Event>,
) {
    match frame {
        ctl::Frame::Response(resp) => {
            if let Some(tx) = pending.lock().unwrap().remove(&resp.id) {
                let _ = tx.send(resp.result);
            }
        }
        ctl::Frame::Event(ev) => {
            let _ = events_tx.send(ev);
        }
        ctl::Frame::Hello { .. } | ctl::Frame::Request(_) => {
            // Wrong direction on this channel.
        }
    }
}
