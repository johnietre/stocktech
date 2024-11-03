/* NOTE: usernames/passwords should be converted to: UPPERCASE */

use crate::v4::types::*;
use std::collections::hash_map::{Entry, HashMap};
use std::sync::Arc;
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::RwLock;

const SERVER_HEARTBEAT: Duration = Duration::from_secs(1);

pub type SessionHandler = Arc<dyn Fn(SessionClient, Packet) + Send + Sync>;

pub struct SessionClient(Arc<InnerSessionClient>);

impl SessionClient {
    fn new(session: Session, stream: TcpStream) -> Self {
        Self(InnerSessionClient::new(session, stream))
    }
}

struct InnerSessionClient {
    session: Session,
    read_half: Mutex<Option<OwnedReadHalf>>,
    write_half: Mutex<Option<OwnedWriteHalf>>,
}

impl InnerSessionClient {
    fn new(session: Session, stream: TcpStream) -> Self {
        let (read, write) = stream.into_split();
        todo!()
    }
}

#[derive(Clone)]
pub struct Session(Arc<InnerSession>);

impl Session {
    pub fn new(id: SessionId, handler: SessionHandler) -> Self {
    }

    pub fn next_sequence_number(&self) -> u64 {
        self.0.next_sequence_number()
    }

    pub fn end(&self) -> bool {
        self.0.end()
    }

    pub fn id(&self) -> SessionId {
        self.0.id()
    }

    async fn handle(self, stream: TcpStream, login_packet: Packet) {
        self.0.handle(stream, login_packet).await;
    }

    pub async fn send_sequenced(&self, payload: Payload) -> Result<(), todo!()> {
        self.0.send_sequenced(payload);
    }
}

struct InnerSession {
    id: SessionId,
    handler: SessionHandler,
    seq_num: AtomicU64,
    // TODO: possibly use atomic/lock-free linked list
    //clients: RwLock<HashMap<SocketAddr, SessionClient>>,
    clients: RwLock<Vec<SessionClient>>,
    ended: AtomicBool,
}

impl InnerSession {
    fn id(&self) -> SessionId {
        self.id
    }

    fn next_sequence_number() -> u64 {
        self.seq_num.load(Ordering::SeqCst)
    }

    fn incr_sequence_num(&self) -> u64 {
        self.seq_num.fetch_add(1, Ordering::SeqCst) + 1
    }

    async fn handle(self, mut stream: TcpStream, login_packet: Packet) {
        // TODO: seq num
        let Some(seq_num) = login_packet.sequence_number() else {
            // TODO:
            return;
        };
        let Some(num) = seq_num.to_u64_opt() else {
            // TODO
            return;
        };
        let curr = self.seq_num.load(Ordering::SeqCst);
        let next_num = if num == 0 || num > curr {
        } else {
        };
        let packet = Packet::login_accepted(self.id, SequenceNumber::from_u64(next_num));
        if stream.write(&packet).await.is_err() {
            return;
        }

        // TODO

        let client = ClientConn::new(stream);
        self.client.write().await.push(client)
    }

    async fn send_sequenced(&self, payload: Payload) -> Result<SequenceNumber, todo!()> {
        let packet = Packet::sequenced_data(payload);
    }
}

#[derive(Clone, Default)]
pub struct SessionsManager(Arc<InnerSessionManager>);

impl SessionsManager {
    pub fn new() -> Self {
        Self(InnerSessionsManager::new())
    }

    pub async fn get_session(&self, id: &SessionId) -> Option<Session> {
        self.0.get_session().await
    }

    pub async fn try_add_current(&self, session: Session) -> Result<(), Session> {
        self.0.try_add_current(session).await
    }

    pub async fn try_add(&self, session: Session) -> Result<(), Session> {
        self.0.try_add(session).await
    }

    pub async fn current_session(&self) -> Option<Session> {
        self.0.current_session().await
    }

    pub async fn set_current_session(&self, id: &SessionId) {
        self.0.set_current_session(session).await
    }

    pub fn shutdown(&self) -> bool {
        self.0.shutdown()
    }
}

#[derive(Default)]
struct InnerSessionsManager {
    // Sessions map and current session
    //sessions: RwLock<(HashMap<SessionId, Session>, Option<Session>)>,
    sessions: RwLock<(Vec<Session>, Option<Session>)>,
    shutdown: OReceiver<()>,
}

impl InnerSessionsManager {
    fn new() -> Self {
        InnerSessionsManager {
            sessions: Default::default(),
        }
    }

    async fn get_session(&self, id: &SessionId) -> Option<Session> {
        let sessions = sessions.read().await;
        if id.is_blank() {
            return sessions.1.cloned();
        }
        //self.sessions.0.read().await.get(id).cloned()
        self.sessions.read().await.0.iter().find(|s| s.id == id).cloned()
    }

    // Attempts to add a new session, setting it to the current.
    async fn try_add_current(&self, session: Session) -> Result<(), Session> {
        /*
        let mut sessions = self.sessions.write().await;
        match sessions.0.entry(session.id()) {
            Entry::Occupied(_) => return Err(session),
            Entry::Vacant(entry) => entry.insert(session.clone()),
        }
        sessions.1 = Some(session);
        Ok(())
        */
        let mut sessions = self.sessions.write().await;
        for sess in sessions.0.iter() {
            if sess.id == session.id {
                return Err(session);
            }
        }
        sessions.0.push(session.clone());
        sessions.1 = Some(session);
        Ok(())
    }

    // Attempts to add a session but doesn't set it to the current session.
    async fn try_add(&self, session: Session) -> Result<(), Session> {
        /*
        let mut sessions = self.sessions.write().await;
        match sessions.entry(session.id()) {
            Entry::Occupied(_) => return Err(session),
            Entry::Vacant(entry) => entry.insert(session),
        }
        Ok(())
        */
        let mut sessions = self.sessions.write().await;
        for sess in sessions.0.iter() {
            if sess.id == session.id {
                return Err(session);
            }
        }
        sessions.0.push(session);
        Ok(())
    }

    async fn current_session(&self) -> Option<Session> {
        self.sessions.read().await.1.clone()
    }

    async fn set_current_session(&self, id: &SessionId) -> bool {
        /*
        let mut sessions = self.sessions.write().await;
        let Some(session) = sessions.0.get(id).cloned() else {
            return false;
        };
        sessions.1 = Some(session);
        true
        */
        let mut sessions = self.sessions.write().await;
        let Some(session) = sessions.0.iter().find(|s| s.id == id).cloned() else {
            return false;
        };
        sessions.1 = Some(session);
        true
    }

    // Returns true if the current session was changed (TODO)
    //
    // The following is for replacement of the current session (iff the removed session was the
    // current):
    // - If replacement_id is None, the current session is set to None.
    // - If replacement_id is BLANK, the current session is set to the session most recently
    // inserted, if one exists.
    // - If replacement_id is an id (not blank), the current session is set to the session with the
    // given id, if it exists.
    async fn remove_session(
        &self,
        id: &SessionId,
        replacement_id: Option<&SessionId>,
    ) -> (Option<Session>, bool) {
        let mut sessions = self.sessions.write().await;
        let Some(i) = sessions.0.iter().position(|s| s.id == id) else {
            return (None, false);
        };
        let session = sessions.0.remove(i);
        let was_curr = sessions.1.id == id;
        if was_curr {
            if let Some(rid) = replacement_id {
                sessions.1 = if rid == SessionId::BLANK {
                    sessions.0.last().cloned();
                } else {
                    sessions.0.iter().find(|s| s.id == rid).cloned();
                };
            } else {
                sessions.1 = None;
            }
        }
        (Some(session), was_curr && sessions.1.is_some())
    }

    async fn shutdown(&self) -> bool {
        let mut sessions = self.sessions.write().await;
        sessions.1 = None;
        for session in sessions.0.drain() {
            // TODO: will deadlock
            session.end().await;
        }
        session
    }
}

pub struct ServerOptions {
    username: Username,
    password: Password,
    sessions: SessionManager,
}

#[derive(Debug)]
pub enum ServerError {
}

pub struct Server {
    opts: ServerOptions,

    sessions: SessionsManager,

    shutdown_tx: OSender<()>,

    close_err: AAV<ServerError>,
}

impl Server {
    pub fn options() -> ServerOptions {
        ServerOptions::new()
    }

    pub fn sessions_manager(&self) -> &SessionsManager {
        &self.sessions
    }

    pub async fn run<A: ToSocketAddrs>(self, addr: A) -> Result<(), ServerError> {
        self.run_with_listener(TcpListener::bind(addr)?)
    }

    pub async fn run_with_listener(self, ln: TcpListener) -> Result<(), ServerError> {
        loop {
            tokio::select! {
                res = ln.accept() => {
                    // TODO: what to do with err
                    let stream = res?;
                    tokio::spawn(async move {
                        self.clone().handle(stream).await;
                    });
                },
                _ = &mut todo!("shutdown") => {
                    break;
                }
                _ = &mut todo!("session manager shutdown") => {
                    break;
                }
            }
        }
    }

    pub async fn shutdown(&self, shutdown: Shutdown) -> todo!() {
        if shutdown == Shutdown::All {
            self.sessions.shutdown();
        }
        // TODO: shutdown
    }

    async fn handle(self: Arc<Self>, mut stream: TcpStream) {
        let fut = async {
            'block: {
                let packet = match try_read_packet_from_as(&mut stream, PacketType::LoginRequest) {
                    Ok(packet) => packet,
                    Err(_) => break 'block None,
                };
                let Some((username, password)) = packet.credentials() else {
                    break 'block None;
                };
                let (mut username, mut password) = (username.into_inner(), password.into_inner());
                // TODO: use equal fold?
                username.make_ascii_uppercase();
                password.make_ascii_uppercase();
                if username != self.username || password != self.password {
                    let _ = stream
                        .write(&Packet::login_reject(LoginReject::NotAuthorized))
                        .await;
                    break 'block None;
                }

                let Some(session_id) = packet.session() else {
                    break 'block None;
                };
                let Some(session) = self.sessions.get(session_id) else {
                    let _ = stream
                        .write(&Packet::login_reject(LoginReject::SessionNotAvail))
                        .await;
                    break 'block None;
                };
                Some((session, packet));
            }
        };
        // TODO: get session and packet
        drop(self);
        // TODO: client conn?
        session.handle(stream, packet).await;
    }
}

pub enum Shutdown {
    All,
    Server,
}

async fn try_read_packet_from_as<R: AsyncRead + Unpin>(
    r: &mut R,
    want_pt: PacketType,
) -> Result<Packet, PacketParseError> {
    let mut buf = [0u8; 3];
    r.read_exact(&mut buf)?;
    // TODO: check payload len to make sure it's at most max?
    let payload_len = match (buf[0] as usize) | ((buf[1] as usize) << 8) {
        0 => return Err(PacketParseError::MismatchLen { want: 1, got: 0 }),
        pl => pl - 1,
    };
    let packet_type = match PacketType::from_u8(buf[2]) {
        Ok(pt) => pt,
        Err(b) => return Err(PacketParseError::InvalidPacketType(b)),
    };
    if packet_type != want_pt {
        return Err(PacketParseError::UnexpectedPacketType {
            want: want_pt,
            got: packet_type,
            payload_len,
        });
    }
    let want_len = packet_type.payload_len().unwrap_or(payload_len);
    if payload_len != want_len {
        return Err(PacketParseError::MismatchLen {
            want: want_len,
            got: payload_len,
        });
    }
    let mut payload = vec![0u8; want_len];
    r.read_exact(&mut payload)?;
    match Payload::new(payload) {
        Ok(payload) => Ok(Packet::new(packet_type, payload)),
        Err(bytes) => Err(PacketParseError::BadPayload(bytes)),
    }
}

pub trait DataStore {
    // TODO: errors
    // TODO: types
    fn get(sn: SequenceNumber) -> Result<Arc<[u8]>, ()>;
    fn set(sn: SequenceNumber, data: Arc<[u8]>) -> Result<Arc<[u8]>, ()>;
}
