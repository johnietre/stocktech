// TODO: returned errors
package soupbintcp

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"io"

	utils "github.com/johnietre/utils/go"
)

const (
	// MaxPayloadLen is the maximum payload length. It's 65534 because the
	// maximum message length is 65535 and 1 byte is needed to encode the packet
	// type.
	MaxPayloadLen = 0xFFFE

	// UsernameLen is the length of a username.
	UsernameLen = 6
	// PasswordLen is the length of a password.
	PasswordLen = 10
	// SessionIdLen is the length of a session.
	SessionIdLen = 10
	// SequenceNumberLen is the length of a sequence number.
	SequenceNumberLen = 20
)

var (
	// ErrPayloadTooLarge is self explanatory.
	ErrPayloadTooLarge = fmt.Errorf("payload too large")
)

// Payload is the payload of a packet. It must be at most MaxPayloadLen bytes
// long. Behaviour when too long may vary, but as of right now, it is
// truncated.
type Payload []byte

// NewPayload returns a new payload wrapping the given bytes. Returns an error
// if the payload is too long.
func NewPayload(b []byte) (Payload, error) {
	if len(b) > MaxPayloadLen {
		return nil, ErrPayloadTooLarge
	}
	return Payload(b), nil
}

// PayloadFromString creates a new payload from a string. See NewPayload for
// more details.
func PayloadFromString(s string) (Payload, error) {
	return NewPayload([]byte(s))
}

// Len returns the length of the payload.
func (p Payload) Len() int {
	return len(p)
}

// Username is a username sent when logging in.
type Username [UsernameLen]byte

// UsernameFromString creates a new username from the given string. Returns an
// error if the username is too long (longer than 6 bytes). If it is shorter,
// space (' ') bytes are padded on the right to fill the rest.
func UsernameFromString(s string) (Username, error) {
	username := Username{}
	if len(s) > UsernameLen {
		return username, fmt.Errorf("username too long")
	}
	n := copy(username[:], s)
	for ; n < UsernameLen; n++ {
		username[n] = ' '
	}
	return username, nil
}

// UsernameFromBytes functions the same as UsernameFromString.
func UsernameFromBytes(b []byte) (Username, error) {
	username := Username{}
	if len(b) > UsernameLen {
		return username, fmt.Errorf("username too long")
	}
	n := copy(username[:], b)
	for ; n < UsernameLen; n++ {
		username[n] = ' '
	}
	return username, nil
}

// UsernameFromStringTrunc functions the same as UsernameFromString, but
// truncates the string input if it's too long.
func UsernameFromStringTrunc(s string) Username {
	username := Username{}
	n := copy(username[:], s)
	for ; n < UsernameLen; n++ {
		username[n] = ' '
	}
	return username
}

// UsernameFromStringTrunc functions the same as UsernameFromBytesTrunc.
func UsernameFromBytesTrunc(b []byte) Username {
	username := Username{}
	n := copy(username[:], b)
	for ; n < UsernameLen; n++ {
		username[n] = ' '
	}
	return username
}

// IsValid returns whether the username is valid or not.
func (u Username) IsValid() bool {
	inSpaces := false
	for _, b := range u {
		if b == ' ' {
			inSpaces = true
			continue
		}
		if inSpaces || !isAsciiAlphaNumeric(b) {
			return false
		}
	}
	return true
}

// Eq checks whether the username is equal to another (case-insensitive).
func (u Username) Eq(other Username) bool {
	return bytes.EqualFold(u[:], other[:])
}

// String converts the username into a string.
func (u Username) String() string {
	return string(u[:])
}

// Password is a password sent when logging in.
type Password [PasswordLen]byte

// PasswordFromString creates a new password from the given string. Returns an
// error if the password is too long (longer than 10 bytes). If it is shorter,
// space (' ') bytes are padded on the right to fill the rest.
func PasswordFromString(s string) (Password, error) {
	password := Password{}
	if len(s) > PasswordLen {
		return password, fmt.Errorf("password too long")
	}
	n := copy(password[:], s)
	for ; n < PasswordLen; n++ {
		password[n] = ' '
	}
	return password, nil
}

// PasswordFromBytes functions the same as PasswordFromString.
func PasswordFromBytes(b []byte) (Password, error) {
	password := Password{}
	if len(b) > PasswordLen {
		return password, fmt.Errorf("password too long")
	}
	n := copy(password[:], b)
	for ; n < PasswordLen; n++ {
		password[n] = ' '
	}
	return password, nil
}

// PasswordFromStringTrunc functions the same as PasswordFromString, but
// truncates the string input if it's too long.
func PasswordFromStringTrunc(s string) Password {
	password := Password{}
	n := copy(password[:], s)
	for ; n < PasswordLen; n++ {
		password[n] = ' '
	}
	return password
}

// PasswordFromBytesTrunc functions the same as PasswordFromStringTrunc.
func PasswordFromBytesTrunc(b []byte) Password {
	password := Password{}
	n := copy(password[:], b)
	for ; n < PasswordLen; n++ {
		password[n] = ' '
	}
	return password
}

// IsValid returns whether the password is valid or not.
func (p Password) IsValid() bool {
	inSpaces := false
	for _, b := range p {
		if b == ' ' {
			inSpaces = true
			continue
		}
		if inSpaces || !isAsciiAlphaNumeric(b) {
			return false
		}
	}
	return true
}

// Eq checks whether the password is equal to another (case-insensitive).
func (p Password) Eq(other Password) bool {
	return bytes.EqualFold(p[:], other[:])
}

// String converts the password into a string.
func (p Password) String() string {
	return string(p[:])
}

// SessionId represents the session on a server.
type SessionId [SessionIdLen]byte

// SessionIdFromBytes creates a new session from the given bytes. Returns an
// error if the session is too long (longer than 10 bytes). If it is shorter,
// space (' ') bytes are padded on the left to fill the rest.
func SessionIdFromBytes(b []byte) (SessionId, error) {
	session, l := SessionId{}, len(b)
	if l > SessionIdLen {
		return session, fmt.Errorf("session ID too long")
	}
	start := 0
	if l < SessionIdLen {
		start = SessionIdLen - l
	}
	copy(session[start:], b)
	for i := 0; i < start; i++ {
		session[i] = ' '
	}
	return session, nil
}

// SessionIdFromString is the same as SessionIdFromBytes but takes a string.
func SessionIdFromString(s string) (SessionId, error) {
	session, l := SessionId{}, len(s)
	if l > SessionIdLen {
		return session, fmt.Errorf("session ID too long")
	}
	start := 0
	if l < SessionIdLen {
		start = SessionIdLen - l
	}
	copy(session[start:], s)
	for i := 0; i < start; i++ {
		session[i] = ' '
	}
	return session, nil
}

// SessionIdFromBytesTrunc functions the same as SessionIdFromBytes, but
// truncates the bytes input if it's too long.
func SessionIdFromBytesTrunc(b []byte) SessionId {
	session, l := SessionId{}, len(b)
	start := 0
	if l < SessionIdLen {
		start = SessionIdLen - l
	}
	copy(session[start:], b)
	for i := 0; i < start; i++ {
		session[i] = ' '
	}
	return session
}

// SessionIdFromStringTrunc is the same as SessionIdFromBytesTrunc but takes
// a string.
func SessionIdFromStringTrunc(s string) SessionId {
	session, l := SessionId{}, len(s)
	start := 0
	if l < SessionIdLen {
		start = SessionIdLen - l
	}
	copy(session[start:], s)
	for i := 0; i < start; i++ {
		session[i] = ' '
	}
	return session
}

// SessionIdBlank returns a new blank session ID.
func SessionIdBlank() SessionId {
	return [SessionIdLen]byte{' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '}
}

// SessionIdBlankPtr returns a pointer to a new blank session ID.
func SessionIdBlankPtr() *SessionId {
	id := SessionIdBlank()
	return &id
}

// IsBlank returns whether the session ID is blank.
func (si SessionId) IsBlank() bool {
	blank := SessionIdBlank()
	return bytes.Equal(si[:], blank[:])
}

// Equal tests for equality against another SessionId.
func (si SessionId) Equal(other SessionId) bool {
	return bytes.Equal(si[:], other[:])
}

// IsValid returns whether the session ID is a valid one.
func (si SessionId) IsValid() bool {
	inSpaces := true
	for _, b := range si {
		if b == ' ' {
			if !inSpaces {
				return false
			}
			continue
		}
		if !isAsciiAlphaNumeric(b) {
			return false
		}
		inSpaces = false
	}
	return true
}

// Eq checks whether the session ID is equal to another (case-sensitive).
func (si SessionId) Eq(other SessionId) bool {
	return bytes.Equal(si[:], other[:])
}

// String converts the session ID into a string.
func (si SessionId) String() string {
	return string(si[:])
}

// SequenceNumber for sequenced data messages.
type SequenceNumber [SequenceNumberLen]byte

// SequenceNumberFromBytes creates a new session from the given bytes. Returns
// an error if the sequence number is too long (longer than 20 bytes). If it is
// shorter, space (' ') bytes are padded on the left to fill the rest.
func SequenceNumberFromBytes(b []byte) (SequenceNumber, error) {
	seqNum, l := SequenceNumber{}, len(b)
	if l > SequenceNumberLen {
		return seqNum, fmt.Errorf("session ID too long")
	}
	start := 0
	if l < SequenceNumberLen {
		start = SequenceNumberLen - l
	}
	copy(seqNum[start:], b)
	for i := 0; i < start; i++ {
		seqNum[i] = ' '
	}
	if seqNum[SequenceNumberLen-1] == ' ' {
		seqNum[SequenceNumberLen-1] = '0'
	}
	return seqNum, nil
}

// SequenceNumberFromString functions the same as SequenceNumberFromBytes.
func SequenceNumberFromString(s string) (SequenceNumber, error) {
	seqNum, l := SequenceNumber{}, len(s)
	if l > SequenceNumberLen {
		return seqNum, fmt.Errorf("session ID too long")
	}
	start := 0
	if l < SequenceNumberLen {
		start = SequenceNumberLen - l
	}
	copy(seqNum[start:], s)
	for i := 0; i < start; i++ {
		seqNum[i] = ' '
	}
	if seqNum[SequenceNumberLen-1] == ' ' {
		seqNum[SequenceNumberLen-1] = '0'
	}
	return seqNum, nil
}

// SequenceNumberFromBytesTrunc functions the same as SequenceNumberFromBytes,
// but truncates the bytes input if it's too long.
func SequenceNumberFromBytesTrunc(b []byte) SequenceNumber {
	seqNum, l := SequenceNumber{}, len(b)
	start := 0
	if l < SequenceNumberLen {
		start = SequenceNumberLen - l
	}
	copy(seqNum[start:], b)
	for i := 0; i < start; i++ {
		seqNum[i] = ' '
	}
	if seqNum[SequenceNumberLen-1] == ' ' {
		seqNum[SequenceNumberLen-1] = '0'
	}
	return seqNum
}

// SequenceNumberFromStringTrunc functions the same as
// SequenceNumberFromBytesTrunc.
func SequenceNumberFromStringTrunc(s string) SequenceNumber {
	seqNum, l := SequenceNumber{}, len(s)
	start := 0
	if l < SequenceNumberLen {
		start = SequenceNumberLen - l
	}
	copy(seqNum[start:], s)
	for i := 0; i < start; i++ {
		seqNum[i] = ' '
	}
	if seqNum[SequenceNumberLen-1] == ' ' {
		seqNum[SequenceNumberLen-1] = '0'
	}
	return seqNum
}

// SequenceNumberZero returns the sequence number for zero.
func SequenceNumberZero() SequenceNumber {
	return [SequenceNumberLen]byte{
		' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
		' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', '0',
	}
}

// SequenceNumberFromUint64 returns the sequence number from the uint64.
func SequenceNumberFromUint64(u uint64) SequenceNumber {
	seqNum := SequenceNumberZero()
	for i := SequenceNumberLen - 1; i >= 0; i-- {
		if u == 0 {
			break
		}
		seqNum[i] = byte(u%10) + '0'
		u /= 10
	}
	return seqNum
}

// ToUint64 returns the sequence number as a uint64, ignoring invalid bytes.
func (sn SequenceNumber) ToUint64() uint64 {
	n := uint64(0)
	for i := 0; i < SequenceNumberLen; i++ {
		b := sn[i]
		if b >= '0' && b <= '9' {
			n = n*10 + uint64(b-'0')
		}
	}
	return n
}

// ToUint64Safe returns the sequence number as a uint64, returning true as well
// if it is valid, otherwise, returns false.
func (sn SequenceNumber) ToUint64Safe() (uint64, bool) {
	// TODO: return invalid on all spaces?
	n, inNum := uint64(0), false
	for i := 0; i < SequenceNumberLen; i++ {
		b := sn[i]
		if b >= '0' && b <= '9' {
			inNum = true
			n = n*10 + uint64(b-'0')
		} else if b == ' ' {
			if inNum {
				return n, false
			}
		} else {
			return n, false
		}
	}
	return n, true
}

// Eq checks whether the sequence number is equal to another.
func (sn SequenceNumber) Eq(other SequenceNumber) bool {
	eq := bytes.Equal(sn[:], other[:])
	if !eq {
		n1, ok := sn.ToUint64Safe()
		if !ok {
			return false
		}
		n2, ok := other.ToUint64Safe()
		if !ok {
			return false
		}
		eq = n1 == n2
	}
	return eq
}

// IsValid returns true if the sequence number is valid.
func (sn SequenceNumber) IsValid() bool {
	return utils.Second(sn.ToUint64Safe())
}

type PacketType byte

const (
	// PacketTypeDebug is the packet type for Debug packets.
	PacketTypeDebug PacketType = '+'
	// PacketTypeLoginAccepted is the packet type for LoginAccepted packets.
	PacketTypeLoginAccepted PacketType = 'A'
	// PacketTypeLoginReject is the packet type for LoginReject packets.
	PacketTypeLoginReject PacketType = 'J'
	// PacketTypeSequencedData is the packet type for SequencedData packets.
	PacketTypeSequencedData PacketType = 'S'
	// PacketTypeUnsequencedData is the packet type for UnsequencedData packets.
	PacketTypeUnsequencedData PacketType = 'U'
	// PacketTypeServerHeartbeat is the packet type for ServerHeartbeat packets.
	PacketTypeServerHeartbeat PacketType = 'H'
	// PacketTypeEndOfSession is the packet type for EndOfSession packets.
	PacketTypeEndOfSession PacketType = 'Z'
	// PacketTypeLoginRequest is the packet type for LoginRequest packets.
	PacketTypeLoginRequest PacketType = 'L'
	// PacketTypeClientHeartbeat is the packet type for ClientHeartbeat packets.
	PacketTypeClientHeartbeat PacketType = 'R'
	// PacketTypeLogoutRequest is the packet type for LogoutRequest packets.
	PacketTypeLogoutRequest PacketType = 'O'
)

func (pt PacketType) String() string {
	switch pt {
	case PacketTypeDebug:
		return "Debug"
	case PacketTypeLoginAccepted:
		return "LoginAccepted"
	case PacketTypeLoginReject:
		return "LoginReject"
	case PacketTypeSequencedData:
		return "SequencedData"
	case PacketTypeUnsequencedData:
		return "UnsequencedData"
	case PacketTypeServerHeartbeat:
		return "ServerHeartbeat"
	case PacketTypeEndOfSession:
		return "EndOfSession"
	case PacketTypeLoginRequest:
		return "LoginRequest"
	case PacketTypeClientHeartbeat:
		return "ClientHeartbeat"
	case PacketTypeLogoutRequest:
		return "LogoutRequest"
	default:
		return "Unknown"
	}
}

// MismatchPacketLenError is returned when the serialized packet length is not
// equal to what the the packet length based on the packet type.
type MismatchPacketLenError struct {
	Want, Got int
}

// Error implements the Error method of the error interface.
func (mple *MismatchPacketLenError) Error() string {
	return fmt.Sprintf(
		"expected packet length of %d, got %d",
		mple.Want, mple.Got,
	)
}

/*
// Packet is a packet to be sent.
type Packet struct {
	// PacketType is the packet type.
	PacketType byte
	// Payload is the packet's payload.
	Payload Payload
}
*/

// Packet is a packet to be sent.
type Packet struct {
	inner []byte
}

func newPacket(pt PacketType, payload Payload) Packet {
	inner := make([]byte, 2+1+len(payload))
	binary.BigEndian.PutUint16(inner, uint16(1+len(payload)))
	inner[2] = byte(pt)
	copy(inner[3:], payload)
	return Packet{inner: inner}
}

// ParsePacket parses a packet from the given bytes.
func ParsePacket(b []byte) (Packet, error) {
	return ReadPacketFrom(bytes.NewReader(b))
}

// ReadPacketFrom reads a packet from the reader.
func ReadPacketFrom(r io.Reader) (Packet, error) {
	return ReadPacketFromWithBuf(r, bytes.NewBuffer(nil))
}

var (
	ErrInvalidPacketLen = fmt.Errorf("invalid packet len")
)

// ReadPacketFrom reads a packet from the reader by reading into the given
// buffer.
func ReadPacketFromWithBuf(r io.Reader, buf *bytes.Buffer) (Packet, error) {
	if buf == nil {
		buf = bytes.NewBuffer(nil)
	}
	if n, err := io.CopyN(buf, r, 3); err != nil {
		return Packet{}, err
	} else if n != 3 {
		return Packet{}, io.ErrUnexpectedEOF
	}
	// TODO: check to make sure length isn't greater than max?
	packetLen := int(binary.BigEndian.Uint16(buf.Next(2))) - 1
	if packetLen < 0 {
		return Packet{}, &MismatchPacketLenError{Want: 1, Got: 0}
	}
	packetType := PacketType(buf.Next(1)[0])
	wantLen := 0
	switch packetType {
	case PacketTypeServerHeartbeat, PacketTypeEndOfSession,
		PacketTypeClientHeartbeat, PacketTypeLogoutRequest:
		wantLen = 0
	case PacketTypeLoginReject:
		wantLen = 1
	case PacketTypeLoginAccepted:
		wantLen = 30
	case PacketTypeLoginRequest:
		wantLen = 46
	case PacketTypeDebug, PacketTypeSequencedData, PacketTypeUnsequencedData:
		wantLen = packetLen
	default:
		return Packet{}, &InvalidPacketError{
			Packet: newPacket(packetType, Payload{}),
			Reason: "invalid packet type",
		}
	}
	if packetLen < wantLen {
		return Packet{}, &MismatchPacketLenError{Want: wantLen, Got: packetLen}
	}
	if n, err := io.CopyN(buf, r, int64(wantLen)); err != nil {
		return Packet{}, err
	} else if n != int64(wantLen) {
		// TODO: Try read more or return different error?
		return Packet{}, io.ErrUnexpectedEOF
	}
	return newPacket(packetType, Payload(utils.CloneSlice(buf.Bytes()))), nil
}

// UnexpectedPacketTypeError is returned when an unexpected packet type is
// received/parsed.
type UnexpectedPacketTypeError struct {
	Want, Got  PacketType
	PayloadLen int
}

// Error implements the Error method of the error interface.
func (upte *UnexpectedPacketTypeError) Error() string {
	return fmt.Sprintf(
		"expected packet type of %s, got %s (payload len: %d)",
		upte.Want, upte.Got, upte.PayloadLen,
	)
}

// TryParsePacketAs attempts to parse a packet with the given packet type.
// See TryReadPacketFromAs for more details.
func TryParsePacketAs(b []byte, want PacketType) (Packet, error) {
	return TryReadPacketFromAs(bytes.NewBuffer(b), want)
}

// TryReadPacketFromAs attempts to read a packet with the given packet
// type.
func TryReadPacketFromAs(r io.Reader, want PacketType) (Packet, error) {
	buf := bytes.NewBuffer(nil)
	if n, err := io.CopyN(buf, r, 3); err != nil {
		return Packet{}, err
	} else if n != 3 {
		return Packet{}, io.ErrUnexpectedEOF
	}
	payloadLen := int(binary.BigEndian.Uint16(buf.Next(2))) - 1
	if payloadLen < 0 {
		return Packet{}, &MismatchPacketLenError{Want: 1, Got: 0}
	}
	packetType := PacketType(buf.Next(1)[0])
	if packetType != want {
		return Packet{}, &UnexpectedPacketTypeError{
			Want:       want,
			Got:        packetType,
			PayloadLen: payloadLen,
		}
	}
	wantLen := 0
	switch packetType {
	case PacketTypeServerHeartbeat, PacketTypeEndOfSession,
		PacketTypeClientHeartbeat, PacketTypeLogoutRequest:
		wantLen = 0
	case PacketTypeLoginReject:
		wantLen = 1
	case PacketTypeLoginAccepted:
		wantLen = 30
	case PacketTypeLoginRequest:
		wantLen = 46
	case PacketTypeDebug, PacketTypeSequencedData, PacketTypeUnsequencedData:
		wantLen = payloadLen
	default:
		return Packet{}, &InvalidPacketError{
			Packet: newPacket(packetType, Payload{}),
			Reason: "invalid packet type",
		}
	}
	if payloadLen < wantLen {
		return Packet{}, &MismatchPacketLenError{Want: wantLen, Got: payloadLen}
	}
	if n, err := io.CopyN(buf, r, int64(wantLen)); err != nil {
		return Packet{}, err
	} else if n != int64(wantLen) {
		// TODO: Try read more or return different error?
		return Packet{}, io.ErrUnexpectedEOF
	}
	return newPacket(packetType, Payload(utils.CloneSlice(buf.Bytes()))), nil
}

// DebugPacket creates a new Debug packet.
func DebugPacket(payload Payload) Packet {
	return newPacket(PacketTypeDebug, payload)
}

// LoginAcceptedPacket creates a new LoginAccepted packet.
func LoginAcceptedPacket(session SessionId, seqNum SequenceNumber) Packet {
	payload := make([]byte, 0, 30)
	payload = append(payload, session[:]...)
	payload = append(payload, seqNum[:]...)
	return newPacket(PacketTypeLoginAccepted, Payload(payload))
}

type LoginReject byte

const (
	// LoginRejectNotAuthorized is sent whenever a login request has invalid
	// credentials.
	LoginRejectNotAuthorized LoginReject = 'A'
	// LoginRejectSessionNotAvail is sent whenever a login request has an invalid
	// session.
	LoginRejectSessionNotAvail LoginReject = 'S'
)

// LoginRejectPacket creates a new LoginReject packet.
func LoginRejectPacket(code LoginReject) Packet {
	// TODO: Check code?
	return newPacket(PacketTypeLoginReject, Payload{byte(code)})
}

// SequencedDataPacket creates a new SequencedData packet.
func SequencedDataPacket(payload Payload) Packet {
	return newPacket(PacketTypeSequencedData, payload)
}

// UnsequencedDataPacket creates a new UnsequencedData packet.
func UnsequencedDataPacket(payload Payload) Packet {
	return newPacket(PacketTypeUnsequencedData, payload)
}

// ServerHeartbeatPacket creates a new ServerHeartbeat packet.
func ServerHeartbeatPacket() Packet {
	return newPacket(PacketTypeServerHeartbeat, Payload{})
}

// EndOfSessionPacket creates a new EndOfSession packet.
func EndOfSessionPacket() Packet {
	return newPacket(PacketTypeEndOfSession, Payload{})
}

// LoginRequestPacket creates a new LoginRequest packet.
func LoginRequestPacket(
	username Username,
	password Password,
	session SessionId,
	seqNum SequenceNumber,
) Packet {
	payload := make(
		[]byte,
		0,
		UsernameLen+PasswordLen+SessionIdLen+SequenceNumberLen,
	)
	payload = append(payload, username[:]...)
	payload = append(payload, password[:]...)
	payload = append(payload, session[:]...)
	payload = append(payload, seqNum[:]...)
	return newPacket(PacketTypeLoginRequest, Payload(payload))
}

// ClientHeartbeatPacket creates a new ClientHeartbeat packet.
func ClientHeartbeatPacket() Packet {
	return newPacket(PacketTypeClientHeartbeat, Payload{})
}

// LogoutRequestPacket creates a new LogoutRequest packet.
func LogoutRequestPacket() Packet {
	return newPacket(PacketTypeLogoutRequest, Payload{})
}

// PacketType returns the packet type the packet.
func (pkt Packet) PacketType() PacketType {
	return PacketType(pkt.inner[2])
}

// Payload returns the payload of the packet.
// SHOULD NOT BE MODIFIED.
func (pkt Packet) Payload() []byte {
	return pkt.inner[3:]
}

// Serialize serializes the packet so that it can be written to the wire.
func (pkt Packet) Bytes() []byte {
	return pkt.inner
}

// Credentials returns the username and password in the packet's payload.
// Returns false if the packet type is not LoginRequest, or if the payload
// isn't long enough.
func (p Packet) Credentials() (username Username, password Password, ok bool) {
	username, ok = p.Username()
	if ok {
		password, ok = p.Password()
	}
	return
}

// Username returns the username in the packet's payload. Returns false if
// the packet type is not LoginRequest or if the payload isn't long enough.
func (p Packet) Username() (username Username, ok bool) {
	payload := p.Payload()
	if p.PacketType() != PacketTypeLoginRequest || len(payload) < UsernameLen {
		return
	}
	return UsernameFromBytesTrunc(p.Payload()[:UsernameLen]), true
}

// Password returns the password in the packet's payload. Returns false if
// the packet type is not LoginRequest or if the payload isn't long enough.
func (p Packet) Password() (password Password, ok bool) {
	payload := p.Payload()
	if p.PacketType() != PacketTypeLoginRequest || len(payload) < UsernameLen+PasswordLen {
		return
	}
	return PasswordFromBytesTrunc(
		p.Payload()[UsernameLen : UsernameLen+PasswordLen],
	), true
}

// SessionId returns the session in the packet's payload. Returns false if the
// packet type is not LoginRequest or LoginAccepted, or if the payload isn't
// long enough.
func (p Packet) SessionId() (SessionId, bool) {
	switch p.PacketType() {
	case PacketTypeLoginRequest:
		if len(p.Payload()) < 26 {
			return SessionId{}, false
		}
		return SessionIdFromBytesTrunc(p.Payload()[16:26]), true
	case PacketTypeLoginAccepted:
		if len(p.Payload()) < SessionIdLen {
			return SessionId{}, false
		}
		return SessionIdFromBytesTrunc(p.Payload()[:SessionIdLen]), true
	default:
		return SessionId{}, false
	}
}

// SequenceNumber returns the sequence number in the packet's payload. Returns
// false is the packet type is not LoginRequest or LoginAccepted, or if the
// payload isn't long enough.
func (p Packet) SequenceNumber() (SequenceNumber, bool) {
	switch p.PacketType() {
	case PacketTypeLoginRequest:
		if len(p.Payload()) < 46 {
			return SequenceNumber{}, false
		}
		return SequenceNumberFromBytesTrunc(p.Payload()[26:46]), true
	case PacketTypeLoginAccepted:
		if len(p.Payload()) < 30 {
			return SequenceNumber{}, false
		}
		return SequenceNumberFromBytesTrunc(p.Payload()[10:30]), true
	default:
		return SequenceNumber{}, false
	}
}

// RejectReason returns the reason for a login rejection. Returns false if the
// packet type is not LoginReject or if the length of the payload is less than
// 1.
func (p Packet) RejectReason() (LoginReject, bool) {
	if p.PacketType() != PacketTypeLoginReject || len(p.Payload()) < 1 {
		return 0, false
	}
	// TODO: do something if invalid
	return LoginReject(p.Payload()[0]), true
}

// PayloadText returns the payload as a string.
func (p Packet) PayloadText() string {
	return string(p.Payload())
}

// IvalidPacketError represents when an invalid packet is received/sent.
type InvalidPacketError struct {
	Packet Packet
	Reason string
}

// Error implements the Error method of the error interface.
func (ipe *InvalidPacketError) Error() string {
	if ipe.Reason == "" {
		return fmt.Sprintf(
			"invalid packet received (packet type: %c, payload len: %d)",
			ipe.Packet.PacketType(), len(ipe.Packet.Payload()),
		)
	}
	return fmt.Sprintf(
		"invalid packet received (packet type: %c, payload len: %d), reason: %s",
		ipe.Packet.PacketType(), len(ipe.Packet.Payload()), ipe.Reason,
	)
}

func isAsciiAlphaNumeric(b byte) bool {
	return (b >= '0' && b <= '9') ||
		(b >= 'A' && b <= 'Z') ||
		(b >= 'a' && b <= 'z')
}
