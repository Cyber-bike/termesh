//! iroh endpoint wiring (v2.0 doc 6.1/7.7, phase A).
//!
//! The agent side of the P2P transport is deliberately NOT gated on the A0
//! spike - A0 only concerns loading the JS binding inside Electron. Here the
//! real `iroh` crate is used directly, and everything in this module is
//! exercised over genuine loopback QUIC connections in the tests below.
//!
//! Three responsibilities, nothing more:
//! - build an `Endpoint` from the persisted [`DeviceIdentity`] (same Ed25519
//!   seed, so the `EndpointId` is stable across restarts),
//! - turn the endpoint's address into the connection code (an
//!   `EndpointTicket` string, doc 5.1) and parse one back,
//! - enforce doc 7.7's single-controller rule via [`ControllerGate`].
//!
//! The accept loop that stitches these into `termy-agent run` (spawning PTY
//! sessions per `termy/terminal/1` stream) is the next slice; keeping it out
//! of this module keeps every piece here testable in isolation.

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};

use iroh::endpoint::{presets, Connection, VarInt};
use iroh::{Endpoint, EndpointAddr, RelayMode, TransportAddr};
use iroh_tickets::endpoint::EndpointTicket;

use crate::identity::DeviceIdentity;
use crate::AgentError;

/// ALPN identifiers (doc 8.1). One connection negotiates exactly one ALPN,
/// so a transfer needs its own connection - see the plan's risk registry
/// entry on ALPN-per-connection.
pub const ALPN_TERMINAL: &[u8] = b"termy/terminal/1";
pub const ALPN_TRANSFER: &[u8] = b"termy/transfer/1";

/// Application close code + reason for doc 7.7's "second controller refused".
/// The reason string is what the control end matches on (doc 13's error
/// table); the numeric code just has to be stable and non-zero.
pub const CLOSE_CODE_CONTROLLER_ALREADY_CONNECTED: u32 = 0x01;
pub const CLOSE_REASON_CONTROLLER_ALREADY_CONNECTED: &[u8] = b"CONTROLLER_ALREADY_CONNECTED";

/// How the endpoint should reach the outside world.
pub enum EndpointProfile {
    /// n0 defaults: production relays + DNS/Pkarr address publishing, so a
    /// controller can find this agent after the machine changes networks
    /// (doc 4 decision 11, doc 7.7).
    Production,
    /// Loopback only, relays and address lookup disabled. For tests and
    /// local development: no packet leaves 127.0.0.1 and nothing is
    /// published to the discovery network.
    Loopback,
}

/// Binds an iroh endpoint whose `EndpointId` is this device's identity.
pub async fn bind_endpoint(
    identity: &DeviceIdentity,
    profile: EndpointProfile,
) -> Result<Endpoint, AgentError> {
    let secret = iroh::SecretKey::from_bytes(&identity.seed_bytes());

    let builder = match profile {
        EndpointProfile::Production => Endpoint::builder(presets::N0),
        EndpointProfile::Loopback => Endpoint::builder(presets::Minimal)
            .relay_mode(RelayMode::Disabled)
            .bind_addr(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
            .map_err(|e| AgentError::Config(format!("cannot bind to loopback: {e}")))?,
    };

    builder
        .secret_key(secret)
        .alpns(vec![ALPN_TERMINAL.to_vec(), ALPN_TRANSFER.to_vec()])
        .bind()
        .await
        .map_err(|e| AgentError::Config(format!("cannot bind the iroh endpoint: {e}")))
}

/// The address a peer on the same host can dial right now. `Endpoint::addr()`
/// is filled in asynchronously by watchers (and, in production, by the relay
/// handshake); the bound sockets are known synchronously and are exactly
/// what's dialable in the loopback profile.
pub fn loopback_addr(endpoint: &Endpoint) -> EndpointAddr {
    EndpointAddr::from_parts(
        endpoint.id(),
        endpoint.bound_sockets().into_iter().map(TransportAddr::Ip),
    )
}

/// The connection code the user copies (doc 5.1): the `EndpointTicket`
/// string form of the given address.
pub fn connection_code(addr: EndpointAddr) -> String {
    EndpointTicket::from(addr).to_string()
}

/// Parses a pasted connection code back into a dialable address. The TS side
/// has its own pre-validation (`connectionCode.ts`); this is the agent-side
/// equivalent of the authoritative parse, used by tests and any future CLI
/// diagnostics.
pub fn parse_connection_code(code: &str) -> Result<EndpointAddr, AgentError> {
    let ticket: EndpointTicket = code
        .trim()
        .parse()
        .map_err(|e| AgentError::Protocol(format!("TICKET_INVALID: {e}")))?;
    Ok(ticket.endpoint_addr().clone())
}

#[derive(Default)]
struct GateState {
    active: bool,
    generation: u64,
}

/// Doc 7.7: the agent serves one control end at a time. The first connection
/// claims the gate and holds it for its whole life; while held, every later
/// connection is refused with `CONTROLLER_ALREADY_CONNECTED`. Dropping the
/// [`ControllerSlot`] (the connection handler ending, however it ends)
/// reopens the gate.
///
/// This is the v2.0 analogue of V1's `pty::SessionSlot`, one level up: V1
/// admitted one *session*, v2.0 admits one *controller* which may hold many
/// sessions (doc 4 decision 4).
#[derive(Default, Clone)]
pub struct ControllerGate {
    inner: Arc<Mutex<GateState>>,
}

pub struct ControllerSlot {
    gate: Arc<Mutex<GateState>>,
    generation: u64,
}

impl ControllerGate {
    pub fn new() -> Self {
        Self::default()
    }

    /// `None` means a controller is already connected.
    pub fn try_claim(&self) -> Option<ControllerSlot> {
        let mut state = self.inner.lock().expect("controller gate poisoned");
        if state.active {
            return None;
        }
        state.active = true;
        state.generation += 1;
        Some(ControllerSlot {
            gate: Arc::clone(&self.inner),
            generation: state.generation,
        })
    }

    pub fn is_held(&self) -> bool {
        self.inner.lock().expect("controller gate poisoned").active
    }
}

impl Drop for ControllerSlot {
    fn drop(&mut self) {
        let mut state = self.gate.lock().expect("controller gate poisoned");
        // The generation check makes a late drop harmless if a force-release
        // path is ever added: a stale slot can only release its own claim.
        if state.generation == self.generation {
            state.active = false;
        }
    }
}

/// Applies the gate to a freshly accepted connection: either the connection
/// is admitted (returning the slot the handler must hold until it is done
/// with the connection), or it is closed with
/// `CONTROLLER_ALREADY_CONNECTED` per doc 7.7.
pub fn admit_controller(gate: &ControllerGate, connection: &Connection) -> Option<ControllerSlot> {
    match gate.try_claim() {
        Some(slot) => Some(slot),
        None => {
            connection.close(
                VarInt::from_u32(CLOSE_CODE_CONTROLLER_ALREADY_CONNECTED),
                CLOSE_REASON_CONTROLLER_ALREADY_CONNECTED,
            );
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::termstream::{Frame, FrameDecoder, OpenPayload, OpenedPayload};
    use std::time::Duration;

    fn test_identity() -> DeviceIdentity {
        let dir = tempfile::tempdir().unwrap();
        DeviceIdentity::load_or_create(&dir.path().join("identity.json")).unwrap()
    }

    async fn bind_loopback(identity: &DeviceIdentity) -> Endpoint {
        bind_endpoint(identity, EndpointProfile::Loopback)
            .await
            .expect("loopback endpoint should bind")
    }

    /// Every network await in these tests is capped: a hang must fail the
    /// test, not wedge the whole suite (as an unbounded read did once
    /// already in session_table's history).
    async fn within<T>(fut: impl std::future::Future<Output = T>) -> T {
        tokio::time::timeout(Duration::from_secs(20), fut)
            .await
            .expect("test network operation timed out")
    }

    #[test]
    fn the_endpoint_id_is_the_device_identity() {
        let identity = test_identity();
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let endpoint = bind_loopback(&identity).await;
            assert_eq!(
                endpoint.id().as_bytes(),
                &identity.public_key_bytes(),
                "iroh must derive the same public key from the same seed"
            );
            endpoint.close().await;
        });
    }

    #[test]
    fn the_connection_code_round_trips_and_is_dialable() {
        let identity = test_identity();
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let agent = bind_loopback(&identity).await;
            let code = connection_code(loopback_addr(&agent));

            // The TS pre-validation's assumptions hold for the real thing.
            assert!(code.starts_with("endpoint"), "unexpected code form: {code}");

            let parsed = parse_connection_code(&code).unwrap();
            assert_eq!(parsed.id, agent.id());
            assert!(
                parsed.addrs.iter().any(|a| a.is_ip()),
                "the ticket must carry the direct address"
            );

            assert!(parse_connection_code("endpointnotaticket").is_err());
            agent.close().await;
        });
    }

    /// Phase A's core exit criterion, run over real loopback QUIC: paste the
    /// code, connect, open a terminal stream, complete the doc 8.2 handshake
    /// with the frames from `termstream`.
    #[test]
    fn a_controller_can_connect_via_the_code_and_complete_the_terminal_handshake() {
        let agent_identity = test_identity();
        let controller_identity = test_identity();
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            let agent = bind_loopback(&agent_identity).await;
            let controller = bind_loopback(&controller_identity).await;
            let code = connection_code(loopback_addr(&agent));

            let agent_side = tokio::spawn({
                let agent = agent.clone();
                async move {
                    let incoming = agent.accept().await.expect("endpoint closed early");
                    let connection = incoming.accept().unwrap().await.unwrap();
                    assert_eq!(connection.alpn(), ALPN_TERMINAL);

                    let (mut send, mut recv) = connection.accept_bi().await.unwrap();

                    let mut decoder = FrameDecoder::new();
                    let mut buf = [0u8; 4096];
                    let open = loop {
                        if let Some(frame) = decoder.next_frame().unwrap() {
                            break frame;
                        }
                        let n = recv.read(&mut buf).await.unwrap().expect("stream ended");
                        decoder.push(&buf[..n]);
                    };
                    assert_eq!(open, Frame::Open(OpenPayload { cols: 80, rows: 24 }));

                    let opened = Frame::Opened(OpenedPayload {
                        session_id: uuid::Uuid::new_v4(),
                        shell: "/bin/bash".into(),
                    });
                    send.write_all(&opened.encode().unwrap()).await.unwrap();
                    send.finish().unwrap();
                    connection.closed().await;
                }
            });

            let addr = parse_connection_code(&code).unwrap();
            let connection = within(controller.connect(addr, ALPN_TERMINAL))
                .await
                .expect("connect via the pasted code should succeed");

            let (mut send, mut recv) = within(connection.open_bi()).await.unwrap();
            let open = Frame::Open(OpenPayload { cols: 80, rows: 24 });
            send.write_all(&open.encode().unwrap()).await.unwrap();

            let mut decoder = FrameDecoder::new();
            let mut buf = [0u8; 4096];
            let opened = loop {
                if let Some(frame) = decoder.next_frame().unwrap() {
                    break frame;
                }
                let n = within(recv.read(&mut buf))
                    .await
                    .unwrap()
                    .expect("agent closed the stream without replying");
                decoder.push(&buf[..n]);
            };
            match opened {
                Frame::Opened(payload) => assert_eq!(payload.shell, "/bin/bash"),
                other => panic!("expected Opened, got {other:?}"),
            }

            connection.close(VarInt::from_u32(0), b"done");
            within(agent_side).await.unwrap();
            agent.close().await;
            controller.close().await;
        });
    }

    /// Doc 7.7 over real QUIC: while one controller holds the gate a second
    /// connection is closed with CONTROLLER_ALREADY_CONNECTED, the first
    /// connection stays usable, and after the first controller leaves a new
    /// one is admitted.
    #[test]
    fn a_second_controller_is_refused_until_the_first_disconnects() {
        let agent_identity = test_identity();
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            let agent = bind_loopback(&agent_identity).await;
            let first = bind_loopback(&test_identity()).await;
            let second = bind_loopback(&test_identity()).await;
            let third = bind_loopback(&test_identity()).await;
            let addr = || parse_connection_code(&connection_code(loopback_addr(&agent))).unwrap();
            let gate = ControllerGate::new();

            let agent_side = tokio::spawn({
                let agent = agent.clone();
                let gate = gate.clone();
                async move {
                    // First controller: admitted, slot held while serving.
                    let conn1 = agent.accept().await.unwrap().accept().unwrap().await.unwrap();
                    let slot = admit_controller(&gate, &conn1).expect("first controller admitted");

                    // Second controller: refused while the slot is held.
                    let conn2 = agent.accept().await.unwrap().accept().unwrap().await.unwrap();
                    assert!(admit_controller(&gate, &conn2).is_none());

                    // The first connection is still serviceable after the
                    // refusal: complete one echo round-trip on a fresh stream.
                    let (mut send, mut recv) = conn1.accept_bi().await.unwrap();
                    let mut byte = [0u8; 5];
                    recv.read_exact(&mut byte).await.unwrap();
                    send.write_all(&byte).await.unwrap();
                    send.finish().unwrap();

                    conn1.closed().await;
                    drop(slot);

                    // Third controller: admitted now that the gate reopened.
                    let conn3 = agent.accept().await.unwrap().accept().unwrap().await.unwrap();
                    let slot3 = admit_controller(&gate, &conn3).expect("gate must reopen");
                    drop(slot3);
                    conn3.close(VarInt::from_u32(0), b"");
                }
            });

            let conn1 = within(first.connect(addr(), ALPN_TERMINAL)).await.unwrap();

            let conn2 = within(second.connect(addr(), ALPN_TERMINAL)).await.unwrap();
            let reason = within(conn2.closed()).await.to_string();
            assert!(
                reason.contains("CONTROLLER_ALREADY_CONNECTED"),
                "the refusal must carry the doc 13 error code, got: {reason}"
            );

            // First controller unaffected by the refused second one.
            let (mut send, mut recv) = within(conn1.open_bi()).await.unwrap();
            send.write_all(b"still").await.unwrap();
            let mut echo = [0u8; 5];
            within(recv.read_exact(&mut echo)).await.unwrap();
            assert_eq!(&echo, b"still");
            conn1.close(VarInt::from_u32(0), b"bye");

            let conn3 = within(third.connect(addr(), ALPN_TERMINAL)).await.unwrap();
            within(conn3.closed()).await;

            within(agent_side).await.unwrap();
            for endpoint in [agent, first, second, third] {
                endpoint.close().await;
            }
        });
    }

    #[test]
    fn the_gate_itself_is_a_plain_state_machine() {
        let gate = ControllerGate::new();
        assert!(!gate.is_held());

        let slot = gate.try_claim().expect("free gate must admit");
        assert!(gate.is_held());
        assert!(gate.try_claim().is_none(), "held gate must refuse");

        drop(slot);
        assert!(!gate.is_held());
        assert!(gate.try_claim().is_some(), "released gate must admit again");
    }
}
