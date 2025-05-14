use std::borrow::Borrow;
use std::ops::Deref;

const SESSION_ID_LEN: usize = 10;

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

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.0))
    }
}

/*
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Header {
    /// The session to which the packet belongs.
    pub session: SessionId,
    /// The sequence number of the first message of the packet.
    pub sequence_number: u64,
    /// The count of messages contained in this packet.
    pub message_count: u16,
}

impl Header {
    pub fn heartbeat() -> Self {
        // TODO
        Self {
            session: [0u8; 10],
            sequence_number: 0,
            message_count: 0,
        }
    }

    pub fn end_session() -> Self {
        // TODO
        Self {
            session: [0u8; 10],
            sequence_number: 0,
            message_count: 0xFFFF,
        }
    }

    pub fn parse(b: &[u8]) -> Result<Self, ()> {
        if b.len() != 20 {
            return Err(());
        }
        Ok(Self {
            session: (&b[..10]).try_into().unwrap(),
            sequence_number: u64::from_be_bytes((&b[10..18]).try_into().unwrap()),
            message_count: u16::from_be_bytes((&b[18..]).try_into().unwrap()),
        })
    }

    pub fn serialize(&self) -> [u8; 20] {
        let mut res = [0u8; 20];
        res[..10].copy_from_slice(&self.session);
        res[10..10+8].copy_from_slice(&self.sequence_number.to_be_bytes());
        res[10+8..].copy_from_slice(&self.message_count.to_be_bytes());
        res
    }

    pub fn is_heartbeat(&self) -> bool {
        self.message_count == 0
    }

    pub fn is_end_session(&self) -> bool {
        self.message_count == 0xFFFF
    }
}
*/

#[derive(Clone)]
pub struct Header([u8; 20]);

impl Header {
    pub fn new(session: SessionId, seq_num: u64, msg_count: u16) -> Self {
        let mut header = Self([0u8; 20]);
        header.set_session(session);
        header.set_sequence_number(seq_num);
        header.set_message_count(msg_count);
        header
    }

    pub fn heartbeat(session: SessionId, next_seq_num: u64) -> Self {
        Self::new(session, next_seq_num, 0)
    }

    pub fn end_session(session: SessionId, next_seq_num: u64) -> Self {
        Self::new(session, next_seq_num, 0xFFFF)
    }

    pub fn parse(b: &[u8]) -> Result<Self, ()> {
        b.try_into().map_err(|_| ()).map(Self)
    }

    pub fn session(&self) -> SessionId {
        SessionId(*slice_byte_arr::<20, 0, SESSION_ID_LEN>(&self.0))
    }

    pub fn sequence_number(&self) -> u64 {
        u64::from_be_bytes(*slice_byte_arr::<20, SESSION_ID_LEN, 8>(&self.0))
    }

    pub fn message_count(&self) -> u16 {
        u16::from_be_bytes(*slice_byte_arr::<20, { SESSION_ID_LEN + 8 }, 2>(&self.0))
    }

    pub fn set_session(&mut self, id: SessionId) {
        *slice_byte_arr_mut::<20, 0, SESSION_ID_LEN>(&mut self.0) = id.0
    }

    pub fn set_sequence_number(&mut self, n: u64) {
        *slice_byte_arr_mut::<20, SESSION_ID_LEN, 8>(&mut self.0) = n.to_be_bytes();
    }

    pub fn set_message_count(&mut self, n: u16) {
        *slice_byte_arr_mut::<20, { SESSION_ID_LEN + 8 }, 2>(&mut self.0) = n.to_be_bytes();
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn is_heartbeat(&self) -> bool {
        self.message_count() == 0
    }

    pub fn is_end_session(&self) -> bool {
        self.message_count() == 0xFFFF
    }
}

pub struct DownstreamPacket {
    header: Header,
    message_blocks: Vec<MessageBlock>,
}

impl DownstreamPacket {
    pub fn new(mut header: Header, message_blocks: Vec<MessageBlock>) -> Result<Self, ()> {
        let l = message_blocks.len();
        if l >= 0xFFFF {
            // TODO
        }
        header.set_message_count(l as _);
        Ok(Self { header, message_blocks })
    }

    pub fn heartbeat() -> Self {
        Self {
            header: Header::heartbeat(),
            message_blocks: Vec::new(),
        }
    }

    pub fn end_session() -> Self {
        Self {
            header: Header::end_session(),
            message_blocks: Vec::new(),
        }
    }

    pub fn parse(mut b: &[u8]) -> Result<Self, ()> {
        if b.len() < 20 {
            return Err(());
        }
        let header = Header::parse(b)?;
        if header.is_heartbeat() || header.is_end_session() {
            return Ok(Self {
                header,
                message_blocks: Vec::new(),
            });
        }
        b = &b[20..];
        let mut message_blocks = Vec::with_capacity(header.message_count() as usize);
        for _ in 0..header.message_count() as usize {
            if b.len() < 2 {
                return Err(todo!());
            }
            let ml = u16::from_be_bytes(b[..2].try_into().unwrap()) as usize;
            if b.len() < ml {
                return Err(todo!());
            }
            /*
            b = &b[2..];
            message_blocks.push(MessageBlock {
                message_len: ml as u16,
                message_data: b[..ml].to_vec(),
            });
            */
            message_blocks.push(MessageBlock(b[..2 + ml].to_vec()));
            b = &b[2 + ml..];
        }
        Ok(Self { header, message_blocks })
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut res = self.header.as_slice().to_vec();
        for block in &self.message_blocks {
            res.extend_from_slice(block.as_slice());
        }
        res
    }
}

#[derive(Clone)]
pub struct MessageBlock(Vec<u8>);

impl MessageBlock {
    pub fn new(mut data: Vec<u8>) -> Result<Self, Vec<u8>> {
        let l = data.len();
        if l > 0xFFFF {
            return Err(data);
        }
        let mut v = Vec::with_capacity(2 + data.len());
        v.extend_from_slice(&(l as u16).to_be_bytes());
        v.append(&mut data);
        Ok(Self(data))
    }

    pub fn len(&self) -> usize {
        self.len_u16() as _
    }

    pub fn len_u16(&self) -> u16 {
        ((self.0[0] as u16) << 8) | (self.0[1] as u16)
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn into_inner(Self(mut v): Self) -> Vec<u8> {
        v.split_off(2)
    }
}

impl Deref for MessageBlock {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0[2..]
    }
}

impl AsRef<[u8]> for MessageBlock {
    fn as_ref(&self) -> &[u8] {
        &self.0[2..]
    }
}

impl Borrow<[u8]> for MessageBlock {
    fn borrow(&self) -> &[u8] {
        &self.0[2..]
    }
}

/*
pub struct RequestPacket {
    pub session: SessionId,
    pub sequence_number: u64,
    pub requested_message_count: u16,
}
*/

#[derive(Clone)]
pub struct RequestPacket([u8; 20]);

impl RequestPacket {
    pub fn new(session: SessionId, seq_num: u64, msg_count: u16) -> Self {
        let mut pkt = Self([0u8; 20]);
        pkt.set_session(session);
        pkt.set_sequence_number(seq_num);
        pkt.set_message_count(msg_count);
        pkt
    }

    pub fn session(&self) -> SessionId {
        SessionId(*slice_byte_arr::<20, 0, SESSION_ID_LEN>(&self.0))
    }

    pub fn sequence_number(&self) -> u64 {
        u64::from_be_bytes(*slice_byte_arr::<20, SESSION_ID_LEN, 8>(&self.0))
    }

    pub fn message_count(&self) -> u16 {
        u16::from_be_bytes(*slice_byte_arr::<20, { SESSION_ID_LEN + 8 }, 2>(&self.0))
    }

    pub fn set_session(&mut self, id: SessionId) {
        *slice_byte_arr_mut::<20, 0, SESSION_ID_LEN>(&mut self.0) = id.0
    }

    pub fn set_sequence_number(&mut self, n: u64) {
        *slice_byte_arr_mut::<20, SESSION_ID_LEN, 8>(&mut self.0) = n.to_be_bytes();
    }

    pub fn set_message_count(&mut self, n: u16) {
        *slice_byte_arr_mut::<20, { SESSION_ID_LEN + 8 }, 2>(&mut self.0) = n.to_be_bytes();
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

fn slice_byte_arr<const N: usize, const START: usize, const LEN: usize>(
    bytes: &[u8; N],
) -> &[u8; LEN] {
    assert!(START + LEN <= N, "length exceeds end");
    // SAFETY:
    // Byte arrays cannot be misaligned and won't be reading past the length of the array as
    // checked by the assert above.
    unsafe { &*(bytes as *const [u8; N]).add(START).cast::<[u8; LEN]>() }
}

fn slice_byte_arr_mut<const N: usize, const START: usize, const LEN: usize>(
    bytes: &mut [u8; N],
) -> &mut [u8; LEN] {
    assert!(START + LEN <= N, "length exceeds end");
    // SAFETY:
    // Byte arrays cannot be misaligned and won't be reading past the length of the array as
    // checked by the assert above.
    unsafe { &mut *(bytes as *mut [u8; N]).add(START).cast::<[u8; LEN]>() }
}
