use std::borrow::Borrow;
use std::convert::AsRef;
use std::error::Error;
use std::fmt;
use std::io::{prelude::*, Cursor, Error as IoError};
use std::ops::{Deref, DerefMut};

pub const MAX_PAYLOAD_LEN: usize = 0xFFFE;
pub const USERNAME_LEN: usize = 6;
pub const PASSWORD_LEN: usize = 10;
pub const SESSION_ID_LEN: usize = 10;
pub const SEQUENCE_NUMBER_LEN: usize = 20;

#[repr(transparent)]
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Payload(Box<[u8]>);

impl Payload {
    /// Returns the value back if it's length is longer than the maximum payload length.
    pub fn new<T: Into<Box<[u8]>>>(arr: T) -> Result<Self, Box<[u8]>> {
        let arr = arr.into();
        if arr.len() > MAX_PAYLOAD_LEN {
            return Err(arr);
        }
        Ok(Self(arr))
    }

    pub fn into_inner(Self(inner): Self) -> Box<[u8]> {
        inner
    }
}

impl Deref for Payload {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl DerefMut for Payload {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

impl AsRef<[u8]> for Payload {
    fn as_ref(&self) -> &[u8] {
        &*self.0
    }
}

impl Borrow<[u8]> for Payload {
    fn borrow(&self) -> &[u8] {
        &*self.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Username([u8; USERNAME_LEN]);

impl Username {
    /// Returns the value back if it's length is longer than the length of a username. If it is
    /// less, padding bytes are added to the end.
    pub fn new<T: AsRef<[u8]>>(slice_t: T) -> Result<Self, T> {
        Self::from_slice(slice_t.as_ref()).map_err(|_| slice_t)
    }

    /// Same as Username::new but silently leaves out extra bytes.
    pub fn new_trunc(slice: impl AsRef<[u8]>) -> Self {
        let slice = slice.as_ref();
        if slice.len() <= USERNAME_LEN {
            Self::from_slice(&slice).unwrap()
        } else {
            Self::from_slice(&slice[..USERNAME_LEN]).unwrap()
        }
    }

    #[inline(always)]
    fn from_slice(slice: &[u8]) -> Result<Self, ()> {
        let slice = slice.as_ref();
        if slice.len() > USERNAME_LEN {
            return Err(());
        }
        let mut arr = [b' '; USERNAME_LEN];
        arr[..slice.len()].copy_from_slice(slice);
        Ok(Self(arr))
    }

    pub const fn into_inner(Self(inner): Self) -> [u8; USERNAME_LEN] {
        inner
    }
}

impl Default for Username {
    fn default() -> Self {
        Self([b' '; USERNAME_LEN])
    }
}

impl Deref for Username {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<[u8]> for Username {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Borrow<[u8]> for Username {
    fn borrow(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Password([u8; PASSWORD_LEN]);

impl Password {
    /// Returns the value back if it's length is longer than the length of a password. If it is
    /// less, padding bytes are added to the end.
    pub fn new<T: AsRef<[u8]>>(slice_t: T) -> Result<Self, T> {
        Self::from_slice(slice_t.as_ref()).map_err(|_| slice_t)
    }

    /// Same as Password::new but silently leaves out extra bytes.
    pub fn new_trunc(slice: impl AsRef<[u8]>) -> Self {
        let slice = slice.as_ref();
        if slice.len() <= PASSWORD_LEN {
            Self::from_slice(&slice).unwrap()
        } else {
            Self::from_slice(&slice[..PASSWORD_LEN]).unwrap()
        }
    }

    #[inline(always)]
    fn from_slice(slice: &[u8]) -> Result<Self, ()> {
        let slice = slice.as_ref();
        if slice.len() > PASSWORD_LEN {
            return Err(());
        }
        let mut arr = [b' '; PASSWORD_LEN];
        arr[..slice.len()].copy_from_slice(slice);
        Ok(Self(arr))
    }

    pub const fn into_inner(Self(inner): Self) -> [u8; PASSWORD_LEN] {
        inner
    }
}

impl Default for Password {
    fn default() -> Self {
        Self([b' '; PASSWORD_LEN])
    }
}

impl Deref for Password {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<[u8]> for Password {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Borrow<[u8]> for Password {
    fn borrow(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId([u8; SESSION_ID_LEN]);

impl SessionId {
    pub const BLANK: Self = Self([b' '; SESSION_ID_LEN]);

    /// Returns the value back if it's length is longer than the length of a session. If it is
    /// less, padding bytes are added to the beginning.
    pub fn new<T: AsRef<[u8]>>(slice_t: T) -> Result<Self, T> {
        Self::from_slice(slice_t.as_ref()).map_err(|_| slice_t)
    }

    /// Same as SessionId::new but silently leaves out extra bytes.
    pub fn new_trunc(slice: impl AsRef<[u8]>) -> Self {
        let slice = slice.as_ref();
        if slice.len() <= SESSION_ID_LEN {
            Self::from_slice(&slice).unwrap()
        } else {
            Self::from_slice(&slice[..SESSION_ID_LEN]).unwrap()
        }
    }

    #[inline(always)]
    fn from_slice(slice: &[u8]) -> Result<Self, ()> {
        let slice = slice.as_ref();
        if slice.len() > SESSION_ID_LEN {
            return Err(());
        }
        let mut arr = [b' '; SESSION_ID_LEN];
        arr[SESSION_ID_LEN - slice.len()..].copy_from_slice(slice);
        Ok(Self(arr))
    }

    pub const fn into_inner(Self(inner): Self) -> [u8; SESSION_ID_LEN] {
        inner
    }

    pub fn is_blank(&self) -> bool {
        self.0 == Self::BLANK.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self([b' '; SESSION_ID_LEN])
    }
}

impl Deref for SessionId {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<[u8]> for SessionId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Borrow<[u8]> for SessionId {
    fn borrow(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SequenceNumber([u8; SEQUENCE_NUMBER_LEN]);

impl SequenceNumber {
    pub const ZERO: Self = Self([
        b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ',
        b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b'0',
    ]);

    /// Returns the value back if it's length is longer than the length of a sequence number. If
    /// it is less, padding bytes are added to the beginning.
    pub fn new<T: AsRef<[u8]>>(slice_t: T) -> Result<Self, T> {
        Self::from_slice(slice_t.as_ref()).map_err(|_| slice_t)
    }

    /// Same as SequenceNumber::new but silently leaves out extra bytes.
    pub fn new_trunc(slice: impl AsRef<[u8]>) -> Self {
        let slice = slice.as_ref();
        if slice.len() <= SEQUENCE_NUMBER_LEN {
            Self::from_slice(&slice).unwrap()
        } else {
            Self::from_slice(&slice[..SEQUENCE_NUMBER_LEN]).unwrap()
        }
    }

    #[inline(always)]
    fn from_slice(slice: &[u8]) -> Result<Self, ()> {
        let slice = slice.as_ref();
        if slice.len() > SEQUENCE_NUMBER_LEN {
            return Err(());
        }
        let mut arr = [b' '; SEQUENCE_NUMBER_LEN];
        arr[SEQUENCE_NUMBER_LEN - slice.len()..].copy_from_slice(slice);
        Ok(Self(arr))
    }

    pub fn from_u64(mut u: u64) -> Self {
        let mut seq_num = Self::ZERO;
        for i in (0..SEQUENCE_NUMBER_LEN).rev() {
            if u == 0 {
                break;
            }
            seq_num.0[i] = (u % 10) as u8 + b'0';
            u /= 10;
        }
        seq_num
    }

    pub fn to_u64(&self) -> u64 {
        let mut n = 0;
        for i in 0..SEQUENCE_NUMBER_LEN {
            match self[i] {
                b @ b'0'..=b'9' => n = n * 10 + (b - b'0') as u64,
                _ => (),
            }
        }
        n
    }

    pub fn to_u64_opt(&self) -> Option<u64> {
        let (mut n, mut in_num) = (0, false);
        for i in 0..SEQUENCE_NUMBER_LEN {
            match self[i] {
                b @ b'0'..=b'9' => {
                    in_num = true;
                    n = n * 10 + (b - b'0') as u64;
                }
                b' ' => {
                    if in_num {
                        return None;
                    }
                }
                _ => return None,
            }
        }
        Some(n)
    }

    pub fn is_valid(&self) -> bool {
        self.to_u64_opt().is_some()
    }

    pub const fn into_inner(Self(inner): Self) -> [u8; SEQUENCE_NUMBER_LEN] {
        inner
    }
}

impl From<u64> for SequenceNumber {
    fn from(u: u64) -> Self {
        Self::from_u64(u)
    }
}

impl Default for SequenceNumber {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Deref for SequenceNumber {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<[u8]> for SequenceNumber {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Borrow<[u8]> for SequenceNumber {
    fn borrow(&self) -> &[u8] {
        &self.0
    }
}

// TODO: possibly make struct
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PacketType {
    // Debug is the packet type for Debug packets.
    Debug = b'+',
    // LoginAccepted is the packet type for LoginAccepted packets.
    LoginAccepted = b'A',
    // LoginReject is the packet type for LoginReject packets.
    LoginReject = b'J',
    // SequencedData is the packet type for SequencedData packets.
    SequencedData = b'S',
    // UnsequencedData is the packet type for UnsequencedData packets.
    UnsequencedData = b'U',
    // ServerHeartbeat is the packet type for ServerHeartbeat packets.
    ServerHeartbeat = b'H',
    // EndOfSession is the packet type for EndOfSession packets.
    EndOfSession = b'Z',
    // LoginRequest is the packet type for LoginRequest packets.
    LoginRequest = b'L',
    // ClientHeartbeat is the packet type for ClientHeartbeat packets.
    ClientHeartbeat = b'R',
    // LogoutRequest is the packet type for LogoutRequest packets.
    LogoutRequest = b'O',
}

impl PacketType {
    pub const fn from_u8(b: u8) -> Result<Self, u8> {
        match b {
            b'+' => Ok(PacketType::Debug),
            b'A' => Ok(PacketType::LoginAccepted),
            b'J' => Ok(PacketType::LoginReject),
            b'S' => Ok(PacketType::SequencedData),
            b'U' => Ok(PacketType::UnsequencedData),
            b'H' => Ok(PacketType::ServerHeartbeat),
            b'Z' => Ok(PacketType::EndOfSession),
            b'L' => Ok(PacketType::LoginRequest),
            b'R' => Ok(PacketType::ClientHeartbeat),
            b'O' => Ok(PacketType::LogoutRequest),
            _ => Err(b),
        }
    }

    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    pub const fn as_char(self) -> char {
        self as u8 as char
    }

    pub const fn payload_len(self) -> Option<usize> {
        match self {
            PacketType::ServerHeartbeat
            | PacketType::EndOfSession
            | PacketType::ClientHeartbeat
            | PacketType::LogoutRequest => Some(0),
            PacketType::LoginReject => Some(1),
            PacketType::LoginAccepted => Some(SESSION_ID_LEN + SEQUENCE_NUMBER_LEN),
            PacketType::LoginRequest => {
                Some(USERNAME_LEN + PASSWORD_LEN + SESSION_ID_LEN + SEQUENCE_NUMBER_LEN)
            }
            PacketType::Debug | PacketType::SequencedData | PacketType::UnsequencedData => None,
        }
    }
}

/*
impl fmt::Display for PacketType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            PacketType::Debug => write!(f, "debug"),
            LoginReject::LoginAccepted => write!(f, "login accepted"),
        }
    }
}
*/

/*
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PacketType(u8);

impl PacketType {
    // Debug is the packet type for Debug packets.
    const DEBUG: Self = Self(b'+');
    // LoginAccepted is the packet type for LoginAccepted packets.
    const LOGIN_ACCEPTED: Self = Self(b'A');
    // LoginReject is the packet type for LoginReject packets.
    const LOGIN_REJECT: Self = Self(b'J');
    // SequencedData is the packet type for SequencedData packets.
    const SEQUENCED_DATA: Self = Self(b'S');
    // UnsequencedData is the packet type for UnsequencedData packets.
    const UNSEQUENCED_DATA: Self = Self(b'U');
    // ServerHeartbeat is the packet type for ServerHeartbeat packets.
    const SERVER_HEARTBEAT: Self = Self(b'H');
    // EndOfSession is the packet type for EndOfSession packets.
    const END_OF_SESSION: Self = Self(b'Z');
    // LoginRequest is the packet type for LoginRequest packets.
    const LOGIN_REQUEST: Self = Self(b'L');
    // ClientHeartbeat is the packet type for ClientHeartbeat packets.
    const CLIENT_HEARTBEAT: Self = Self(b'R');
    // LogoutRequest is the packet type for LogoutRequest packets.
    const LOGOUT_REQUEST: Self = Self(b'O');
}
*/

/*
pub struct Packet {
    packet_type: u8,
    payload: Payload,
}
*/

#[derive(Debug)]
pub enum PacketParseError {
    InvalidPacketType(u8),
    UnexpectedPacketType { want: PacketType, got: PacketType, payload_len: usize },
    MismatchLen { want: usize, got: usize },
    BadPayload(Box<[u8]>),
    Io(IoError),
}

impl From<IoError> for PacketParseError {
    fn from(e: IoError) -> Self {
        PacketParseError::Io(e)
    }
}

impl fmt::Display for PacketParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PacketParseError::InvalidPacketType(b) => write!(f, "invalid packet type byte: {b}"),
            PacketParseError::UnexpectedPacketType { want, got, payload_len } => {
                write!(
                    f,
                    "expected packet type {want:?} ({}), got {got:?} ({}) (payload len: {payload_len})",
                    want.as_char(), got.as_char(),
                )
            }
            PacketParseError::MismatchLen { want, got } => {
                write!(f, "expected {want} bytes, got {got}")
            }
            // TODO: print payload?
            PacketParseError::BadPayload(ref p) => write!(f, "bad payload ({} bytes)", p.len()),
            PacketParseError::Io(ref e) => write!(f, "io error: {e}"),
        }
    }
}

impl Error for PacketParseError {}

#[repr(transparent)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Packet(Vec<u8>);

impl Packet {
    pub(crate) fn new(packet_type: PacketType, payload: Payload) -> Self {
        // Payload length + packet type + payload
        let mut bytes = Vec::with_capacity(2 + 1 + payload.len());
        // Need to include packet_type in length
        bytes.extend_from_slice(&(1 + payload.len() as u16).to_be_bytes());
        bytes.push(packet_type.as_u8());
        bytes.extend_from_slice(&payload);
        Self(bytes)
    }

    pub unsafe fn from_parts(packet_type: PacketType, payload: Payload) -> Self {
        Self::new(packet_type, payload)
    }

    pub unsafe fn from_bytes(b: Vec<u8>) -> Self {
        Self(b)
    }

    pub fn debug(payload: Payload) -> Self {
        Self::new(PacketType::Debug, payload)
    }

    pub fn login_accepted(session: SessionId, seq_num: SequenceNumber) -> Self {
        let mut payload = Payload::new([0u8; SESSION_ID_LEN + SEQUENCE_NUMBER_LEN]).unwrap();
        payload[..SESSION_ID_LEN].copy_from_slice(&session);
        payload[SESSION_ID_LEN..].copy_from_slice(&seq_num);
        Self::new(PacketType::LoginAccepted, payload)
    }

    pub fn login_reject(code: LoginReject) -> Self {
        Self::new(PacketType::LoginReject, Payload::new(vec![code.as_u8()]).unwrap())
    }

    pub fn sequenced_data(payload: Payload) -> Self {
        Self::new(PacketType::SequencedData, payload)
    }

    pub fn unsequenced_data(payload: Payload) -> Self {
        Self::new(PacketType::UnsequencedData, payload)
    }

    pub fn server_heartbeat() -> Self {
        Self::new(PacketType::ServerHeartbeat, Payload::default())
    }

    pub fn end_of_session() -> Self {
        Self::new(PacketType::EndOfSession, Payload::default())
    }

    pub fn login_request(
        username: Username,
        password: Password,
        session: SessionId,
        seq_num: SequenceNumber,
    ) -> Self {
        let mut payload = [0u8; USERNAME_LEN + PASSWORD_LEN + SESSION_ID_LEN + SEQUENCE_NUMBER_LEN];
        let mut slice = payload.as_mut_slice();
        slice[..USERNAME_LEN].copy_from_slice(&username);
        slice = &mut slice[USERNAME_LEN..];
        slice[..PASSWORD_LEN].copy_from_slice(&password);
        slice = &mut slice[PASSWORD_LEN..];
        slice[..SESSION_ID_LEN].copy_from_slice(&session);
        slice = &mut slice[SESSION_ID_LEN..];
        slice[..SEQUENCE_NUMBER_LEN].copy_from_slice(&seq_num);
        Self::new(PacketType::LoginRequest, Payload::new(payload).unwrap())
    }

    pub fn client_heartbeat() -> Self {
        Self::new(PacketType::ClientHeartbeat, Payload::default())
    }

    pub fn logout_request() -> Self {
        Self::new(PacketType::LogoutRequest, Payload::default())
    }

    pub fn parse(b: &[u8]) -> Result<Self, PacketParseError> {
        Self::read_from(&mut Cursor::new(b))
    }

    pub fn parse_as(pt: PacketType, b: &[u8]) -> Result<Self, PacketParseError> {
        Self::try_read_from_as(pt, &mut Cursor::new(b))
    }

    pub fn read_from<R: Read>(r: &mut R) -> Result<Self, PacketParseError> {
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
            Ok(payload) => Ok(Self::new(packet_type, payload)),
            Err(bytes) => Err(PacketParseError::BadPayload(bytes)),
        }
    }

    pub fn try_read_from_as<R: Read>(
        r: &mut R,
        want_pt: PacketType,
    ) -> Result<Self, PacketParseError> {
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
            Ok(payload) => Ok(Self::new(packet_type, payload)),
            Err(bytes) => Err(PacketParseError::BadPayload(bytes)),
        }
    }

    pub fn packet_type(&self) -> PacketType {
        // Ok since packet type can only be set with the PacketType enum
        PacketType::from_u8(self.0[2]).unwrap()
    }

    pub fn credentials(&self) -> Option<(Username, Password)> {
        self.username().zip(self.password())
    }

    pub fn credentials_slices(&self) -> Option<(&[u8], &[u8])> {
        self.username_slice().zip(self.password_slice())
    }

    pub fn username(&self) -> Option<Username> {
        self.username_slice().map(Username::new_trunc)
    }

    pub fn username_slice(&self) -> Option<&[u8]> {
        if self.packet_type() != PacketType::LoginRequest {
            return None;
        }
        self.payload().get(..USERNAME_LEN)
    }

    pub fn password(&self) -> Option<Password> {
        self.password_slice().map(Password::new_trunc)
    }

    pub fn password_slice(&self) -> Option<&[u8]> {
        if self.packet_type() != PacketType::LoginRequest {
            return None;
        }
        self.payload()
            .get(USERNAME_LEN..USERNAME_LEN + PASSWORD_LEN)
    }

    #[inline(always)]
    pub fn session(&self) -> Option<SessionId> {
        self.session_slice().map(SessionId::new_trunc)
    }

    #[inline(always)]
    pub fn session_slice(&self) -> Option<&[u8]> {
        match self.packet_type() {
            PacketType::LoginRequest => self.payload().get(16..26),
            PacketType::LoginAccepted => self.payload().get(..10),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn sequence_number(&self) -> Option<SequenceNumber> {
        self.sequence_number_slice().map(SequenceNumber::new_trunc)
    }

    #[inline(always)]
    pub fn sequence_number_slice(&self) -> Option<&[u8]> {
        match self.packet_type() {
            PacketType::LoginRequest => self.payload().get(26..46),
            PacketType::LoginAccepted => self.payload().get(10..30),
            _ => None,
        }
    }

    pub fn reject_reason(&self) -> Option<LoginReject> {
        if self.packet_type() != PacketType::LoginReject {
            return None;
        }
        // TODO: what to do with error
        LoginReject::from_u8(self.payload().get(0).copied().unwrap_or(0)).ok()
    }

    pub fn payload(&self) -> &[u8] {
        &self.0[3..]
    }

    pub fn payload_text(&self) -> Option<&str> {
        std::str::from_utf8(self.payload()).ok()
    }

    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }

    pub fn parts(&self) -> (PacketType, &[u8]) {
        (self.packet_type(), self.payload())
    }

    pub fn into_parts(mut self) -> (PacketType, Payload) {
        let pt = self.packet_type();
        (pt, Payload(self.0.split_off(3).into()))
    }
}

impl Deref for Packet {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        //self.0.as_slice()
        &self.0[2..]
    }
}

impl AsRef<[u8]> for Packet {
    fn as_ref(&self) -> &[u8] {
        //self.0.as_slice()
        &self.0[2..]
    }
}

impl Borrow<[u8]> for Packet {
    fn borrow(&self) -> &[u8] {
        //self.0.as_slice()
        &self.0[2..]
    }
}

// TODO: make struct  (in similar way to PacketType)?
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoginReject {
    // NotAuthorized is sent whenever a login request has invalid credentials.
    NotAuthorized = b'A',
    // SessionNotAvail is sent whenever a login request has an invalid session.
    SessionNotAvail = b'S',
}

impl LoginReject {
    fn from_u8(b: u8) -> Result<Self, u8> {
        match b {
            b'A' => Ok(LoginReject::NotAuthorized),
            b'S' => Ok(LoginReject::SessionNotAvail),
            _ => Err(b),
        }
    }

    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

impl fmt::Display for LoginReject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoginReject::NotAuthorized => write!(f, "not authorized"),
            LoginReject::SessionNotAvail => write!(f, "session not available"),
        }
    }
}
