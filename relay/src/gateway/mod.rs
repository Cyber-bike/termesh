//! WSS gateway (doc 8.2, 8.6, 8.7, 9, 11.2).

pub mod agent;
pub mod control;
pub mod registry;

use std::time::Duration;

use axum::extract::ws::{CloseFrame, Message, WebSocket};
use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use tokio::sync::mpsc::Receiver;

use registry::Outbound;

pub const SUBPROTOCOL: &str = "termy.v1";
pub const HEARTBEAT_INTERVAL_MS: u64 = 20_000;
/// Doc 8.6: no heartbeat or pong within this window means offline.
pub const AGENT_OFFLINE_TIMEOUT: Duration = Duration::from_secs(50);
/// Doc 8.3: a control frame may not exceed 64 KiB.
pub const MAX_CONTROL_FRAME_BYTES: usize = 65_536;
/// Doc 8.6: drain at most this much terminal data before letting one file chunk through.
pub const TERMINAL_DRAIN_BUDGET: usize = 64 * 1024;

/// Doc 8.7.
pub mod close {
    pub const NORMAL: u16 = 1000;
    pub const GOING_AWAY: u16 = 1001;
    pub const PROTOCOL: u16 = 4400;
    pub const UNAUTHORIZED: u16 = 4401;
    pub const FORBIDDEN: u16 = 4403;
    pub const TIMEOUT: u16 = 4408;
    pub const CONFLICT: u16 = 4409;
    pub const TOO_LARGE: u16 = 4413;
    pub const INTERNAL: u16 = 4500;
}

/// Drains the two outbound lanes onto one socket with terminal priority.
///
/// Doc 8.6 fixes the policy: empty up to 64 KiB of control/terminal traffic,
/// then let exactly one file chunk through. A plain biased select would starve
/// the file lane whenever a terminal is chatty, so the budget is counted
/// explicitly and the loop always gives the file lane its turn.
pub async fn writer_task(
    mut sink: SplitSink<WebSocket, Message>,
    mut control_rx: Receiver<Outbound>,
    mut file_rx: Receiver<Outbound>,
) {
    use tokio::sync::mpsc::error::TryRecvError;

    loop {
        let mut sent_any = false;
        let mut drained = 0usize;

        while drained < TERMINAL_DRAIN_BUDGET {
            match control_rx.try_recv() {
                Ok(msg) => {
                    drained += msg.byte_len();
                    sent_any = true;
                    if !send(&mut sink, msg).await {
                        return;
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return,
            }
        }

        match file_rx.try_recv() {
            Ok(msg) => {
                sent_any = true;
                if !send(&mut sink, msg).await {
                    return;
                }
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => return,
        }

        if sent_any {
            continue;
        }

        // Both lanes are empty; wait for whichever wakes first, still preferring
        // control so a terminal keystroke does not queue behind a chunk.
        tokio::select! {
            biased;
            msg = control_rx.recv() => match msg {
                Some(msg) => if !send(&mut sink, msg).await { return },
                None => return,
            },
            msg = file_rx.recv() => match msg {
                Some(msg) => if !send(&mut sink, msg).await { return },
                None => return,
            },
        }
    }
}

/// Returns false once the socket is finished, either because it errored or
/// because a Close was just written.
async fn send(sink: &mut SplitSink<WebSocket, Message>, msg: Outbound) -> bool {
    let (message, terminal) = match msg {
        Outbound::Text(s) => (Message::Text(s.into()), false),
        Outbound::Binary(b) => (Message::Binary(b.into()), false),
        Outbound::Ping => (Message::Ping(Vec::new().into()), false),
        Outbound::Close(code, reason) => (
            Message::Close(Some(CloseFrame { code, reason: reason.into() })),
            true,
        ),
    };

    if sink.send(message).await.is_err() {
        return false;
    }
    !terminal
}

/// Close reasons must be a stable error code only: doc 8.7 caps them at 123
/// bytes and forbids tokens, user input or payload bytes.
pub fn close_reason(code: &str) -> String {
    let mut reason = code.to_string();
    reason.truncate(123);
    reason
}
