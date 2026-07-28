//! In-memory connection, session and transfer registry (doc 11.1).
//!
//! Nothing here is persisted: a relay restart drops every connection, which doc
//! 17.10 lists as a known limitation. Agents reconnect with backoff (doc 7.6),
//! so the state rebuilds itself.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use uuid::Uuid;

/// One message queued for a socket.
#[derive(Debug, Clone)]
pub enum Outbound {
    Text(String),
    Binary(Vec<u8>),
    Ping,
    Close(u16, String),
}

impl Outbound {
    pub fn byte_len(&self) -> usize {
        match self {
            Outbound::Text(s) => s.len(),
            Outbound::Binary(b) => b.len(),
            Outbound::Ping => 0,
            Outbound::Close(_, r) => r.len(),
        }
    }
}

/// Channel capacities.
///
/// The split is the whole point of doc 8.6's directional rule. `control` carries
/// JSON and terminal frames; `file` carries chunks only. Sizing `file` at 64
/// slots of 256 KiB puts its ceiling at the 16 MiB hard limit, so a sender that
/// respects its credit window can never fill it - and one that does not gets
/// closed with BACKPRESSURE_LIMIT instead of stalling the terminal.
pub const CONTROL_CAPACITY: usize = 256;
pub const FILE_CAPACITY: usize = 64;

#[derive(Clone)]
pub struct ConnHandle {
    pub control: tokio::sync::mpsc::Sender<Outbound>,
    pub file: tokio::sync::mpsc::Sender<Outbound>,
}

impl ConnHandle {
    pub fn channel() -> (Self, tokio::sync::mpsc::Receiver<Outbound>, tokio::sync::mpsc::Receiver<Outbound>) {
        let (control, control_rx) = tokio::sync::mpsc::channel(CONTROL_CAPACITY);
        let (file, file_rx) = tokio::sync::mpsc::channel(FILE_CAPACITY);
        (Self { control, file }, control_rx, file_rx)
    }

    /// Non-blocking send on the control lane.
    ///
    /// Never awaits: this lane is used while holding no lock but inside a read
    /// loop, and blocking here would stall the socket that is being read.
    pub fn try_send_control(&self, msg: Outbound) -> Result<(), SendError> {
        self.control.try_send(msg).map_err(|e| match e {
            tokio::sync::mpsc::error::TrySendError::Full(_) => SendError::Full,
            tokio::sync::mpsc::error::TrySendError::Closed(_) => SendError::Closed,
        })
    }

    pub fn try_send_file(&self, msg: Outbound) -> Result<(), SendError> {
        self.file.try_send(msg).map_err(|e| match e {
            tokio::sync::mpsc::error::TrySendError::Full(_) => SendError::Full,
            tokio::sync::mpsc::error::TrySendError::Closed(_) => SendError::Closed,
        })
    }

    /// Blocking send, used only for Agent -> plugin terminal output.
    ///
    /// This is doc 8.6's downstream watermark: when the plugin cannot keep up,
    /// awaiting here stops the task reading the agent socket, which propagates
    /// back through the PTY and throttles the remote process. Safe in that
    /// direction precisely because no file body flows downstream in MVP.
    pub async fn send_control(&self, msg: Outbound) -> Result<(), SendError> {
        self.control.send(msg).await.map_err(|_| SendError::Closed)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SendError {
    Full,
    Closed,
}

struct AgentEntry {
    user_id: Uuid,
    handle: ConnHandle,
    /// Server clock only. `agent.heartbeat.timestamp` is not trusted (doc 11.1).
    last_seen: Instant,
}

struct ControlEntry {
    handle: ConnHandle,
}

pub struct DroppedRoutes {
    pub sessions: Vec<(Uuid, Route)>,
    pub transfers: Vec<(Uuid, Route)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Route {
    pub user_id: Uuid,
    pub device_id: Uuid,
}

#[derive(Default)]
struct Inner {
    agents: HashMap<Uuid, AgentEntry>,
    controls: HashMap<Uuid, ControlEntry>,
    sessions: HashMap<Uuid, Route>,
    transfers: HashMap<Uuid, Route>,
}

pub struct Registry {
    inner: Mutex<Inner>,
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry {
    pub fn new() -> Self {
        Self { inner: Mutex::new(Inner::default()) }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().expect("registry mutex poisoned")
    }

    // --- agents -------------------------------------------------------------

    /// Registers an agent connection. Doc 8.2: one connection per device, the
    /// new one wins. The displaced handle is returned so the caller can close it
    /// with 4409 after releasing the lock.
    pub fn register_agent(
        &self,
        device_id: Uuid,
        user_id: Uuid,
        handle: ConnHandle,
    ) -> Option<ConnHandle> {
        let mut inner = self.lock();
        let previous = inner.agents.remove(&device_id).map(|e| e.handle);
        inner.agents.insert(device_id, AgentEntry { user_id, handle, last_seen: Instant::now() });
        previous
    }

    /// Removes an agent only if the stored handle is the one that is leaving.
    ///
    /// Guards against a displaced connection's cleanup deleting the replacement
    /// that just took its place.
    pub fn unregister_agent_if_current(&self, device_id: Uuid, handle: &ConnHandle) -> bool {
        let mut inner = self.lock();
        let is_current = inner
            .agents
            .get(&device_id)
            .is_some_and(|e| e.handle.control.same_channel(&handle.control));
        if is_current {
            inner.agents.remove(&device_id);
        }
        is_current
    }

    pub fn agent_handle(&self, device_id: Uuid) -> Option<ConnHandle> {
        self.lock().agents.get(&device_id).map(|e| e.handle.clone())
    }

    pub fn is_online(&self, device_id: Uuid) -> bool {
        self.lock().agents.contains_key(&device_id)
    }

    pub fn online_devices(&self) -> Vec<Uuid> {
        self.lock().agents.keys().copied().collect()
    }

    pub fn touch_agent(&self, device_id: Uuid) {
        if let Some(entry) = self.lock().agents.get_mut(&device_id) {
            entry.last_seen = Instant::now();
        }
    }

    pub fn agent_last_seen(&self, device_id: Uuid) -> Option<Instant> {
        self.lock().agents.get(&device_id).map(|e| e.last_seen)
    }

    pub fn agent_owner(&self, device_id: Uuid) -> Option<Uuid> {
        self.lock().agents.get(&device_id).map(|e| e.user_id)
    }

    // --- control connections ------------------------------------------------

    /// Doc 4.10: one control connection per account, new wins. This is what lets
    /// responses route by `userId` alone, with no requestId table.
    pub fn register_control(&self, user_id: Uuid, handle: ConnHandle) -> Option<ConnHandle> {
        let mut inner = self.lock();
        let previous = inner.controls.remove(&user_id).map(|e| e.handle);
        inner.controls.insert(user_id, ControlEntry { handle });
        previous
    }

    pub fn unregister_control_if_current(&self, user_id: Uuid, handle: &ConnHandle) -> bool {
        let mut inner = self.lock();
        let is_current = inner
            .controls
            .get(&user_id)
            .is_some_and(|e| e.handle.control.same_channel(&handle.control));
        if is_current {
            inner.controls.remove(&user_id);
        }
        is_current
    }

    pub fn control_handle(&self, user_id: Uuid) -> Option<ConnHandle> {
        self.lock().controls.get(&user_id).map(|e| e.handle.clone())
    }

    // --- sessions and transfers ---------------------------------------------

    pub fn open_session(&self, session_id: Uuid, route: Route) {
        self.lock().sessions.insert(session_id, route);
    }

    pub fn session_route(&self, session_id: Uuid) -> Option<Route> {
        self.lock().sessions.get(&session_id).copied()
    }

    pub fn close_session(&self, session_id: Uuid) -> Option<Route> {
        self.lock().sessions.remove(&session_id)
    }

    pub fn device_has_session(&self, device_id: Uuid) -> bool {
        self.lock().sessions.values().any(|r| r.device_id == device_id)
    }

    pub fn open_transfer(&self, transfer_id: Uuid, route: Route) {
        self.lock().transfers.insert(transfer_id, route);
    }

    pub fn transfer_route(&self, transfer_id: Uuid) -> Option<Route> {
        self.lock().transfers.get(&transfer_id).copied()
    }

    pub fn close_transfer(&self, transfer_id: Uuid) -> Option<Route> {
        self.lock().transfers.remove(&transfer_id)
    }

    /// Doc 11.2.4: when either side of a route goes away, drop the routes and
    /// report them so the peer can be told.
    pub fn drop_routes_for_device(&self, device_id: Uuid) -> DroppedRoutes {
        self.drop_routes(|route| route.device_id == device_id)
    }

    pub fn drop_routes_for_user(&self, user_id: Uuid) -> DroppedRoutes {
        self.drop_routes(|route| route.user_id == user_id)
    }

    /// Returns the routes themselves, not just their ids: the caller has to tell
    /// the peer, and once a route is removed there is nowhere left to look up
    /// which device it pointed at.
    fn drop_routes(&self, matches: impl Fn(&Route) -> bool) -> DroppedRoutes {
        let mut inner = self.lock();

        let sessions: Vec<(Uuid, Route)> = inner
            .sessions
            .iter()
            .filter(|(_, r)| matches(r))
            .map(|(id, r)| (*id, *r))
            .collect();
        let transfers: Vec<(Uuid, Route)> = inner
            .transfers
            .iter()
            .filter(|(_, r)| matches(r))
            .map(|(id, r)| (*id, *r))
            .collect();

        for (id, _) in &sessions {
            inner.sessions.remove(id);
        }
        for (id, _) in &transfers {
            inner.transfers.remove(id);
        }

        DroppedRoutes { sessions, transfers }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn handle() -> ConnHandle {
        ConnHandle::channel().0
    }

    #[test]
    fn a_new_agent_connection_displaces_the_old_one() {
        let registry = Registry::new();
        let device = Uuid::new_v4();
        let user = Uuid::new_v4();

        let first = handle();
        assert!(registry.register_agent(device, user, first.clone()).is_none());

        let second = handle();
        let displaced = registry.register_agent(device, user, second.clone()).unwrap();
        assert!(displaced.control.same_channel(&first.control));

        // The displaced connection's own cleanup must not evict the replacement.
        assert!(!registry.unregister_agent_if_current(device, &first));
        assert!(registry.is_online(device));

        assert!(registry.unregister_agent_if_current(device, &second));
        assert!(!registry.is_online(device));
    }

    #[test]
    fn a_new_control_connection_displaces_the_old_one() {
        let registry = Registry::new();
        let user = Uuid::new_v4();

        let first = handle();
        assert!(registry.register_control(user, first.clone()).is_none());
        let second = handle();
        assert!(registry.register_control(user, second.clone()).is_some());

        assert!(!registry.unregister_control_if_current(user, &first));
        assert!(registry.control_handle(user).is_some());
        assert!(registry.unregister_control_if_current(user, &second));
        assert!(registry.control_handle(user).is_none());
    }

    #[test]
    fn routes_are_dropped_with_their_device() {
        let registry = Registry::new();
        let user = Uuid::new_v4();
        let device = Uuid::new_v4();
        let other_device = Uuid::new_v4();

        let session = Uuid::new_v4();
        let transfer = Uuid::new_v4();
        let unrelated = Uuid::new_v4();

        registry.open_session(session, Route { user_id: user, device_id: device });
        registry.open_transfer(transfer, Route { user_id: user, device_id: device });
        registry.open_session(unrelated, Route { user_id: user, device_id: other_device });

        assert!(registry.device_has_session(device));

        let dropped = registry.drop_routes_for_device(device);
        assert_eq!(dropped.sessions.len(), 1);
        assert_eq!(dropped.sessions[0].0, session);
        assert_eq!(dropped.sessions[0].1.device_id, device);
        assert_eq!(dropped.transfers.len(), 1);
        assert_eq!(dropped.transfers[0].0, transfer);

        assert!(!registry.device_has_session(device));
        assert!(registry.session_route(unrelated).is_some(), "another device's route survives");
    }

    #[test]
    fn routes_are_dropped_with_their_user() {
        let registry = Registry::new();
        let alice = Uuid::new_v4();
        let bob = Uuid::new_v4();
        let device = Uuid::new_v4();

        let alice_session = Uuid::new_v4();
        let bob_session = Uuid::new_v4();
        registry.open_session(alice_session, Route { user_id: alice, device_id: device });
        registry.open_session(bob_session, Route { user_id: bob, device_id: device });

        let dropped = registry.drop_routes_for_user(alice);
        assert_eq!(dropped.sessions.len(), 1);
        assert_eq!(dropped.sessions[0].0, alice_session);
        assert!(registry.session_route(bob_session).is_some());
    }

    #[test]
    fn file_lane_fills_before_the_control_lane_stalls() {
        // The property doc 8.6 depends on: a flood of file chunks must not be
        // able to block a control message.
        let (handle, _control_rx, _file_rx) = ConnHandle::channel();

        for _ in 0..FILE_CAPACITY {
            assert_eq!(handle.try_send_file(Outbound::Binary(vec![0; 16])), Ok(()));
        }
        assert_eq!(handle.try_send_file(Outbound::Binary(vec![0; 16])), Err(SendError::Full));

        // Control still has room.
        assert_eq!(handle.try_send_control(Outbound::Text("{}".into())), Ok(()));
    }
}
