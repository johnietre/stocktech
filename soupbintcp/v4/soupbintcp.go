package soupbintcp

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"io"
	"net"
  "sync/atomic"
  "time"

  utils "github.com/johnietre/utils/go"
)

const (
	// PacketTypeDebug is the packet type for Debug packets.
	PacketTypeDebug byte = '+'
	// PacketTypeLoginAccepted is the packet type for LoginAccepted packets.
	PacketTypeLoginAccepted byte = 'A'
	// PacketTypeLoginReject is the packet type for LoginReject packets.
	PacketTypeLoginReject byte = 'J'
	// PacketTypeSequencedData is the packet type for SequencedData packets.
	PacketTypeSequencedData byte = 'S'
	// PacketTypeUnsequencedData is the packet type for UnsequencedData packets.
	PacketTypeUnsequencedData byte = 'U'
	// PacketTypeServerHeartbeat is the packet type for ServerHeartbeat packets.
	PacketTypeServerHeartbeat byte = 'H'
	// PacketTypeEndOfSession is the packet type for EndOfSession packets.
	PacketTypeEndOfSession byte = 'Z'
	// PacketTypeLoginRequest is the packet type for LoginRequest packets.
	PacketTypeLoginRequest byte = 'L'
	// PacketTypeClientHeartbeat is the packet type for ClientHeartbeat packets.
	PacketTypeClientHeartbeat byte = 'R'
	// PacketTypeLogoutRequest is the packet type for LogoutRequest packets.
	PacketTypeLogoutRequest byte = 'O'
)

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

// Packet is a packet to be sent.
type Packet struct {
	// PacketType is the packet type.
	PacketType byte
	// Payload is the packet's payload.
	Payload Payload
}

// ParsePacket parses a packet from the given bytes.
func ParsePacket(b []byte) (Packet, error) {
	return ReadPacketFrom(bytes.NewReader(b))
}

// ReadPacketFrom reads a packet from the reader.
func ReadPacketFrom(r io.Reader) (Packet, error) {
	return ReadPacketFromWithBuf(r, bytes.NewBuffer(nil))
}

// ReadPacketFrom reads a packet from the reader by reading into the given
// buffer.
func ReadPacketFromWithBuf(r io.Reader, buf *bytes.Buffer) (Packet, error) {
	if buf == nil {
		buf = bytes.NewBuffer(nil)
	}
	if n, err := io.CopyN(buf, r, 3); err != nil {
		return Packet{}, err
	} else if n != 3 {
		return Packet{}, fmt.Errorf("not enough bytes")
	}
	packetLen := int(binary.BigEndian.Uint16(buf.Next(2))) - 1
	if packetLen < 0 {
		return Packet{}, fmt.Errorf("invalid packet length")
	}
	packetType := buf.Next(1)[0]
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
			Packet{
				PacketType: packetType,
			},
		}
	}
	if packetLen < wantLen {
		return Packet{}, &MismatchPacketLenError{Want: wantLen, Got: packetLen}
	}
	if n, err := io.CopyN(buf, r, int64(wantLen)); err != nil {
		return Packet{}, err
	} else if n != int64(wantLen) {
		// TODO: Try read more or return different error?
		return Packet{}, fmt.Errorf("not enough bytes")
	}
	return Packet{
		PacketType: packetType,
		// TODO: Do better?
		Payload: Payload(bytes.Clone(buf.Bytes())),
	}, nil
}

// DebugPacket creates a new Debug packet.
func DebugPacket(payload Payload) Packet {
	return Packet{
		PacketType: PacketTypeDebug,
		Payload:    payload,
	}
}

// LoginAcceptedPacket creates a new LoginAccepted packet.
func LoginAcceptedPacket(session Session, seqNum SequenceNumber) Packet {
	// TODO: Padding?
	payload := make([]byte, 0, 30)
  payload = append(payload, session[:]...)
  payload = append(payload, seqNum[:]...)
	return Packet{
		PacketType: PacketTypeLoginAccepted,
		Payload:    Payload(payload),
	}
}

const (
	// LoginRejectNotAuthorized is sent whenever a login request has invalid
	// credentials.
	LoginRejectNotAuthorized byte = 'A'
	// LoginRejectSessionNotAvail is sent whenever a login request has an invalid
	// session.
	LoginRejectSessionNotAvail byte = 'S'
)

// LoginRejectPacket creates a new LoginReject packet.
func LoginRejectPacket(code byte) Packet {
	// TODO: Check code?
	return Packet{
		PacketType: PacketTypeLoginReject,
		Payload:    Payload{code},
	}
}

// SequencedDataPacket creates a new SequencedData packet.
func SequencedDataPacket(payload Payload) Packet {
	return Packet{
		PacketType: PacketTypeSequencedData,
		Payload:    payload,
	}
}

// UnsequencedDataPacket creates a new UnsequencedData packet.
func UnsequencedDataPacket(payload Payload) Packet {
	return Packet{
		PacketType: PacketTypeUnsequencedData,
		Payload:    payload,
	}
}

// ServerHeartbeatPacket creates a new ServerHeartbeat packet.
func ServerHeartbeatPacket() Packet {
	return Packet{PacketType: PacketTypeServerHeartbeat}
}

// EndOfSessionPacket creates a new EndOfSession packet.
func EndOfSessionPacket() Packet {
	return Packet{PacketType: PacketTypeEndOfSession}
}

// LoginRequestPacket creates a new LoginRequest packet.
func LoginRequestPacket(
	username Username,
	password Password,
	session Session,
	seqNum SequenceNumber,
) Packet {
	payload := make([]byte, 0, 6+10+10+20)
  payload = append(payload, username[:]...)
  payload = append(payload, password[:]...)
  payload = append(payload, session[:]...)
  payload = append(payload, seqNum[:]...)
	return Packet{
		PacketType: PacketTypeLoginRequest,
		Payload:    Payload(payload),
	}
}

// ClientHeartbeatPacket creates a new ClientHeartbeat packet.
func ClientHeartbeatPacket() Packet {
	return Packet{PacketType: PacketTypeClientHeartbeat}
}

// LogoutRequestPacket creates a new LogoutRequest packet.
func LogoutRequestPacket() Packet {
	return Packet{PacketType: PacketTypeLogoutRequest}
}

// Serialize serializes the packet so that it can be written to the wire.
func (pkt Packet) Serialize() []byte {
	l := len(pkt.Payload)
	if l > MaxPayloadLen {
		l = MaxPayloadLen
	}
	// TODO: Do better
	buf := make([]byte, 0, l+2)
	buf = binary.BigEndian.AppendUint16(buf, uint16(l))
	return append(buf, pkt.Payload...)
}

// Credentials returns the username and password in the packet's payload.
// Returns false if the packet type is not LoginRequest, or if the payload
// isn't long enough.
func (p Packet) Credentials() (username Username, password Password, ok bool) {
	if p.PacketType != PacketTypeLoginRequest {
		return
	} else if len(p.Payload) < 16 {
		return
	}
	username = UsernameFromBytesTrunc(p.Payload[:6])
	password = PasswordFromBytesTrunc(p.Payload[6:16])
	return username, password, true
}

// Session returns the session in the packet's payload. Returns false if the
// packet type is not LoginRequest or LoginAccepted, or if the payload isn't
// long enough.
func (p Packet) Session() (Session, bool) {
	switch p.PacketType {
	case PacketTypeLoginRequest:
		if len(p.Payload) < 26 {
			return Session{}, false
		}
		return SessionFromBytesTrunc(p.Payload[16:26]), true
	case PacketTypeLoginAccepted:
		if len(p.Payload) < 10 {
			return Session{}, false
		}
		return SessionFromBytesTrunc(p.Payload[:10]), true
	default:
		return Session{}, false
	}
}

// SequenceNumber returns the sequence number in the packet's payload. Returns
// false is the packet type is not LoginRequest or LoginAccepted, or if the
// payload isn't long enough.
func (p Packet) SequenceNumber() (SequenceNumber, bool) {
	switch p.PacketType {
	case PacketTypeLoginRequest:
		if len(p.Payload) < 46 {
			return SequenceNumber{}, false
		}
		return SequenceNumberFromBytesTrunc(p.Payload[26:46]), true
	case PacketTypeLoginAccepted:
		if len(p.Payload) < 10 {
			return SequenceNumber{}, false
		}
		return SequenceNumberFromBytesTrunc(p.Payload[10:30]), true
	default:
		return SequenceNumber{}, false
	}
}

// RejectReason returns the reason for a login rejection. Returns false if the
// packet type is not LoginReject or if the length of the payload is less than
// 1.
func (p Packet) RejectReason() (byte, bool) {
	if p.PacketType != PacketTypeLoginReject || len(p.Payload) < 1 {
		return 0, false
	}
	return p.Payload[0], true
}

// PayloadText returns the payload as a string.
func (p Packet) PayloadText() string {
	return string(p.Payload)
}

// LoginRejctedError is an error based on a login rejection.
type LoginRejectedError struct {
	Reason byte
}

// Error implements the Error method of the error interface.
func (lre *LoginRejectedError) Error() string {
	return fmt.Sprintf("login rejected (reason: %c)", lre.Reason)
}

// IvalidPacketError represents when an invalid packet is received/sent.
type InvalidPacketError struct {
	Packet Packet
}

// Error implements the Error method of the error interface.
func (ipe *InvalidPacketError) Error() string {
	return fmt.Sprintf(
		"invalid packet received (packet type: %c, payload len: %d)",
		ipe.Packet.PacketType, len(ipe.Packet.Payload),
	)
}

// ClientHandler is a handler to handle packet receipt.
type ClientHandler = func(Packet)

const (
	heartbeatDur = time.Millisecond * 1001
)

var (
	// ErrLoggedOut just means that the client has been logged out without any
	// other error.
	ErrLoggedOut = fmt.Errorf("logged out")
)

// ConnectOpts are options to set for when connecting to a server.
type ConnectOpts struct {
	Session        Session
	SequenceNumber SequenceNumber
	Username       Username
	Password       Password
	// The read and write deadline for the underlying conn for when connecting.
	Deadline time.Time
}

// Client is a client to connect to a server.
type Client struct {
	conn           net.Conn
	opts           *ConnectOpts
	handler        ClientHandler
	heartbeatTimer *time.Timer
	closeErr       *utils.AValue[utils.ErrorValue]
}

// Connect connects to a server, returning a client.
func Connect(
	addr string,
	username Username,
	password Password,
	handler ClientHandler,
) (*Client, error) {
	return ConnectWithOpts(
		addr,
    handler,
		&ConnectOpts{
			Username: username,
			Password: password,
		},
	)
}

// ConnectWithOpts connects to a server with the given ConnectOpts.
func ConnectWithOpts(
	addr string,
	handler ClientHandler,
	opts *ConnectOpts,
) (*Client, error) {
	if opts == nil {
		opts = &ConnectOpts{}
	}
	conn, err := net.Dial("tcp", addr)
	if err != nil {
		return nil, err
	}
	closeConn := utils.NewT(true)
	defer func() {
		if *closeConn {
			conn.Close()
		}
	}()
	if err := conn.SetDeadline(opts.Deadline); err != nil {
		return nil, err
	}
  packet := LoginRequestPacket(
    opts.Username, opts.Password, opts.Session, opts.SequenceNumber,
  )
	if _, err := conn.Write(packet.Serialize()); err != nil {
		return nil, err
	}
	buf := make([]byte, 30)
	if _, err := conn.Read(buf); err != nil {
		return nil, err
	}
	packet, err = ParsePacket(buf)
	if err != nil {
		return nil, err
	}
	switch packet.PacketType {
	case 'A':
	case 'J':
		return nil, &LoginRejectedError{packet.Payload[0]}
	default:
		return nil, &InvalidPacketError{packet}
	}
	conn.SetDeadline(time.Time{})
	*closeConn = false
	c := &Client{
		conn:     conn,
		opts:     opts,
		handler:  handler,
		closeErr: &utils.AValue[utils.ErrorValue]{},
	}
	c.heartbeatTimer = time.AfterFunc(heartbeatDur, c.heartbeat)
	go c.listenPackets()
	return c, nil
}

func (c *Client) listenPackets() {
	for {
		packet, err := ReadPacketFrom(c.conn)
		if err != nil {
			c.closeErr.StoreIfEmpty(utils.NewErrorValue(err))
			return
		}
		go c.handler(packet)
	}
}

func (c *Client) sendPacket(packet Packet) error {
	if err := c.CloseErr(); err != nil {
		return err
	}
	c.resetHeartbeat()
	// TODO: Close?
	_, err := c.conn.Write(packet.Serialize())
	return err
}

// SendUnsequenced sends an unsequenced packet.
func (c *Client) SendUnsequenced(payload Payload) error {
	return c.sendPacket(UnsequencedDataPacket(payload))
}

// Logout logs out (sends a logout packet) and closes the connection.
func (c *Client) Logout() error {
	err := c.sendPacket(LogoutRequestPacket())
	c.conn.Close()
	if err == nil {
		c.closeErr.StoreIfEmpty(utils.NewErrorValue(ErrLoggedOut))
	} else {
		c.closeErr.StoreIfEmpty(utils.NewErrorValue(err))
	}
	return err
}

func (c *Client) heartbeat() {
	c.sendPacket(ClientHeartbeatPacket())
}

func (c *Client) resetHeartbeat() {
	c.heartbeatTimer.Reset(heartbeatDur)
}

// CloseErr gets the reason for the client being closed, if there is one.
// A successful close is represented by ErrLoggedOut.
func (c *Client) CloseErr() error {
	ev, _ := c.closeErr.LoadSafe()
	return ev.Error
}

// IsClosed returns whether the client is closed or not.
func (c *Client) IsClosed() bool {
	return c.CloseErr() != nil
}

type ClientConn struct {
  srvr *Server
  conn net.Conn
	heartbeatTimer *time.Timer
}

func (s *Server) newClientConn(conn net.Conn) *ClientConn {
  cc := &ClientConn{srvr: s, conn: conn}
	cc.heartbeatTimer = time.AfterFunc(heartbeatDur, cc.heartbeat)
	return cc
}

func (cc *ClientConn) run() {
  // TODO: Any error/bad packet handling?
  for {
    packet, err := ReadPacketFrom(cc.conn)
    if err != nil {
      break
    }
    switch packet.PacketType {
    case PacketTypeUnsequencedData:
      go cc.srvr.Handler(cc, packet)
    case PacketTypeClientHeartbeat:
      // TODO
      continue
    case PacketTypeLogoutRequest:
      break
    default:
      break
    }
  }
  // TODO: Is any sort of synchronization needed?
  cc.srvr.clients.Remove(cc)
  cc.conn.Close()
}

// SendUnsequenced sends an unsequenced packet to the client.
func (cc *ClientConn) SendUnsequenced(payload Payload) error {
  return cc.sendPacket(UnsequencedDataPacket(payload))
}

func (cc *ClientConn) sendPacket(packet Packet) error {
	return cc.sendBytes(packet.Serialize())
}

func (cc *ClientConn) sendBytes(b []byte) error {
	cc.resetHeartbeat()
	_, err := cc.conn.Write(b)
	return err
}

func (cc *ClientConn) heartbeat() {
	cc.sendPacket(ServerHeartbeatPacket())
}

func (cc *ClientConn) resetHeartbeat() {
	cc.heartbeatTimer.Reset(heartbeatDur)
}

type ServerHandler = func(*ClientConn, Packet)

// Server is a server to serve clients (duh).
type Server struct {
	// Addr is the address to run the server on.
	Addr string
	// Session is the session the server is running.
	Session Session
	// Username is the username used to authenticate clients.
	Username Username
	// Password is the password used to authenticate clients.
	Password Password
  // ServerHandler is the handler for any packet received from a client.
  Handler ServerHandler

  ln net.Listener
  seqNum atomic.Uint64
	clients  *utils.SyncSet[*ClientConn]
	closeErr *utils.AValue[utils.ErrorValue]
}

// Run runs the server.
func (s *Server) Run() error {
  var err error
	if s.clients != nil {
		err = s.CloseErr()
		if err == nil {
			err = fmt.Errorf("already running")
		}
		return err
	}
  if s.Handler == nil {
    s.Handler = func(*ClientConn, Packet) {}
  }
	s.clients = utils.NewSyncSet[*ClientConn]()
  s.closeErr = &utils.AValue[utils.ErrorValue]{}
  s.ln, err = net.Listen("tcp", s.Addr)
  if err != nil {
    s.closeErr.Store(utils.NewErrorValue(err))
    return err
  }
	for {
		conn, err := s.ln.Accept()
		if err != nil {
			return err
		}
		go s.handleLogin(conn)
	}
}

// CloseErr gets the reason for the server being closed, if there is one.
// A successful close is represented by ErrLoggedOut.
func (s *Server) CloseErr() error {
  ev, _ := s.closeErr.LoadSafe()
  return ev.Error
}

// IsClosed returns whether the server is closed or not.
func (s *Server) IsClosed() bool {
  return s.CloseErr() == nil
}

func (s *Server) handleLogin(conn net.Conn) {
	// TODO: Error handling
	closeConn := utils.NewT(true)
	defer func() {
		if *closeConn {
			conn.Close()
		}
	}()
	buf := make([]byte, 46)
	if n, err := conn.Read(buf[:3]); err != nil {
		return
	} else if n != 3 {
		return
	} else if buf[2] != PacketTypeLoginRequest {
		return
	} else if binary.BigEndian.Uint16(buf) != 47 {
		return
	}
	if n, err := conn.Read(buf); err != nil {
		return
	} else if n != 46 {
		return
	}
	packet := Packet{
		PacketType: PacketTypeLoginRequest,
		Payload:    Payload(buf),
	}
	username, password, _ := packet.Credentials()
  session, _ := packet.Session()
  // TODO: Sequence number
  if !bytes.EqualFold(username[:], s.Username[:]) ||
    !bytes.EqualFold(password[:], s.Password[:]) {
    packet = LoginRejectPacket(LoginRejectNotAuthorized)
    conn.Write(packet.Serialize())
    return
  } else if !bytes.Equal(session[:], s.Session[:]) {
    packet = LoginRejectPacket(LoginRejectSessionNotAvail)
    conn.Write(packet.Serialize())
    return
  }
  packet = LoginAcceptedPacket(s.Session, s.nextSeqNum())
  if _, err := conn.Write(packet.Serialize()); err != nil {
    return
  }
  *closeConn = false
  cc := s.newClientConn(conn)
  go cc.run()
  s.clients.Insert(cc)
}

func (s *Server) nextSeqNum() SequenceNumber {
  // TODO
  return SequenceNumber{}
}

func (s *Server) sendPacket(packet Packet) error {
	if err := s.CloseErr(); err != nil {
		return err
	}
	// TODO: Close?
	packetBytes := packet.Serialize()
	s.clients.Range(func(cc *ClientConn) bool {
		if err := cc.sendBytes(packetBytes); err != nil {
			// TODO: Close?
			s.clients.Remove(cc)
		}
		return true
	})
  return nil
}

// SendSequenced sends a sequenced packet to all clients.
func (s *Server) SendSequenced(payload Payload) error {
  err := s.sendPacket(SequencedDataPacket(payload))
  if err != nil {
    s.seqNum.Add(1)
  }
  return err
}

// SendUnsequenced sends an unsequenced packet to all clients.
func (s *Server) SendUnsequenced(payload Payload) error {
	if err := s.CloseErr(); err != nil {
		return err
	}
	return s.sendPacket(UnsequencedDataPacket(payload))
}

var (
  ErrSessionEnded = fmt.Errorf("session ended")
)

// EndSessionAndClose ends the session (sends EndOfSession messages) and closes
// the server.
// TODO: How long to wait to close clients.
func (s *Server) EndSessionAndClose() error {
	err := s.sendPacket(EndOfSessionPacket())
	if err == nil {
		s.closeErr.StoreIfEmpty(utils.NewErrorValue(ErrSessionEnded))
	} else {
		s.closeErr.StoreIfEmpty(utils.NewErrorValue(err))
	}
  // TODO: Is this the right thing here?
  if s.ln != nil {
    s.ln.Close()
  }
	return err
}
