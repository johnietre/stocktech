package moldudp64

import (
	"encoding/binary"
	"errors"
	"fmt"
	"net"
	"sync/atomic"
	"time"

	utils "github.com/johnietre/utils/go"
)

type timerPtr struct {
	ptr *time.Timer
}

type Transmitter struct {
	conn       *net.UDPConn
	session    [10]byte
	nextSeqNum atomic.Uint64

	heartbeatTimer *utils.AValue[timerPtr]
	heartbeatDur   atomic.Int64
}

func NewTransmitter(addr string, session [10]byte) (*Transmitter, error) {
	uaddr, err := net.ResolveUDPAddr("udp4", addr)
	if err != nil {
		return nil, err
	}
	conn, err := net.DialUDP("udp4", nil, uaddr)
	if err != nil {
		return nil, err
	}
	return &Transmitter{
		conn:           conn,
		session:        session,
		heartbeatTimer: utils.NewAValue(timerPtr{}),
	}, nil
}

func (t *Transmitter) SetHeartbeat(dur time.Duration) {
	t.heartbeatDur.Store(int64(dur))
	if dur == 0 {
		if old, _ := t.heartbeatTimer.Swap(timerPtr{}); old.ptr != nil {
			old.ptr.Stop()
		}
		return
	}
	if timer := t.heartbeatTimer.Load().ptr; timer != nil {
		timer.Reset(dur)
	} else {
		t.heartbeatTimer.Store(timerPtr{time.AfterFunc(dur, t.sendHeartbeat)})
	}
	return
}

func (t *Transmitter) SetHeartbeatDur(dur time.Duration) {
	old := t.heartbeatDur.Swap(int64(dur))
	if old == 0 {
		timer := time.AfterFunc(dur, t.sendHeartbeat)
		t.heartbeatTimer.Store(timerPtr{timer})
	} else if dur == 0 {
		if ptr, _ := t.heartbeatTimer.Swap(timerPtr{}); ptr.ptr != nil {
			ptr.ptr.Stop()
		}
	}
}

func (t *Transmitter) sendHeartbeat() {
	old := t.heartbeatTimer.Load().ptr
	// TODO: Error?
	t.SendHeartbeat()
	dur := time.Duration(t.heartbeatDur.Load())
	if dur == 0 || old == nil || t.heartbeatTimer.Load().ptr != old {
		return
	}
	old.Reset(dur)
}

func (t *Transmitter) Send(packet DownstreamPacket) error {
	b := packet.Serialize()
	// TODO: What to do when not all bytes sent
	_, err := t.conn.Write(b)
	return err
}

func (t *Transmitter) SendMessageBlocks(blocks []MessageBlock) error {
	l := uint64(len(blocks))
	packet, err := NewDownstreamPacket(
		t.session, t.nextSeqNum.Add(l)-l, blocks,
	)
	if err != nil {
		return err
	}
	return t.Send(packet)
}

func (t *Transmitter) SendHeartbeat() error {
	return t.Send(NewHeartbeatDownstreamPacket(t.session, t.nextSeqNum.Load()))
}

func (t *Transmitter) SendEndSession() error {
	return t.Send(NewEndSessionDownstreamPacket(t.session, t.nextSeqNum.Load()))
}

// DownstreamPacket is a packet sent "downstream" and received by MoldUDP64
// listeners. May contain a payload of 0+ stream messages.
type DownstreamPacket struct {
	// Header is the header for the packet.
	Header Header
	// MessageBlocks are the message blocks in the packet.
	MessageBlocks []MessageBlock
}

func NewDownstreamPacket(
	session [10]byte,
	seqNum uint64,
	blocks []MessageBlock,
) (DownstreamPacket, error) {
	l := len(blocks)
	if l > 0xFFFF {
		return DownstreamPacket{}, errors.New("number of blocks must be less than 0xFFFF (65535)")
	} else if l == 0xFFFF {
		return DownstreamPacket{}, errors.New("number of blocks must be less than 0xFFFF (65535)")
	}
	return DownstreamPacket{
		Header: Header{
			Session:        session,
			SequenceNumber: seqNum,
			MessageCount:   uint16(l),
		},
		MessageBlocks: blocks,
	}, nil
}

func NewHeartbeatDownstreamPacket(
	session [10]byte, seqNum uint64,
) DownstreamPacket {
	packet, _ := NewDownstreamPacket(session, seqNum, nil)
	return packet
}

func NewEndSessionDownstreamPacket(
	session [10]byte, seqNum uint64,
) DownstreamPacket {
	return DownstreamPacket{Header: EndSessionHeader(session, seqNum)}
}

func ParseDownstreamPacket(b []byte) (DownstreamPacket, error) {
	header, err := ParseHeader(b)
	if err != nil {
		return DownstreamPacket{}, err
	}
	if header.MessageCount == 0xFFFF {
		return DownstreamPacket{Header: header}, nil
	}
	b = b[20:]
	blocks := make([]MessageBlock, 0, header.MessageCount)
	for i := uint16(0); i < header.MessageCount; i++ {
		if len(b) < 2 {
			return DownstreamPacket{}, fmt.Errorf("not enough bytes to parse")
		}
		ml := binary.BigEndian.Uint16(b)
		b = b[2:]
		if len(b) < int(ml) {
			return DownstreamPacket{}, fmt.Errorf("not enough bytes to parse")
		}
		blocks = append(blocks, MessageBlock{
			MessageLen:  ml,
			MessageData: b[:ml],
		})
		b = b[ml:]
	}
	return DownstreamPacket{
		Header:        header,
		MessageBlocks: blocks,
	}, nil
}

func (dp DownstreamPacket) IsHeartbeat() bool {
	return dp.Header.IsHeartbeat()
}

func (dp DownstreamPacket) IsEndSession() bool {
	return dp.Header.IsEndSession()
}

func (dp DownstreamPacket) Serialize() []byte {
	l := 20
	for _, block := range dp.MessageBlocks {
		l += 2 + int(block.MessageLen)
	}
	b := make([]byte, 0, l)
	b = append(b, dp.Header.Serialize()...)
	for _, block := range dp.MessageBlocks {
		b = append(b, block.Serialize()...)
	}
	return b
}

// Header is a DownstreamPacket header.
type Header struct {
	// TODO
	// Session indicates the session to which this packet belongs.
	Session [10]byte
	// SequenceNumber is the sequence number of the first message in the packet.
	SequenceNumber uint64
	// MessageCount is the count of messages contained in this packet.
	MessageCount uint16
}

func HeartbeatHeader(session [10]byte, seqNum uint64) Header {
	return Header{
		Session:        session,
		SequenceNumber: seqNum,
		MessageCount:   0,
	}
}

func EndSessionHeader(session [10]byte, seqNum uint64) Header {
	return Header{
		Session:        session,
		SequenceNumber: seqNum,
		MessageCount:   0xFFFF,
	}
}

func ParseHeader(b []byte) (Header, error) {
	if len(b) < 20 {
		// TODO
		return Header{}, fmt.Errorf("expected at least 20 bytes")
	}
	var session [10]byte
	copy(session[:], b[:10])
	return Header{
		Session:        session,
		SequenceNumber: binary.BigEndian.Uint64(b[10:]),
		MessageCount:   binary.BigEndian.Uint16(b[18:]),
	}, nil
}

func (h Header) Serialize() []byte {
	b := make([]byte, 0, 20)
	b = append(b, h.Session[:]...)
	b = binary.BigEndian.AppendUint64(b, h.SequenceNumber)
	b = binary.BigEndian.AppendUint16(b, h.MessageCount)
	return b
}

func (h Header) IsHeartbeat() bool {
	return h.MessageCount == 0
}

func (h Header) IsEndSession() bool {
	return h.MessageCount == 0xFFFF
}

// MessageBlock is the actual block for a message.
type MessageBlock struct {
	// MessageLen indicates the length in bytes of the message contained in this
	// Message block.
	// Does not include the 2 bytes of the message length field.
	MessageLen uint16
	// MessageData is the message data.
	MessageData []byte
}

var ErrMsgTooLong = errors.New("message data too long")

func NewMessageBlock(data []byte) (MessageBlock, error) {
	l := len(data)
	if l > 0xFFFF {
		return MessageBlock{}, ErrMsgTooLong
	}
	return MessageBlock{
		MessageLen:  uint16(l),
		MessageData: data,
	}, nil
}

func (mb MessageBlock) Serialize() []byte {
	b := make([]byte, 0, 2+int(mb.MessageLen))
	b = binary.BigEndian.AppendUint16(b, mb.MessageLen)
	return append(b, mb.MessageData...)
}

type Receiver struct {
	conn *net.UDPConn
	// Addresses of Request Servers
	requestAddrs []*net.UDPAddr

	// TODO: Session and sequence number of next expected message.
	session    [10]byte
	nextSeqNum uint64

	closedChan chan struct{}
	err        *utils.AValue[utils.ErrorValue]
}

type PacketHandler = func(DownstreamPacket)

func NewReceiver(
	smcgroup string,
	requestServers []*net.UDPAddr,
	packetHandler PacketHandler,
	session [10]byte,
) (*Receiver, error) {
	mcgroup, err := net.ResolveUDPAddr("udp4", smcgroup)
	if err != nil {
		return nil, err
	}
	conn, err := net.ListenMulticastUDP("udp4", nil, mcgroup)
	if err != nil {
		return nil, err
	}
	// TODO: Set Read Buffer
	rcvr := &Receiver{
		conn:         conn,
		requestAddrs: requestServers,
		session:      session,
		nextSeqNum:   0,
		closedChan:   make(chan struct{}),
		err:          &utils.AValue[utils.ErrorValue]{},
	}
	go runRcvr(rcvr, packetHandler)
	return rcvr, nil
}

func (r *Receiver) Err() error {
	err, _ := r.err.LoadSafe()
	return err.Error
}

func (r *Receiver) ClosedChan() <-chan struct{} {
	return r.closedChan
}

func (r *Receiver) IsClosed() bool {
	select {
	case _, _ = <-r.closedChan:
		return true
	default:
		return false
	}
}

func runRcvr(rcvr *Receiver, packetHandler PacketHandler) {
	// TODO: Msg len
	for {
		b := make([]byte, 4096)
		n, _, err := rcvr.conn.ReadFromUDP(b)
		if err != nil {
			rcvr.err.StoreIfEmpty(utils.NewErrorValue(err))
			break
		}
		packet, err := ParseDownstreamPacket(b[:n])
		if err != nil {
			rcvr.err.StoreIfEmpty(utils.NewErrorValue(err))
			break
		}
		if !utils.SliceEq(packet.Header.Session[:], rcvr.session[:]) {
			// TODO
			rcvr.err.StoreIfEmpty(utils.NewErrorValue(
				fmt.Errorf(
					"expected session %v, got %v",
					rcvr.session, packet.Header.Session,
				),
			))
			break
		}
		if packet.IsHeartbeat() {
			// TODO
			next := packet.Header.SequenceNumber
			if rcvr.nextSeqNum != next {
				rcvr.err.StoreIfEmpty(utils.NewErrorValue(
					fmt.Errorf(
						"expected sequence number %d, got %d",
						rcvr.nextSeqNum, packet.Header.SequenceNumber,
					),
				))
				break
			}
			println("heartbeat")
			continue
		} else if packet.IsEndSession() {
			// TODO
			next := packet.Header.SequenceNumber
			if rcvr.nextSeqNum != next {
				rcvr.err.StoreIfEmpty(utils.NewErrorValue(
					fmt.Errorf(
						"expected sequence number %d, got %d",
						rcvr.nextSeqNum, packet.Header.SequenceNumber,
					),
				))
				break
			}
			break
		} else if rcvr.nextSeqNum == 0 {
			// TODO
			rcvr.nextSeqNum = packet.Header.SequenceNumber
		}
		if packet.Header.SequenceNumber != rcvr.nextSeqNum {
			// TODO
			rcvr.err.StoreIfEmpty(utils.NewErrorValue(
				fmt.Errorf(
					"expected sequence number %d, got %d",
					rcvr.nextSeqNum, packet.Header.SequenceNumber,
				),
			))
			break
		}
		rcvr.nextSeqNum += uint64(packet.Header.MessageCount)
		packetHandler(packet)
	}
	close(rcvr.closedChan)
}

// RequestPacket is the packet MoldUDP64 clients send to request
// retransmission of message(s) from a Re-request server.
type RequestPacket struct {
	// Session indicates the session to which this packet belongs.
	Session [10]byte
	// SequenceNumber is the first requested sequence number.
	SequenceNumber uint64
	// RequestedMessageCount is the number of messages requested for
	// retransmission.
	RequestedMessageCount uint16
}
