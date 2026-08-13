//! v2.0 accept loop: the part of `termy-agent run` that serves a control end
//! over iroh (doc 7.5/7.7, phase A/B).
//!
//! Layering: `p2p` owns endpoint construction and the single-controller
//! gate, `session_table` owns PTY lifecycles, `termstream` owns the wire
//! format - this module only stitches them together:
//!
//! - every inbound connection is dispatched by ALPN; `termy/terminal/1`
//!   connections pass through the [`ControllerGate`], everything else is
//!   closed (`termy/transfer/1` arrives in phase C),
//! - every bi-stream on the admitted connection is either one terminal
//!   session (doc 8.2 handshake: `Open` first) or one directory-tree
//!   list+watch (candidate doc "目录树与双向文件传输" phase 2A: `FsList`
//!   first, see `termstream`'s module doc for why this rides the terminal
//!   connection instead of getting its own ALPN); `serve_bi_stream` reads
//!   the first frame and routes to whichever it is,
//! - the controller connection dying tears down every session under it
//!   (doc 7.5) and reopens the gate.
//!
//! PTY output is read on a dedicated blocking thread and handed to the
//! async side over an mpsc channel - the same shape as V1's
//! `client::spawn_output_pump`, kept rather than redesigned so the two
//! stacks behave identically while they coexist.

use std::sync::{Arc, Mutex};

use iroh::endpoint::{Connection, RecvStream, SendStream, VarInt};
use iroh::Endpoint;
use tokio::sync::mpsc;
use uuid::Uuid;
use std::path::Path;

use crate::config::ShellConfig;
use crate::fs_browse;
use crate::p2p::{admit_controller, ControllerGate, ControllerSlot, ALPN_TERMINAL, ALPN_TRANSFER};
use crate::pty::PtySession;
use crate::session_table::SessionTable;
use crate::termstream::{
    ClosePayload, ErrorPayload, Frame, FrameDecoder, FsChangedPayload, FsListPayload,
    FsListResultPayload, OpenPayload, OpenedPayload, ShellEventPayload, TransferAcceptedPayload,
    TransferChunkPayload, TransferCreditPayload, TransferEntry, TransferFileEndPayload,
    TransferManifestPayload, TransferPullManifestPayload, TransferPullRequestPayload,
    TransferResultPayload,
};
use crate::transfer;
use crate::AgentError;

/// Candidate doc §4.7: a lightweight "structure changed" signal, not
/// real-time sync - matches the local `LocalDirectoryTreeSource`'s
/// debounced `fs.watch`, just via polling instead of a native watcher
/// since the agent takes on no new filesystem-event dependency for it.
const FS_WATCH_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_secs(2);

/// How long the controller has to send the `open` frame after opening a
/// stream before the agent gives up on it. Generous: it only guards against
/// a stream opened and then abandoned.
const OPEN_HANDSHAKE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

pub struct ServeOptions {
    pub shell: ShellConfig,
    pub max_concurrent_sessions: usize,
    /// Fallback transfer destination (doc §7.6) when a `TransferManifest`
    /// names a `sessionId` with no known cwd, or none at all.
    pub receive_root: std::path::PathBuf,
}

/// Serves control ends until the endpoint is closed.
pub async fn serve(endpoint: Endpoint, options: ServeOptions) {
    let options = Arc::new(options);
    let gate = ControllerGate::new();

    while let Some(incoming) = endpoint.accept().await {
        let gate = gate.clone();
        let options = Arc::clone(&options);
        // Handshakes complete in their own task so a slow or stalled peer
        // cannot block the accept loop for everyone else.
        tokio::spawn(async move {
            let connection = match incoming.accept() {
                Ok(accepting) => match accepting.await {
                    Ok(connection) => connection,
                    Err(e) => {
                        tracing::debug!("inbound handshake failed: {e}");
                        return;
                    }
                },
                Err(e) => {
                    tracing::debug!("inbound connection rejected: {e}");
                    return;
                }
            };
            dispatch_connection(connection, gate, options).await;
        });
    }
}

async fn dispatch_connection(
    connection: Connection,
    gate: ControllerGate,
    options: Arc<ServeOptions>,
) {
    match connection.alpn() {
        alpn if alpn == ALPN_TERMINAL => {
            let Some(slot) = admit_controller(&gate, &connection) else {
                tracing::info!(
                    remote = %connection.remote_id().fmt_short(),
                    "second controller refused (CONTROLLER_ALREADY_CONNECTED)"
                );
                return;
            };
            serve_controller(connection, slot, options).await;
        }
        alpn if alpn == ALPN_TRANSFER => {
            // Phase C. Closing with PROTOCOL_ERROR (doc 13) is more honest
            // than accepting a manifest this build cannot act on.
            connection.close(VarInt::from_u32(0x02), b"PROTOCOL_ERROR: transfer not supported yet");
        }
        other => {
            tracing::warn!(alpn = %String::from_utf8_lossy(other), "unknown ALPN");
            connection.close(VarInt::from_u32(0x02), b"PROTOCOL_ERROR: unknown ALPN");
        }
    }
}

/// Serves one admitted controller until its connection dies. Holding `_slot`
/// for the whole body is what enforces doc 7.7 - it drops (reopening the
/// gate) only after every session has been torn down.
async fn serve_controller(connection: Connection, _slot: ControllerSlot, options: Arc<ServeOptions>) {
    let remote = connection.remote_id();
    tracing::info!(remote = %remote.fmt_short(), "controller connected");

    let table = Arc::new(Mutex::new(SessionTable::new(options.max_concurrent_sessions)));

    loop {
        match connection.accept_bi().await {
            Ok((send, recv)) => {
                let table = Arc::clone(&table);
                let options = Arc::clone(&options);
                tokio::spawn(async move {
                    serve_bi_stream(send, recv, table, options).await;
                });
            }
            Err(e) => {
                tracing::info!(remote = %remote.fmt_short(), "controller connection closed: {e}");
                break;
            }
        }
    }

    // Doc 7.5: the connection dying kills every session under it. Dropping
    // the handles terminates each PTY's whole process tree.
    let orphaned = table.lock().expect("session table poisoned").close_all();
    if !orphaned.is_empty() {
        tracing::info!(count = orphaned.len(), "terminating sessions with the connection");
    }
    drop(orphaned);
}

/// What the blocking PTY-reader thread reports to the async side.
enum PtyEvent {
    Output(Vec<u8>),
    Shell {
        name: &'static str,
        source: &'static str,
        exit_code: Option<i32>,
    },
    /// EOF on the master: the shell is gone.
    Exited,
}

/// Every bi-stream on an admitted controller connection starts with a
/// handshake frame that decides what the stream is for (see the module
/// doc): `Open` -> a terminal session (doc 8.2), `FsList` -> a
/// directory-tree list+watch (candidate doc phase 2A), `TransferManifest`
/// -> a note transfer (doc §8.4/8.6/10, see termstream's module doc for why
/// this also isn't `ALPN_TRANSFER`), `TransferPullRequest` -> the reverse
/// direction (candidate doc phase 2B: copying a directory-tree entry into
/// the vault). Reading that one frame here, rather than in each handler, is
/// what lets all four share a connection without their own ALPN.
async fn serve_bi_stream(
    mut send: SendStream,
    mut recv: RecvStream,
    table: Arc<Mutex<SessionTable>>,
    options: Arc<ServeOptions>,
) {
    let mut decoder = FrameDecoder::new();

    let first =
        tokio::time::timeout(OPEN_HANDSHAKE_TIMEOUT, read_frame(&mut recv, &mut decoder)).await;
    match first {
        Ok(Ok(Some(Frame::Open(payload)))) => {
            serve_terminal_session(payload, send, recv, decoder, table, options).await;
        }
        Ok(Ok(Some(Frame::FsList(payload)))) => {
            serve_fs_stream(payload, send, recv, decoder).await;
        }
        Ok(Ok(Some(Frame::TransferManifest(payload)))) => {
            serve_transfer_stream(payload, send, recv, decoder, table, options).await;
        }
        Ok(Ok(Some(Frame::TransferPullRequest(payload)))) => {
            serve_transfer_pull_stream(payload, send, recv, decoder).await;
        }
        Ok(Ok(Some(_))) => {
            send_error(
                &mut send,
                "PROTOCOL_ERROR: expected an open, fsList, transferManifest or transferPullRequest frame",
            )
            .await;
        }
        Ok(Ok(None)) | Ok(Err(_)) => {}
        Err(_) => {
            send_error(&mut send, "PROTOCOL_ERROR: handshake timed out").await;
        }
    }
}

/// One `termy/terminal/1`-style bi-stream = one session (doc 8.2), from
/// just past the `open` handshake (read by `serve_bi_stream`) onward.
async fn serve_terminal_session(
    open: OpenPayload,
    mut send: SendStream,
    mut recv: RecvStream,
    mut decoder: FrameDecoder,
    table: Arc<Mutex<SessionTable>>,
    options: Arc<ServeOptions>,
) {
    // 2. Admission before spawn: refusing at the cap without paying for a
    //    shell start. The authoritative check is still `SessionTable::open`.
    //    (The guard must not live across an await - async Send analysis.)
    let over_cap = {
        let table = table.lock().expect("session table poisoned");
        table.count() >= options.max_concurrent_sessions
    };
    if over_cap {
        let message = AgentError::SessionLimitReached(options.max_concurrent_sessions).to_string();
        send_error(&mut send, &message).await;
        return;
    }

    // 3. Spawn the shell. Same starting directory as V1: the user's home.
    let session_id = Uuid::new_v4();
    let (pty, reader) = match PtySession::spawn(
        &options.shell.program,
        &options.shell.args,
        Some(&crate::config::home_dir()),
        open.cols,
        open.rows,
    ) {
        Ok(spawned) => spawned,
        Err(e) => {
            let message = format!("SHELL_START_FAILED: {}", redact(&e.to_string()));
            send_error(&mut send, &message).await;
            return;
        }
    };
    let shell = pty.shell.clone();

    // Bound to a block so the guard (and the Ok branch's borrow of the
    // table) is gone before any await - same Send-analysis constraint as
    // the pre-check above.
    let admission_error = {
        let mut table = table.lock().expect("session table poisoned");
        table.open(session_id, pty).map(|_| ()).err()
    };
    if let Some(e) = admission_error {
        // Lost the admission race; the just-spawned PTY was dropped (and
        // thereby terminated) by `open` rejecting it.
        send_error(&mut send, &e.to_string()).await;
        return;
    }

    let opened = Frame::Opened(OpenedPayload {
        session_id,
        shell: shell.clone(),
    });
    if write_frame(&mut send, &opened).await.is_err() {
        table.lock().expect("session table poisoned").close(session_id);
        return;
    }
    tracing::info!(session = %session_id, %shell, "terminal session opened");

    // 4. Pump until either side ends the session.
    let (event_tx, mut event_rx) = mpsc::channel::<PtyEvent>(256);
    spawn_output_pump(reader, event_tx);

    let mut buf = vec![0u8; 32 * 1024];
    let close_reason: Option<ClosePayload> = loop {
        tokio::select! {
            event = event_rx.recv() => match event {
                Some(PtyEvent::Output(bytes)) => {
                    if write_frame(&mut send, &Frame::Data(bytes)).await.is_err() {
                        break None;
                    }
                }
                Some(PtyEvent::Shell { name, source, exit_code }) => {
                    let frame = Frame::ShellEvent(ShellEventPayload {
                        event: name.to_string(),
                        source: Some(source.to_string()),
                        // cwd comes with phase C's shell-integration work;
                        // see the plan's doc/code-mismatch note on osc.rs.
                        cwd: None,
                        exit_code,
                    });
                    if write_frame(&mut send, &frame).await.is_err() {
                        break None;
                    }
                }
                Some(PtyEvent::Exited) | None => {
                    // EOF on the master can land a few milliseconds before
                    // the child is reapable, so a single try_wait would
                    // sometimes report an unknown status for a shell that
                    // exited perfectly normally. Poll briefly for the real
                    // one. (Guard scoped per iteration - Send analysis.)
                    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(2);
                    let exit_code = loop {
                        let code = {
                            let mut table = table.lock().expect("session table poisoned");
                            table.get_mut(session_id).and_then(|handle| handle.pty.try_wait())
                        };
                        if code.is_some() || tokio::time::Instant::now() >= deadline {
                            break code;
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                    };
                    break Some(ClosePayload {
                        reason: Some("shell_exited".into()),
                        exit_code,
                    });
                }
            },

            read = recv.read(&mut buf) => match read {
                Ok(Some(n)) => {
                    decoder.push(&buf[..n]);
                    match drain_incoming_frames(&mut decoder, session_id, &table) {
                        Ok(true) => {}
                        // Controller asked to close, or sent garbage; either
                        // way the session is over and needs no close frame
                        // echoed back.
                        Ok(false) | Err(_) => break None,
                    }
                }
                // Peer finished or reset the stream: treat as close.
                Ok(None) | Err(_) => break None,
            },
        }
    };

    if let Some(payload) = close_reason {
        let _ = write_frame(&mut send, &Frame::Close(payload)).await;
    }
    let _ = send.finish();
    table.lock().expect("session table poisoned").close(session_id);
    tracing::info!(session = %session_id, "terminal session closed");
}

/// One directory-tree list+watch bi-stream, from just past the `fsList`
/// handshake (read by `serve_bi_stream`) onward: list once, then poll and
/// push `FsChanged` until the controller closes the stream (closing is how
/// the client "unwatches" - there is no separate unwatch frame).
async fn serve_fs_stream(
    first: FsListPayload,
    mut send: SendStream,
    mut recv: RecvStream,
    mut decoder: FrameDecoder,
) {
    let path = std::path::PathBuf::from(&first.path);

    let mut last_listing = match fs_browse::list_directory(&path) {
        Ok(entries) => {
            let frame = Frame::FsListResult(FsListResultPayload {
                entries: entries.clone(),
            });
            if write_frame(&mut send, &frame).await.is_err() {
                return;
            }
            entries
        }
        Err(e) => {
            let message = format!("FS_LIST_FAILED: {}", redact(&e.to_string()));
            send_error(&mut send, &message).await;
            return;
        }
    };

    let mut buf = [0u8; 256];
    loop {
        tokio::select! {
            _ = tokio::time::sleep(FS_WATCH_POLL_INTERVAL) => {
                let current = match fs_browse::list_directory(&path) {
                    Ok(entries) => entries,
                    // The directory disappearing mid-watch isn't a protocol
                    // error - just stop pushing updates, the same posture
                    // `LocalDirectoryTreeSource.watch` takes on an ENOENT.
                    Err(_) => break,
                };
                if current != last_listing {
                    last_listing = current;
                    let frame = Frame::FsChanged(FsChangedPayload { kind: "unknown".into() });
                    if write_frame(&mut send, &frame).await.is_err() {
                        break;
                    }
                }
            }
            read = recv.read(&mut buf) => match read {
                // The client has nothing legitimate to send after the
                // handshake; any bytes are ignored rather than parsed, and
                // only the stream actually ending stops the watch.
                Ok(Some(n)) => decoder.push(&buf[..n]),
                Ok(None) | Err(_) => break,
            },
        }
    }

    let _ = send.finish();
}

/// Fixed initial window (doc §8.6 default; `transfer::CREDIT_STEP` is the
/// top-up size once bytes land). Large enough that a typical note plus a
/// small attachment never has to wait for the first grant.
const INITIAL_TRANSFER_CREDIT: u64 = 4 * 1024 * 1024;

/// One note-transfer bi-stream, from just past the `transferManifest`
/// handshake (read by `serve_bi_stream`) onward: validate the manifest,
/// accept with an initial credit grant, then drive the existing
/// `transfer::TransferSession` off `transferChunk`/`transferFileEnd`/
/// `transferComplete` frames until a result is sent or the stream ends.
async fn serve_transfer_stream(
    manifest: TransferManifestPayload,
    mut send: SendStream,
    mut recv: RecvStream,
    mut decoder: FrameDecoder,
    table: Arc<Mutex<SessionTable>>,
    options: Arc<ServeOptions>,
) {
    let receive_root = resolve_transfer_root(
        manifest.target_path.as_deref(),
        manifest.session_id,
        &table,
        &options,
    );

    let entries: Vec<transfer::Entry> = manifest
        .entries
        .iter()
        .map(|e| transfer::Entry {
            index: e.index,
            relative_path: e.relative_path.clone(),
            size: e.size,
        })
        .collect();

    let mut session = match transfer::TransferSession::new(
        receive_root,
        entries,
        &manifest.root_note,
        INITIAL_TRANSFER_CREDIT,
    ) {
        Ok(session) => session,
        Err(e) => {
            let message = format!("TRANSFER_REJECTED: {}", redact(&e.to_string()));
            send_error(&mut send, &message).await;
            return;
        }
    };

    let accepted = Frame::TransferAccepted(TransferAcceptedPayload {
        granted_bytes: INITIAL_TRANSFER_CREDIT,
    });
    if write_frame(&mut send, &accepted).await.is_err() {
        session.abort();
        return;
    }

    let mut buf = vec![0u8; 32 * 1024];
    let result: Option<TransferResultPayload> = loop {
        let frame = match decoder.next_frame() {
            Ok(Some(frame)) => frame,
            Ok(None) => match recv.read(&mut buf).await {
                Ok(Some(n)) => {
                    decoder.push(&buf[..n]);
                    continue;
                }
                // Peer vanished mid-transfer: doc 4.8 accepts a partial file
                // being left behind, and no reply is possible anyway.
                Ok(None) | Err(_) => {
                    session.abort();
                    return;
                }
            },
            Err(e) => {
                session.abort();
                break Some(transfer_failed_result("PROTOCOL_ERROR", &e.to_string()));
            }
        };

        match frame {
            Frame::TransferChunk(chunk) => {
                match session.write_chunk(chunk.file_index, chunk.offset, &chunk.data) {
                    Ok(Some(granted_bytes)) => {
                        let credit = Frame::TransferCredit(TransferCreditPayload { granted_bytes });
                        if write_frame(&mut send, &credit).await.is_err() {
                            session.abort();
                            return;
                        }
                    }
                    Ok(None) => {}
                    Err(e) => {
                        session.abort();
                        break Some(transfer_failed_result("TRANSFER_FAILED", &e.to_string()));
                    }
                }
            }
            Frame::TransferFileEnd(end) => {
                if let Err(e) = session.finish_file(end.file_index, end.sent_size) {
                    session.abort();
                    break Some(transfer_failed_result("TRANSFER_FAILED", &e.to_string()));
                }
            }
            Frame::TransferComplete(_) => match session.complete() {
                Ok(()) => {
                    break Some(TransferResultPayload {
                        success: true,
                        code: None,
                        message: String::new(),
                    })
                }
                Err(e) => {
                    session.abort();
                    break Some(transfer_failed_result("TRANSFER_FAILED", &e.to_string()));
                }
            },
            other => {
                // Anything else is either a peer bug or a stray frame from a
                // different stream kind; tolerate it rather than aborting an
                // otherwise-healthy transfer.
                tracing::warn!(?other, "ignoring unexpected frame on a transfer stream");
            }
        }
    };

    if let Some(payload) = result {
        let _ = write_frame(&mut send, &Frame::TransferResult(payload)).await;
    }
    let _ = send.finish();
}

fn transfer_failed_result(code: &str, message: &str) -> TransferResultPayload {
    TransferResultPayload {
        success: false,
        code: Some(code.to_string()),
        message: redact(message),
    }
}

/// Resolution order: an explicit `targetPath` (directory-tree "drop onto
/// this node", candidate doc §4.1 point 4) wins outright - the user picked
/// a specific place, it is not a hint to be overridden. Otherwise doc
/// §7.6's existing order applies: a session with a known cwd wins over the
/// configured `receive_root`, the same fallback `serve_terminal_session`'s
/// `lastKnownCwd` tracking already exists to support.
fn resolve_transfer_root(
    target_path: Option<&str>,
    session_id: Option<Uuid>,
    table: &Arc<Mutex<SessionTable>>,
    options: &ServeOptions,
) -> std::path::PathBuf {
    if let Some(path) = target_path {
        return fs_browse::expand_user_path(Path::new(path));
    }
    if let Some(id) = session_id {
        let mut table = table.lock().expect("session table poisoned");
        if let Some(cwd) = table.get_mut(id).and_then(|handle| handle.last_known_cwd.clone()) {
            return cwd;
        }
    }
    options.receive_root.clone()
}

/// Matches `FILE_CHUNK_BYTES` in `src/services/remote/creditWindow.ts` -
/// nothing requires the two directions use the same chunk size, but there
/// is no reason for them not to.
const PULL_CHUNK_BYTES: usize = 256 * 1024;

/// One pull bi-stream, from just past the `transferPullRequest` handshake
/// (read by `serve_bi_stream`) onward (candidate doc phase 2B): walk the
/// requested path, send the manifest, wait for the client's initial credit
/// grant, then stream `TransferChunk`/`TransferFileEnd` per file -
/// respecting further `TransferCredit` grants that arrive mid-send, the
/// same flow-control contract `TransferSession` enforces in the other
/// direction - and finish with a `TransferResult`.
async fn serve_transfer_pull_stream(
    request: TransferPullRequestPayload,
    mut send: SendStream,
    mut recv: RecvStream,
    mut decoder: FrameDecoder,
) {
    let path = std::path::PathBuf::from(&request.path);
    let entries = match fs_browse::walk_for_pull(&path) {
        Ok(entries) => entries,
        Err(message) => {
            send_error(&mut send, &format!("PULL_FAILED: {}", redact(&message))).await;
            return;
        }
    };

    let manifest_entries: Vec<TransferEntry> = entries
        .iter()
        .map(|e| TransferEntry {
            index: e.index,
            relative_path: e.relative_path.clone(),
            size: e.size,
        })
        .collect();
    let manifest = Frame::TransferPullManifest(TransferPullManifestPayload {
        entries: manifest_entries,
    });
    if write_frame(&mut send, &manifest).await.is_err() {
        return;
    }

    let mut granted: u64 = match wait_for_credit(&mut recv, &mut decoder).await {
        Some(bytes) => bytes,
        None => return,
    };
    let mut sent: u64 = 0;

    for entry in &entries {
        let bytes = match std::fs::read(&entry.absolute_path) {
            Ok(bytes) => bytes,
            Err(e) => {
                let payload = TransferResultPayload {
                    success: false,
                    code: Some("PULL_FAILED".into()),
                    message: format!("PULL_FAILED: {}", redact(&e.to_string())),
                };
                let _ = write_frame(&mut send, &Frame::TransferResult(payload)).await;
                let _ = send.finish();
                return;
            }
        };

        let mut offset: usize = 0;
        while offset < bytes.len() {
            while sent >= granted {
                match wait_for_credit(&mut recv, &mut decoder).await {
                    Some(new_granted) => granted = new_granted,
                    None => return,
                }
            }
            let room = (granted - sent) as usize;
            let take = (bytes.len() - offset).min(PULL_CHUNK_BYTES).min(room);
            let chunk = Frame::TransferChunk(TransferChunkPayload {
                file_index: entry.index,
                offset: offset as u64,
                data: bytes[offset..offset + take].to_vec(),
            });
            if write_frame(&mut send, &chunk).await.is_err() {
                return;
            }
            sent += take as u64;
            offset += take;
        }

        let file_end = Frame::TransferFileEnd(TransferFileEndPayload {
            file_index: entry.index,
            sent_size: bytes.len() as u64,
        });
        if write_frame(&mut send, &file_end).await.is_err() {
            return;
        }
    }

    let result = Frame::TransferResult(TransferResultPayload {
        success: true,
        code: None,
        message: String::new(),
    });
    let _ = write_frame(&mut send, &result).await;
    let _ = send.finish();
}

/// Reads frames until a `TransferCredit` grant arrives, ignoring anything
/// else (there is nothing else the client legitimately sends on a pull
/// stream). `None` means the peer is gone or the stream is unusable.
async fn wait_for_credit(recv: &mut RecvStream, decoder: &mut FrameDecoder) -> Option<u64> {
    let mut buf = vec![0u8; 4096];
    loop {
        match decoder.next_frame() {
            Ok(Some(Frame::TransferCredit(payload))) => return Some(payload.granted_bytes),
            Ok(Some(_)) => continue,
            Ok(None) => {}
            Err(_) => return None,
        }
        match recv.read(&mut buf).await {
            Ok(Some(n)) => decoder.push(&buf[..n]),
            Ok(None) | Err(_) => return None,
        }
    }
}

/// Applies every complete frame sitting in the decoder. `Ok(true)` keeps the
/// session running; `Ok(false)` means the controller sent `close`.
fn drain_incoming_frames(
    decoder: &mut FrameDecoder,
    session_id: Uuid,
    table: &Arc<Mutex<SessionTable>>,
) -> Result<bool, AgentError> {
    while let Some(frame) = decoder.next_frame()? {
        match frame {
            Frame::Data(bytes) => {
                let mut table = table.lock().expect("session table poisoned");
                if let Some(handle) = table.get_mut(session_id) {
                    handle.pty.write_input(&bytes)?;
                }
            }
            Frame::Resize(payload) => {
                let mut table = table.lock().expect("session table poisoned");
                if let Some(handle) = table.get_mut(session_id) {
                    let _ = handle.pty.resize(payload.cols, payload.rows);
                }
            }
            Frame::Close(_) => return Ok(false),
            other => {
                // open twice, or frames only the agent may send - a protocol
                // violation, but killing the session over it helps nobody.
                tracing::warn!(session = %session_id, "ignoring unexpected frame: {other:?}");
            }
        }
    }
    Ok(true)
}

async fn read_frame(
    recv: &mut RecvStream,
    decoder: &mut FrameDecoder,
) -> Result<Option<Frame>, AgentError> {
    let mut buf = [0u8; 4096];
    loop {
        if let Some(frame) = decoder.next_frame()? {
            return Ok(Some(frame));
        }
        match recv
            .read(&mut buf)
            .await
            .map_err(|e| AgentError::Protocol(e.to_string()))?
        {
            Some(n) => decoder.push(&buf[..n]),
            None => return Ok(None),
        }
    }
}

async fn write_frame(send: &mut SendStream, frame: &Frame) -> Result<(), AgentError> {
    let bytes = frame.encode()?;
    send.write_all(&bytes)
        .await
        .map_err(|e| AgentError::Protocol(e.to_string()))
}

async fn send_error(send: &mut SendStream, message: &str) {
    let frame = Frame::Error(ErrorPayload {
        message: message.to_string(),
    });
    let _ = write_frame(send, &frame).await;
    let _ = send.finish();
}

/// Doc 8.8.6 and 13.2: error text crossing the wire must not leak local
/// paths or anything else about the host. (Inherited verbatim from the
/// retired V1 relay client.)
fn redact(message: &str) -> String {
    let mut out: String = message
        .split_whitespace()
        .map(|word| {
            if word.contains('/') || word.contains('\\') {
                "<path>".to_string()
            } else {
                word.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    out.truncate(512);
    out
}

/// Same shape as V1's `client::spawn_output_pump`: a dedicated blocking
/// thread owns the PTY reader, OSC-scans the bytes for shell events and
/// forwards everything over the channel. EOF means the shell died.
fn spawn_output_pump(mut reader: Box<dyn std::io::Read + Send>, tx: mpsc::Sender<PtyEvent>) {
    std::thread::spawn(move || {
        let mut scanner = crate::osc::OscScanner::new();
        let mut buf = vec![0u8; 32 * 1024];

        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    for event in scanner.scan(&buf[..n]) {
                        let _ = tx.blocking_send(PtyEvent::Shell {
                            name: event.event_name(),
                            source: event.source_name(),
                            exit_code: event.exit_code(),
                        });
                    }
                    if tx.blocking_send(PtyEvent::Output(buf[..n].to_vec())).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }

        let _ = tx.blocking_send(PtyEvent::Exited);
    });
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::identity::DeviceIdentity;
    use crate::p2p::{bind_endpoint, connection_code, loopback_addr, parse_connection_code, EndpointProfile};
    use crate::termstream::{
        FsEntry, FsListPayload, OpenPayload, ResizePayload, TransferChunkPayload,
        TransferCompletePayload, TransferEntry, TransferFileEndPayload,
    };
    use std::time::Duration;

    fn test_shell() -> ShellConfig {
        ShellConfig {
            program: "/bin/sh".into(),
            args: vec![],
        }
    }

    #[test]
    fn redaction_strips_paths() {
        // Whole whitespace-separated words are replaced, punctuation and all.
        assert_eq!(
            redact("cannot start /usr/local/bin/fish: no such file"),
            "cannot start <path> no such file"
        );
        assert_eq!(redact(r"error at C:\Users\a\shell.exe here"), "error at <path> here");
    }

    #[test]
    fn redaction_bounds_length() {
        let long = "x".repeat(2000);
        assert!(redact(&long).len() <= 512);
    }

    fn test_identity() -> DeviceIdentity {
        let dir = tempfile::tempdir().unwrap();
        DeviceIdentity::load_or_create(&dir.path().join("identity.json")).unwrap()
    }

    /// Caps every await so a bug wedges one test, not the whole suite.
    async fn within<T>(fut: impl std::future::Future<Output = T>) -> T {
        tokio::time::timeout(Duration::from_secs(20), fut)
            .await
            .expect("test network operation timed out")
    }

    struct TestStream {
        send: SendStream,
        recv: RecvStream,
        decoder: FrameDecoder,
        buf: Vec<u8>,
    }

    impl TestStream {
        fn new(send: SendStream, recv: RecvStream) -> Self {
            Self {
                send,
                recv,
                decoder: FrameDecoder::new(),
                buf: vec![0u8; 32 * 1024],
            }
        }

        async fn send_frame(&mut self, frame: Frame) {
            self.send.write_all(&frame.encode().unwrap()).await.unwrap();
        }

        async fn next_frame(&mut self) -> Frame {
            within(async {
                loop {
                    if let Some(frame) = self.decoder.next_frame().unwrap() {
                        return frame;
                    }
                    let n = self
                        .recv
                        .read(&mut self.buf)
                        .await
                        .unwrap()
                        .expect("agent ended the stream unexpectedly");
                    self.decoder.push(&self.buf[..n]);
                }
            })
            .await
        }

        /// Reads Data frames until `marker` shows up in the accumulated
        /// output, returning everything seen. Panics via timeout if the
        /// marker never arrives.
        async fn read_output_until(&mut self, marker: &str) -> String {
            let mut seen = String::new();
            within(async {
                loop {
                    match self.next_frame_inner().await {
                        Frame::Data(bytes) => {
                            seen.push_str(&String::from_utf8_lossy(&bytes));
                            if seen.contains(marker) {
                                return;
                            }
                        }
                        Frame::ShellEvent(_) => {}
                        other => panic!("unexpected frame while waiting for output: {other:?}"),
                    }
                }
            })
            .await;
            seen
        }

        /// `next_frame` without the outer timeout, for callers that already
        /// hold one.
        async fn next_frame_inner(&mut self) -> Frame {
            loop {
                if let Some(frame) = self.decoder.next_frame().unwrap() {
                    return frame;
                }
                let n = self
                    .recv
                    .read(&mut self.buf)
                    .await
                    .unwrap()
                    .expect("agent ended the stream unexpectedly");
                self.decoder.push(&self.buf[..n]);
            }
        }

        /// Waits for the Close frame, skipping remaining output.
        async fn expect_close(&mut self) -> ClosePayload {
            within(async {
                loop {
                    match self.next_frame_inner().await {
                        Frame::Close(payload) => return payload,
                        Frame::Data(_) | Frame::ShellEvent(_) => {}
                        other => panic!("expected close, got {other:?}"),
                    }
                }
            })
            .await
        }
    }

    /// The returned `TempDir` must be kept alive (bound to a variable, not
    /// `_`) for as long as the agent might still resolve `receive_root` -
    /// dropping it deletes the directory.
    async fn start_agent(
        max_sessions: usize,
    ) -> (Endpoint, String, tokio::task::JoinHandle<()>, tempfile::TempDir) {
        let identity = test_identity();
        let endpoint = bind_endpoint(&identity, EndpointProfile::Loopback)
            .await
            .unwrap();
        let code = connection_code(loopback_addr(&endpoint));
        let receive_root_dir = tempfile::tempdir().unwrap();
        let serve_task = tokio::spawn(serve(
            endpoint.clone(),
            ServeOptions {
                shell: test_shell(),
                max_concurrent_sessions: max_sessions,
                receive_root: receive_root_dir.path().to_path_buf(),
            },
        ));
        (endpoint, code, serve_task, receive_root_dir)
    }

    async fn connect_controller(code: &str) -> (Endpoint, Connection) {
        let controller = bind_endpoint(&test_identity(), EndpointProfile::Loopback)
            .await
            .unwrap();
        let addr = parse_connection_code(code).unwrap();
        let connection = within(controller.connect(addr, ALPN_TERMINAL)).await.unwrap();
        (controller, connection)
    }

    async fn open_session(connection: &Connection) -> (TestStream, OpenedPayload) {
        let (send, recv) = within(connection.open_bi()).await.unwrap();
        let mut stream = TestStream::new(send, recv);
        stream
            .send_frame(Frame::Open(OpenPayload { cols: 80, rows: 24 }))
            .await;
        match stream.next_frame().await {
            Frame::Opened(payload) => (stream, payload),
            other => panic!("expected opened, got {other:?}"),
        }
    }

    /// Opens an fs-list-and-watch bi-stream (the `FsList`-first counterpart
    /// to `open_session`'s `Open`-first handshake) and sends the request.
    async fn open_fs_stream(connection: &Connection, path: &str) -> TestStream {
        let (send, recv) = within(connection.open_bi()).await.unwrap();
        let mut stream = TestStream::new(send, recv);
        stream
            .send_frame(Frame::FsList(FsListPayload { path: path.to_string() }))
            .await;
        stream
    }

    #[test]
    fn fs_list_and_watch_work_over_loopback_quic() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let (agent, code, serve_task, _receive_root) = start_agent(4).await;
            let (controller, connection) = connect_controller(&code).await;

            let dir = tempfile::tempdir().unwrap();
            std::fs::write(dir.path().join("a.txt"), "").unwrap();

            let mut stream = open_fs_stream(&connection, dir.path().to_str().unwrap()).await;
            match stream.next_frame().await {
                Frame::FsListResult(payload) => {
                    assert_eq!(
                        payload.entries,
                        vec![FsEntry {
                            name: "a.txt".into(),
                            is_directory: false,
                        }]
                    );
                }
                other => panic!("expected FsListResult, got {other:?}"),
            }

            // A change under the watched directory must surface as FsChanged
            // within a couple of poll cycles - no explicit "watch" frame is
            // needed, the still-open stream itself is the subscription.
            std::fs::write(dir.path().join("b.txt"), "").unwrap();
            match stream.next_frame().await {
                Frame::FsChanged(payload) => assert_eq!(payload.kind, "unknown"),
                other => panic!("expected FsChanged, got {other:?}"),
            }

            // Closing the stream is how the client "unwatches"; the terminal
            // connection and gate are unaffected either way.
            drop(stream);

            // The same connection can still open an ordinary terminal
            // session afterwards - fs streams and terminal streams coexist.
            let (mut terminal, _) = open_session(&connection).await;
            terminal
                .send_frame(Frame::Data(b"echo fs-coexist-marker\n".to_vec()))
                .await;
            let seen = terminal.read_output_until("fs-coexist-marker").await;
            assert!(seen.contains("fs-coexist-marker"));

            connection.close(VarInt::from_u32(0), b"done");
            controller.close().await;
            agent.close().await;
            within(serve_task).await.unwrap();
        });
    }

    #[test]
    fn fs_list_on_a_missing_path_reports_fs_list_failed() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let (agent, code, serve_task, _receive_root) = start_agent(4).await;
            let (controller, connection) = connect_controller(&code).await;

            let dir = tempfile::tempdir().unwrap();
            let missing = dir.path().join("does-not-exist");

            let mut stream = open_fs_stream(&connection, missing.to_str().unwrap()).await;
            match stream.next_frame().await {
                Frame::Error(payload) => {
                    assert!(payload.message.starts_with("FS_LIST_FAILED"), "got: {}", payload.message);
                }
                other => panic!("expected error, got {other:?}"),
            }

            connection.close(VarInt::from_u32(0), b"done");
            controller.close().await;
            agent.close().await;
            within(serve_task).await.unwrap();
        });
    }

    /// Opens a note-transfer bi-stream (the `TransferManifest`-first
    /// counterpart to `open_session`'s `Open`-first handshake) and sends
    /// the manifest.
    async fn open_transfer_stream(
        connection: &Connection,
        manifest: TransferManifestPayload,
    ) -> TestStream {
        let (send, recv) = within(connection.open_bi()).await.unwrap();
        let mut stream = TestStream::new(send, recv);
        stream.send_frame(Frame::TransferManifest(manifest)).await;
        stream
    }

    fn manifest(
        transfer_id: &str,
        entries: Vec<TransferEntry>,
        session_id: Option<Uuid>,
    ) -> TransferManifestPayload {
        TransferManifestPayload {
            transfer_id: transfer_id.into(),
            root_note: entries[0].relative_path.clone(),
            entries,
            session_id,
            target_path: None,
        }
    }

    /// Opens a pull bi-stream (the `TransferPullRequest`-first counterpart
    /// to `open_transfer_stream`'s `TransferManifest`-first handshake).
    async fn open_pull_stream(connection: &Connection, path: &str) -> TestStream {
        let (send, recv) = within(connection.open_bi()).await.unwrap();
        let mut stream = TestStream::new(send, recv);
        stream
            .send_frame(Frame::TransferPullRequest(TransferPullRequestPayload {
                path: path.to_string(),
            }))
            .await;
        stream
    }

    #[test]
    fn a_full_pull_sends_manifest_chunks_and_a_success_result() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let (agent, code, serve_task, _receive_root) = start_agent(4).await;
            let (controller, connection) = connect_controller(&code).await;

            let source_dir = tempfile::tempdir().unwrap();
            std::fs::write(source_dir.path().join("demo.md"), b"hello world").unwrap();

            let mut stream =
                open_pull_stream(&connection, source_dir.path().to_str().unwrap()).await;

            match stream.next_frame().await {
                Frame::TransferPullManifest(payload) => {
                    assert_eq!(payload.entries.len(), 1);
                    assert_eq!(payload.entries[0].size, 11);
                }
                other => panic!("expected transferPullManifest, got {other:?}"),
            }

            stream
                .send_frame(Frame::TransferCredit(TransferCreditPayload {
                    granted_bytes: 4 * 1024 * 1024,
                }))
                .await;

            let chunk = stream.next_frame().await;
            let Frame::TransferChunk(chunk_payload) = chunk else {
                panic!("expected transferChunk, got {chunk:?}");
            };
            assert_eq!(chunk_payload.data, b"hello world");

            match stream.next_frame().await {
                Frame::TransferFileEnd(payload) => assert_eq!(payload.sent_size, 11),
                other => panic!("expected transferFileEnd, got {other:?}"),
            }
            match stream.next_frame().await {
                Frame::TransferResult(payload) => assert!(payload.success, "got: {payload:?}"),
                other => panic!("expected transferResult, got {other:?}"),
            }

            connection.close(VarInt::from_u32(0), b"done");
            controller.close().await;
            agent.close().await;
            within(serve_task).await.unwrap();
        });
    }

    #[test]
    fn a_pull_respects_a_tight_credit_window_across_multiple_grants() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let (agent, code, serve_task, _receive_root) = start_agent(4).await;
            let (controller, connection) = connect_controller(&code).await;

            let source_dir = tempfile::tempdir().unwrap();
            let big = vec![7u8; 300 * 1024]; // > PULL_CHUNK_BYTES, forces multiple chunks
            std::fs::write(source_dir.path().join("big.bin"), &big).unwrap();

            let mut stream =
                open_pull_stream(&connection, source_dir.path().to_str().unwrap()).await;
            match stream.next_frame().await {
                Frame::TransferPullManifest(payload) => {
                    assert_eq!(payload.entries[0].size, big.len() as u64)
                }
                other => panic!("expected transferPullManifest, got {other:?}"),
            }

            // Grant less than the whole file so a second grant is required.
            stream
                .send_frame(Frame::TransferCredit(TransferCreditPayload {
                    granted_bytes: 100 * 1024,
                }))
                .await;

            let mut received = Vec::new();
            loop {
                match stream.next_frame().await {
                    Frame::TransferChunk(payload) => {
                        received.extend_from_slice(&payload.data);
                        if received.len() < big.len() {
                            // Keep the window moving so the agent does not stall waiting forever.
                            stream
                                .send_frame(Frame::TransferCredit(TransferCreditPayload {
                                    granted_bytes: big.len() as u64,
                                }))
                                .await;
                        }
                    }
                    Frame::TransferFileEnd(payload) => {
                        assert_eq!(payload.sent_size, big.len() as u64);
                        break;
                    }
                    other => panic!("unexpected frame while collecting chunks: {other:?}"),
                }
            }
            assert_eq!(received, big, "every byte must arrive, in order");

            match stream.next_frame().await {
                Frame::TransferResult(payload) => assert!(payload.success),
                other => panic!("expected transferResult, got {other:?}"),
            }

            connection.close(VarInt::from_u32(0), b"done");
            controller.close().await;
            agent.close().await;
            within(serve_task).await.unwrap();
        });
    }

    #[test]
    fn a_pull_of_a_missing_path_reports_pull_failed() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let (agent, code, serve_task, _receive_root) = start_agent(4).await;
            let (controller, connection) = connect_controller(&code).await;

            let dir = tempfile::tempdir().unwrap();
            let missing = dir.path().join("does-not-exist");
            let mut stream = open_pull_stream(&connection, missing.to_str().unwrap()).await;

            match stream.next_frame().await {
                Frame::Error(payload) => {
                    assert!(payload.message.starts_with("PULL_FAILED"), "got: {}", payload.message);
                }
                other => panic!("expected error, got {other:?}"),
            }

            connection.close(VarInt::from_u32(0), b"done");
            controller.close().await;
            agent.close().await;
            within(serve_task).await.unwrap();
        });
    }

    #[test]
    fn a_full_transfer_writes_the_files_and_reports_success() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let (agent, code, serve_task, receive_root) = start_agent(4).await;
            let (controller, connection) = connect_controller(&code).await;

            let entries = vec![
                TransferEntry {
                    index: 0,
                    relative_path: "notes/demo.md".into(),
                    size: 11,
                },
                TransferEntry {
                    index: 1,
                    relative_path: "assets/img.png".into(),
                    size: 4,
                },
            ];
            let mut stream = open_transfer_stream(&connection, manifest("t1", entries, None)).await;

            match stream.next_frame().await {
                Frame::TransferAccepted(payload) => assert!(payload.granted_bytes > 0),
                other => panic!("expected accepted, got {other:?}"),
            }

            stream
                .send_frame(Frame::TransferChunk(TransferChunkPayload {
                    file_index: 0,
                    offset: 0,
                    data: b"hello world".to_vec(),
                }))
                .await;
            stream
                .send_frame(Frame::TransferFileEnd(TransferFileEndPayload {
                    file_index: 0,
                    sent_size: 11,
                }))
                .await;
            stream
                .send_frame(Frame::TransferChunk(TransferChunkPayload {
                    file_index: 1,
                    offset: 0,
                    data: b"\x89PNG".to_vec(),
                }))
                .await;
            stream
                .send_frame(Frame::TransferFileEnd(TransferFileEndPayload {
                    file_index: 1,
                    sent_size: 4,
                }))
                .await;
            stream
                .send_frame(Frame::TransferComplete(TransferCompletePayload {}))
                .await;

            match stream.next_frame().await {
                Frame::TransferResult(payload) => assert!(payload.success, "got: {payload:?}"),
                other => panic!("expected result, got {other:?}"),
            }

            assert_eq!(
                std::fs::read(receive_root.path().join("notes/demo.md")).unwrap(),
                b"hello world",
            );
            assert_eq!(
                std::fs::read(receive_root.path().join("assets/img.png")).unwrap(),
                b"\x89PNG",
            );

            connection.close(VarInt::from_u32(0), b"done");
            controller.close().await;
            agent.close().await;
            within(serve_task).await.unwrap();
        });
    }

    #[test]
    fn a_manifest_with_a_traversal_path_is_rejected_before_anything_is_written() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let (agent, code, serve_task, receive_root) = start_agent(4).await;
            let (controller, connection) = connect_controller(&code).await;

            let entries = vec![TransferEntry {
                index: 0,
                relative_path: "../escape.md".into(),
                size: 1,
            }];
            let mut stream = open_transfer_stream(&connection, manifest("t2", entries, None)).await;

            match stream.next_frame().await {
                Frame::Error(payload) => {
                    assert!(
                        payload.message.starts_with("TRANSFER_REJECTED"),
                        "got: {}",
                        payload.message
                    );
                }
                other => panic!("expected error, got {other:?}"),
            }
            assert_eq!(std::fs::read_dir(receive_root.path()).unwrap().count(), 0);

            connection.close(VarInt::from_u32(0), b"done");
            controller.close().await;
            agent.close().await;
            within(serve_task).await.unwrap();
        });
    }

    /// `resolve_transfer_root` itself, not a QUIC round trip: driving a real
    /// session's `lastKnownCwd` through the wire would need OSC shell
    /// integration the test shell does not emit, so this exercises the
    /// function directly against a table holding a real (but otherwise
    /// idle) session - the same thing `serve_transfer_stream` does with a
    /// `sessionId`-bearing manifest.
    #[test]
    fn resolve_transfer_root_prefers_a_known_sessions_cwd_over_receive_root() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let (pty, _reader) =
                PtySession::spawn(&test_shell().program, &test_shell().args, None, 80, 24).unwrap();
            let session_id = Uuid::new_v4();
            let mut raw_table = SessionTable::new(4);
            raw_table.open(session_id, pty).unwrap();
            let cwd_dir = tempfile::tempdir().unwrap();
            raw_table.set_cwd(session_id, cwd_dir.path().to_path_buf());
            let table = Arc::new(Mutex::new(raw_table));

            let fallback_root = tempfile::tempdir().unwrap();
            let options = ServeOptions {
                shell: test_shell(),
                max_concurrent_sessions: 4,
                receive_root: fallback_root.path().to_path_buf(),
            };

            assert_eq!(
                resolve_transfer_root(None, Some(session_id), &table, &options),
                cwd_dir.path()
            );
            assert_eq!(
                resolve_transfer_root(None, Some(Uuid::new_v4()), &table, &options),
                fallback_root.path(),
                "an unknown sessionId falls back to receive_root"
            );
            assert_eq!(
                resolve_transfer_root(None, None, &table, &options),
                fallback_root.path(),
                "no sessionId at all falls back to receive_root"
            );

            let explicit = tempfile::tempdir().unwrap();
            assert_eq!(
                resolve_transfer_root(
                    Some(explicit.path().to_str().unwrap()),
                    Some(session_id),
                    &table,
                    &options
                ),
                explicit.path(),
                "an explicit targetPath wins even over a session with a known cwd"
            );

            table.lock().unwrap().close(session_id);
        });
    }

    #[test]
    fn a_full_terminal_session_works_over_loopback_quic() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let (agent, code, serve_task, _receive_root) = start_agent(4).await;
            let (controller, connection) = connect_controller(&code).await;

            let (mut stream, opened) = open_session(&connection).await;
            assert_eq!(opened.shell, "/bin/sh");

            // Real shell echo round-trip.
            stream
                .send_frame(Frame::Data(b"echo serve-e2e-marker\n".to_vec()))
                .await;
            let seen = stream.read_output_until("serve-e2e-marker").await;
            assert!(seen.contains("serve-e2e-marker"), "shell output must flow back");

            // Resize is observable from inside the session.
            stream
                .send_frame(Frame::Resize(ResizePayload { cols: 120, rows: 40 }))
                .await;
            stream.send_frame(Frame::Data(b"stty size\n".to_vec())).await;
            let seen = stream.read_output_until("40 120").await;
            assert!(seen.contains("40 120"), "stty must report the resized dimensions");

            // A second concurrent session on the same connection (doc 4
            // decision 4), fully independent.
            let (mut second, second_opened) = open_session(&connection).await;
            assert_ne!(second_opened.session_id, opened.session_id);
            second
                .send_frame(Frame::Data(b"echo second-session-marker\n".to_vec()))
                .await;
            second.read_output_until("second-session-marker").await;

            // Typing exit ends the shell and the agent reports shell_exited
            // with the real exit status.
            second.send_frame(Frame::Data(b"exit 7\n".to_vec())).await;
            let close = second.expect_close().await;
            assert_eq!(close.reason.as_deref(), Some("shell_exited"));
            assert_eq!(close.exit_code, Some(7));

            connection.close(VarInt::from_u32(0), b"done");
            controller.close().await;
            agent.close().await;
            within(serve_task).await.unwrap();
        });
    }

    #[test]
    fn the_session_cap_refuses_with_session_limit_reached() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let (agent, code, serve_task, _receive_root) = start_agent(1).await;
            let (controller, connection) = connect_controller(&code).await;

            let (_stream, _) = open_session(&connection).await;

            let (send, recv) = within(connection.open_bi()).await.unwrap();
            let mut refused = TestStream::new(send, recv);
            refused
                .send_frame(Frame::Open(OpenPayload { cols: 80, rows: 24 }))
                .await;
            match refused.next_frame().await {
                Frame::Error(payload) => assert!(
                    payload.message.starts_with("SESSION_LIMIT_REACHED"),
                    "got: {}",
                    payload.message
                ),
                other => panic!("expected error, got {other:?}"),
            }

            connection.close(VarInt::from_u32(0), b"done");
            controller.close().await;
            agent.close().await;
            within(serve_task).await.unwrap();
        });
    }

    #[test]
    fn dropping_the_connection_kills_every_session_process_tree() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let (agent, code, serve_task, _receive_root) = start_agent(4).await;
            let (controller, connection) = connect_controller(&code).await;

            let (mut stream, _) = open_session(&connection).await;

            // A grandchild that must not survive the disconnect. Parse the
            // pid from any occurrence followed by digits - the pty echoes
            // the input line (with "$!") back first.
            stream
                .send_frame(Frame::Data(b"sleep 300 & echo GRANDCHILD=$!\n".to_vec()))
                .await;
            let seen = stream.read_output_until("GRANDCHILD=").await;
            let pid = within(async {
                let mut seen = seen;
                loop {
                    if let Some(pid) = seen
                        .split("GRANDCHILD=")
                        .skip(1)
                        .find_map(|rest| {
                            let digits: String =
                                rest.chars().take_while(char::is_ascii_digit).collect();
                            digits.parse::<i32>().ok()
                        })
                    {
                        return pid;
                    }
                    match stream.next_frame_inner().await {
                        Frame::Data(bytes) => seen.push_str(&String::from_utf8_lossy(&bytes)),
                        _ => {}
                    }
                }
            })
            .await;
            assert!(
                std::path::Path::new(&format!("/proc/{pid}")).exists(),
                "the grandchild should be alive before the disconnect"
            );

            // Simulate the controller vanishing (doc 7.5: sessions die with
            // the connection).
            connection.close(VarInt::from_u32(0), b"gone");

            let deadline = std::time::Instant::now() + Duration::from_secs(15);
            while std::time::Instant::now() < deadline
                && std::path::Path::new(&format!("/proc/{pid}")).exists()
            {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            assert!(
                !std::path::Path::new(&format!("/proc/{pid}")).exists(),
                "sleep {pid} survived the controller disconnect"
            );

            controller.close().await;
            agent.close().await;
            within(serve_task).await.unwrap();
        });
    }
}
