// TODO: debug packets
// TODO: returned errors
package soupbintcp

import (
	"context"
	"fmt"
	"net"
	"sync/atomic"
	"time"

	utils "github.com/johnietre/utils/go"
)

const (
	// DefaultServerTimeout is the default timeout for not receiving anything
	// from the server, afterwhich, the connection is closed.
	DefaultServerTimeout time.Duration = time.Second * 15
)

// LoginRejctedError is an error based on a login rejection.
type LoginRejectedError struct {
	Reason LoginReject
}

// Error implements the Error method of the error interface.
func (lre *LoginRejectedError) Error() string {
	return fmt.Sprintf("login rejected (reason: %c)", lre.Reason)
}

// ClientHandler is a handler to handle packet receipt.
type ClientHandler = func(*Client, Packet)

const (
	// ClientHeartbeat is the interval at when a client sends heartbeat messages
	// if no message has been sent.
	ClientHeartbeat = time.Millisecond * 1000
)

var (
	// ErrLoggedOut just means that the client has been logged out without any
	// other error.
	ErrLoggedOut = fmt.Errorf("logged out")
	// ErrServerTimedOut means the server has timed out.
	ErrServerTimedOut = fmt.Errorf("server timed out")
)

// ConnectOpts are options to set for when connecting to a server.
type ConnectOpts struct {
	Session        SessionId
	SequenceNumber SequenceNumber
	Username       Username
	Password       Password
	ServerTimeout  time.Duration
	// The read and write deadline for the underlying conn for when connecting.
	Deadline     time.Time
	DebugHandler ClientHandler
}

// Client is a client to connect to a server.
type Client struct {
	laddr, raddr net.Addr
	// TODO: what to do when closing (when to set nil, etc.)
	readConn  *utils.Mutex[*net.TCPConn]
	writeConn *utils.Mutex[*net.TCPConn]
	opts      *ConnectOpts
	// FIXME: following error was received:
	// invalid use of type alias ClientHandler in recursive type (see issue #50729)
	//handler        ClientHandler
	// work around
	handler func(*Client, Packet)

	nextSeqNum  atomic.Uint64
	currSession SessionId

	clientHeartbeatTimer *time.Timer
	serverHeartbeatTimer *time.Timer

	closeErr *utils.AValue[utils.ErrorValue]
  // Is closed when the client is closed.
  closedChan chan utils.Unit
}

// Connect connects to a server, returning a client. See ConnectWithOpts for
// more details.
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

// ConnectWithOpts connects to a server with the given ConnectOpts. Passing
// with a nil handler will result in messages needing to be handled manually
// via Client.ReadPacket.
func ConnectWithOpts(
	addr string,
	handler ClientHandler,
	opts *ConnectOpts,
) (*Client, error) {
	if opts == nil {
		opts = &ConnectOpts{}
	}
	if opts.ServerTimeout == 0 {
		opts.ServerTimeout = DefaultServerTimeout
	}
	// TODO: return error? handle if not set (i.e., all zeros)?
	if !opts.SequenceNumber.IsValid() {
		opts.SequenceNumber = SequenceNumberZero()
	}

	conn, err := net.Conn(nil), error(nil)
	if opts.Deadline.IsZero() {
		conn, err = net.Dial("tcp", addr)
	} else {
		conn, err = net.DialTimeout("tcp", addr, opts.Deadline.Sub(time.Now()))
	}
	if err != nil {
		return nil, err
	}
	closeConn := utils.NewT(true)
	defer utils.DeferClose(closeConn, conn)

	if err := conn.SetDeadline(opts.Deadline); err != nil {
		return nil, err
	}
  sess := opts.Session
  if sess.Eq(SessionId{}) {
    sess = SessionIdBlank()
  }
  // TODO: is this necessary?
  seqNum := opts.SequenceNumber
  if seqNum.Eq(SequenceNumber{}) {
    seqNum = SequenceNumberZero()
  }
	packet := LoginRequestPacket(
		opts.Username, opts.Password,
		sess, seqNum,
	)
	if _, err := conn.Write(packet.Bytes()); err != nil {
		return nil, err
	}

	packet, err = ReadPacketFrom(conn)
	if err != nil {
		return nil, err
	}
	switch packet.PacketType() {
	case PacketTypeLoginAccepted:
	case PacketTypeLoginReject:
		reason, ok := packet.RejectReason()
		if !ok {
			return nil, &InvalidPacketError{
				Packet: packet,
				Reason: "missing reject reason",
			}
		}
		return nil, &LoginRejectedError{reason}
	default:
		return nil, &InvalidPacketError{
			Packet: packet,
			Reason: "invalid packet type",
		}
	}
	conn.SetDeadline(time.Time{})

	sessionId, ok := packet.SessionId()
	if !ok {
		return nil, &InvalidPacketError{
			Packet: packet,
			Reason: "missing session ID",
		}
	}
	nsn, ok := packet.SequenceNumber()
	if !ok {
		return nil, &InvalidPacketError{
			Packet: packet,
			Reason: "missing sequence number",
		}
	}
	nextSeqNum, ok := nsn.ToUint64Safe()
	if !ok {
		return nil, &InvalidPacketError{
			Packet: packet,
			Reason: "invalid sequence number",
		}
	}

	*closeConn = false
	tcpConn := conn.(*net.TCPConn)
	c := &Client{
		laddr:     tcpConn.LocalAddr(),
		raddr:     tcpConn.RemoteAddr(),
		readConn:  utils.NewMutex(tcpConn),
		writeConn: utils.NewMutex(tcpConn),
		opts:      opts,
		handler:   handler,

		currSession: sessionId,

		closeErr: &utils.AValue[utils.ErrorValue]{},
    closedChan: make(chan utils.Unit, 0),
	}

	c.nextSeqNum.Store(nextSeqNum)

	c.clientHeartbeatTimer = time.AfterFunc(ClientHeartbeat, c.heartbeat)
	c.serverHeartbeatTimer = time.AfterFunc(opts.ServerTimeout, func() {
		c.closeWithErr(ErrServerTimedOut)
		c.readConn.Apply(func(cp **net.TCPConn) {
			conn := *cp
			if conn != nil {
				conn.Close()
			}
		})
	})

	if c.handler != nil {
		if !c.startListenPackets() {
			panic("didn't start listening to packets")
		}
	}
	return c, nil
}

func (c *Client) resetClientHeartbeat() {
	if c.IsClosed() {
		return
	}
	c.clientHeartbeatTimer.Reset(ClientHeartbeat)
}

// ResetServerHeartbeat is used to reset the server heartbeat check whenever
// a message is received. This should only be called when the messages are
// handled manually. Otherwise, it will be called automatically while handling
// messages.
func (c *Client) ResetServerHeartbeat() {
	if c.IsClosed() {
		return
	}
	c.serverHeartbeatTimer.Reset(c.opts.ServerTimeout)
}

// NextSeqNum returns the next expected sequence number.
func (c *Client) NextSeqNum() uint64 {
	return c.nextSeqNum.Load()
}

// NextSequenceNumber returns the next expected sequence number as a
// SequenceNumber.
func (c *Client) NextSequenceNumber() SequenceNumber {
	return SequenceNumberFromUint64(c.NextSeqNum())
}

// IncrSequenceNumber is used to increment the sequence number when a sequenced
// packet is received. This should only be called when the messages are handled
// manually. Otherwise, it will be called automatically while handling
// messagses.
func (c *Client) IncrSequenceNumber() {
	// Messages are handled not handled manually
	if c.handler != nil {
		return
	}
  c.incrSequenceNumber()
}

func (c *Client) incrSequenceNumber() {
	c.nextSeqNum.Add(1)
}

// Handler returns the handler the client uses to handle messages.
func (c *Client) Handler() ClientHandler {
	return c.handler
}

func (c *Client) startListenPackets() bool {
	ptr := c.readConn.Lock()
	conn := *ptr
	*ptr = nil
	c.readConn.Unlock()
	if conn == nil {
		return false
	}
	handler := c.handler
	if handler == nil {
		return false
	}
	debugHandler := c.opts.DebugHandler
  nextSeqNum := c.NextSeqNum()
  nextPktCh := utils.NewAValue(make(chan utils.Unit, 0))
  close(nextPktCh.Load())
	go func() {
		for {
			packet, err := ReadPacketFrom(conn)
			if err != nil {
				conn.Close()
				c.closeWithErr(err)
				return
			}
			c.ResetServerHeartbeat()
			switch pt := packet.PacketType(); pt {
			case PacketTypeServerHeartbeat:
				continue
			case PacketTypeDebug:
				if debugHandler != nil {
					debugHandler(c, packet)
				}
				continue
			case PacketTypeSequencedData:
				//c.IncrSequenceNumber()
        nextSeqNum++
			case PacketTypeEndOfSession:
				conn.Close()
				c.closeWithErr(ErrSessionEnded)
        c.handler(c, packet)
				return
				// TODO: handle other (besides unsequenced)?
			}
			//go c.handler(c, packet)
			go func(seqNum uint64) {
        if packet.PacketType() == PacketTypeSequencedData {
          fmt.Println("SEQ NUM:", seqNum)
          if seqNum < c.NextSeqNum() {
            return
          }
          ch := nextPktCh.Load()
NextPktChLoop:
          for {
            select {
            case _, _ = <-ch:
              if seqNum == c.NextSeqNum() {
                ch = make(chan utils.Unit, 0)
                nextPktCh.Store(ch)
                break NextPktChLoop
              }
              ch = nextPktCh.Load()
            }
          }
          //fmt.Println("GOT:", seqNum)
          c.incrSequenceNumber()
          close(ch)
          /*
          fmt.Println("WAITING:", seqNum, c.NextSeqNum())
          for seqNum > c.NextSeqNum() {}
          c.incrSequenceNumber()
          fmt.Println("GOT:", seqNum, c.NextSeqNum())
          */
        }
        c.handler(c, packet)
      }(nextSeqNum-1)
		}
	}()
	return true
}

func (c *Client) sendPacket(packet Packet) error {
	if err := c.CloseErr(); err != nil {
		return err
	}
	c.resetClientHeartbeat()
	// TODO: Close?
	var err error
	c.writeConn.Apply(func(cp **net.TCPConn) {
		conn := *cp
		if conn == nil {
			err = c.CloseErr()
			return
		}
		_, err = conn.Write(packet.Bytes())
		if err != nil {
			err = c.closeWithErr(err)
			conn.Close()
			*cp = nil
		}
	})
	return err
}

// Returns false if the client is already listening for packets in a separate
// goroutine (i.e., a non-nil ClientHandler was passed when connecting). This
// should only be called when handling packets manually. This also means that
// heartbeats, sequenced, unsequenced, etc. packets must be handled manually.
// The only thing handled is an EndOfSession packet, which is still returned
// from this function, but  will automatically close the connection with
// ErrSessionEnded.
func (c *Client) ReadPacket() (packet Packet, err error, ok bool) {
	c.readConn.Apply(func(cp **net.TCPConn) {
		conn := *cp
		if conn == nil {
			return
		}
		ok = true
		packet, err = ReadPacketFrom(conn)
		if err != nil {
			err = c.closeWithErr(err)
			conn.Close()
			*cp = nil
		} else if packet.PacketType() == PacketTypeEndOfSession {
			c.closeWithErr(ErrSessionEnded)
			conn.Close()
			*cp = nil
		}
	})
	return
}

// SendUnsequenced sends an unsequenced packet.
func (c *Client) SendUnsequenced(payload Payload) error {
	return c.sendPacket(UnsequencedDataPacket(payload))
}

// Logout logs out (sends a logout packet) and closes the connection.
func (c *Client) Logout() error {
	err := c.sendPacket(LogoutRequestPacket())
	c.writeConn.Apply(func(cp **net.TCPConn) {
		conn := *cp
		if conn == nil {
			return
		}
		conn.Close()
		*cp = nil
	})
	if err == nil {
		err = ErrLoggedOut
	}
	return c.closeWithErr(err)
}

func (c *Client) LocalAddr() net.Addr {
	return c.laddr
}

func (c *Client) RemoteAddr() net.Addr {
	return c.raddr
}

func (c *Client) heartbeat() {
	c.sendPacket(ClientHeartbeatPacket())
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

// Wait waits for the client to be closed. If there is no done channel or the
// context is nil, the function returns immediately whether it is closed or
// not. Otherwise, it waits for closure or the context to be canceled,
// whichever is first.
func (c *Client) Wait(ctx context.Context) bool {
  _, ok := <-c.closedChan
  if ok {
    return true
  }
  if ctx == nil {
    return false
  }
  done := ctx.Done()
  if done == nil {
    return false
  }
  closed := false
  select {
  case _, _ = <-done:
  case _, _ = <- c.closedChan:
    closed = true
  }
  return closed
}

func (c *Client) closeWithErr(err error) error {
	if c.closeErr.StoreIfEmpty(utils.NewErrorValue(err)) {
    close(c.closedChan)
  }
	return c.closeErr.Load().Error
}
