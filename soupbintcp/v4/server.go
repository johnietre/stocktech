// TODO: chan sizes
// TODO: debug packets
package soupbintcp

import (
	"bytes"
	"fmt"
	"net"
	"reflect"
	"sync/atomic"
	"time"

	utils "github.com/johnietre/utils/go"
)

var (
	globalTick     = newGlobalTicker()
	globalSessions = utils.NewSyncSet[*Session]()
)

func init() {
	globalTick.add(func(t time.Time) {
		globalSessions.Range(func(sess *Session) bool {
			// TODO
			return sess.sendHeartbeat() != nil
		})
	})
	globalTick.start()
}

const (
	ServerHeartbeat      = time.Second
	DefaultClientTimeout = time.Second * 15
)

var (
	ErrSlowClient     = fmt.Errorf("slow client")
	ErrClientTimedout = fmt.Errorf("client timed out")
	// TODO: make struct?
	ErrSessionEnded    = fmt.Errorf("session ended")
	ErrClosed          = fmt.Errorf("closed")
	ErrClientLoggedOut = fmt.Errorf("client logged out")
)

type sequencedPacket struct {
	// TODO: make SequenceNumber?
	// Zero is treated as invalid (not sequenced)
	seqNum uint64
	packet Packet
}

type SessionHandler = func(*SessionClient, Packet)

type SessionClient struct {
	session atomic.Pointer[Session]

	conn       *net.TCPConn
	packetChan *utils.RWMutex[chan sequencedPacket]

	startSeqNum uint64

	// Second-precision timestamp
	lastHeartbeat atomic.Int64
	timeout       time.Duration

	closeErr *utils.AValue[utils.ErrorValue]
}

func newSessionClient(
	sess *Session,
	startSeqNum uint64,
	conn *net.TCPConn,
) *SessionClient {
	sc := &SessionClient{

		conn:       conn,
		packetChan: utils.NewRWMutex(make(chan sequencedPacket, 15)),

		startSeqNum: startSeqNum,

		timeout: sess.clientTimeout,

		//closeErr: utils.NewAValue(utils.NewErrorValue(nil)),
		closeErr: &utils.AValue[utils.ErrorValue]{},
	}
	sc.session.Store(sess)
	sc.lastHeartbeat.Store(time.Now().Unix())
	return sc
}

func (sc *SessionClient) LocalAddr() net.Addr {
	return sc.conn.LocalAddr()
}

func (sc *SessionClient) RemoteAddr() net.Addr {
	return sc.conn.RemoteAddr()
}

// SendUnsequenced attempts to send unsequenced data to the client. If an error
// is returned, the Close should be called.
func (sc *SessionClient) SendUnsequenced(payload Payload) error {
	if err := sc.CloseErr(); err != nil {
		return err
	}
	return sc.sendPacket(sequencedPacket{
		seqNum: 0,
		packet: UnsequencedDataPacket(payload),
	})
}

func (sc *SessionClient) sendPacket(packet sequencedPacket) error {
	err := sc.CloseErr()
	if err != nil {
		return err
	}
	sc.packetChan.RApply(func(cp *chan sequencedPacket) {
		ch := *cp
		if ch == nil {
			err = sc.CloseErr()
		}
		// TODO
		select {
		case ch <- packet:
		default:
			err = sc.closeWithErr(ErrSlowClient)
		}
	})
	return err
}

func (sc *SessionClient) Close() {
	sc.closeWithErr(ErrClosed)
	sc.closeAndRemove()
	return
}

func (sc *SessionClient) CloseErr() error {
	ev, _ := sc.closeErr.LoadSafe()
	return ev.Error
}

func (sc *SessionClient) IsClosed() bool {
	return sc.CloseErr() == nil
}

func (sc *SessionClient) start() {
	go sc.runSend()
	go sc.runListen()
}

func (sc *SessionClient) runSend() {
	ch := *sc.packetChan.RLock()
	sc.packetChan.RUnlock()
	if ch == nil {
		return
	}
	// TODO: what to do if a sequence number is skipped
	nextSeqNum := sc.startSeqNum
	queue := utils.NewSlice[sequencedPacket](nil)
Loop:
	for {
		select {
		case seqPacket, ok := <-ch:
			if !ok {
				//fmt.Println("CLOSED:", sc.RemoteAddr().String())
				break Loop
			}
			if seqPacket.packet.PacketType() == PacketTypeEndOfSession {
				//fmt.Println("END OF SESSION:", sc.RemoteAddr().String())
			}
			isSeq := seqPacket.seqNum != 0 &&
				// packet.PacketType() == PacketTypeSequencedData &&
				true
			if isSeq {
				if seqPacket.seqNum > nextSeqNum {
					queue.PushBack(seqPacket)
					continue
				} else if seqPacket.seqNum < nextSeqNum {
					continue
				}
			}
			if _, err := sc.conn.Write(seqPacket.packet.Bytes()); err != nil {
				sc.closeWithErr(err)
				break Loop
			}
			if seqPacket.seqNum == nextSeqNum {
				nextSeqNum++
			}
			if queue.Len() != 0 {
				data := queue.Data()
				for len(data) != 0 {
					spkt := data[0]
					if spkt.seqNum > nextSeqNum {
						break
					}
					if spkt.seqNum == nextSeqNum {
						if _, err := sc.conn.Write(spkt.packet.Bytes()); err != nil {
							sc.closeWithErr(err)
							break Loop
						}
						nextSeqNum++
					}
					data = data[1:]
				}
				queue.SetData(data)
			}
		}
	}
	sc.conn.Close()
	sc.closeAndRemove()
}

func (sc *SessionClient) runListen() {
Loop:
	for {
		packet, err := ReadPacketFrom(sc.conn)
		if err != nil {
			sc.closeWithErr(err)
			break
		}
		sc.updateHeartbeat()
		sess := sc.session.Load()
		if sess == nil {
			break Loop
		}
		switch pt := packet.PacketType(); pt {
		case PacketTypeDebug:
			if sess.debugHandler != nil {
				sess.debugHandler(sc, packet)
			}
		case PacketTypeUnsequencedData:
			if sess.handler != nil {
				sess.handler(sc, packet)
			}
		case PacketTypeClientHeartbeat:
		case PacketTypeLogoutRequest:
			sc.closeWithErr(ErrClientLoggedOut)
			break Loop
		default:
			sc.closeWithErr(&InvalidPacketError{Packet: packet})
			break Loop
		}
	}
	sc.closeAndRemove()
}

func (sc *SessionClient) updateHeartbeat() {
	sc.lastHeartbeat.Store(time.Now().Unix())
}

// This should be called within this package, not Close.
func (sc *SessionClient) closeAndRemove() {
	sess := sc.session.Swap(nil)
	if sess == nil {
		return
	}
	sess.clients.Apply(func(sp **utils.Slice[*SessionClient]) {
		(*sp).RemoveFirst(func(c *SessionClient) bool {
			return c == sc
		})
	})
	sc.close()
}

// Closes the packet chan.
/* NOT SO CURRENTLY: and the TCP connection. */
func (sc *SessionClient) close() {
	// TODO: should the conn be closed here or in runSend?
	//sc.conn.Close()
	sc.packetChan.Apply(func(cp *chan sequencedPacket) {
		ch := *cp
		if ch != nil {
			close(ch)
			*cp = nil
		}
	})
}

func (sc *SessionClient) closeWithErr(err error) error {
	sc.closeErr.StoreIfEmpty(utils.NewErrorValue(err))
	return sc.closeErr.Load().Error
}

type SessionOpts struct {
	Id               SessionId
	SequenceNumber   uint64
	PacketChanLen    int
	ClientTimeout    time.Duration
	NewClientHandler SessionHandler
	DebugHandler     SessionHandler
	//Store DataStore
}

const (
	runningNotStarted int32 = iota
	runningRunning
	runningStopped
)

type Session struct {
	manager atomic.Pointer[SessionsManager]

	id               SessionId
	seqNum           atomic.Uint64
	handler          SessionHandler
	clientTimeout    time.Duration
	newClientHandler SessionHandler
	debugHandler     SessionHandler

	packetChan *utils.RWMutex[chan Packet]
	clients    *utils.Mutex[*utils.Slice[*SessionClient]]
	store      DataStore

	running atomic.Int32

	closeErr *utils.AValue[utils.ErrorValue]
}

func NewSession(
	handler SessionHandler,
	store DataStore,
	opts *SessionOpts,
) *Session {
	if opts.PacketChanLen <= 0 {
		opts.PacketChanLen = 15
	}
	if opts.ClientTimeout <= 0 {
		opts.ClientTimeout = DefaultClientTimeout
	}
	// TODO
	if store == nil {
		store = NewSliceDataStore(1)
	}
	s := &Session{
		id:               opts.Id,
		handler:          handler,
		clientTimeout:    opts.ClientTimeout,
		newClientHandler: opts.NewClientHandler,
		debugHandler:     opts.DebugHandler,

		packetChan: utils.NewRWMutex(make(chan Packet, opts.PacketChanLen)),
		clients:    utils.NewMutex(utils.NewSlice[*SessionClient](nil)),
		store:      store,

		//closeErr: utils.NewAValue(utils.ErrorValue{}),
		closeErr: &utils.AValue[utils.ErrorValue]{},
	}
	// TODO: set seq num as 0?
	s.seqNum.Store(opts.SequenceNumber)
	return s
}

func (s *Session) Id() SessionId {
	return s.id
}

// LastSeqNum returns the sequence number of the last sequenced packet.
func (s *Session) LastSeqNum() uint64 {
	return s.seqNum.Load()
}

// NextSeqNum returns the next sequence number for the next sequenced
// packet.
func (s *Session) NextSeqNum() uint64 {
	return s.LastSeqNum() + 1
}

// LastSequenceNumber returns the sequence number of the last sequenced packet
// as a SequenceNumber.
func (s *Session) LastSequenceNumber() SequenceNumber {
	return SequenceNumberFromUint64(s.LastSeqNum())
}

// NextSequenceNumber returns the next sequence number for the next sequenced
// packet as a SequenceNumber.
func (s *Session) NextSequenceNumber() SequenceNumber {
	return SequenceNumberFromUint64(s.NextSeqNum())
}

// IncrSequenceNumber increments the sequence number, returning the newest
// number.
func (s *Session) incrSequenceNumber() uint64 {
	return s.seqNum.Add(1)
}

func (s *Session) SendSequenced(payload Payload) error {
	err := s.CloseErr()
	if err != nil {
		return err
	}
	s.packetChan.RApply(func(cp *chan Packet) {
		ch := *cp
		if ch == nil {
			err = s.CloseErr()
			return
		}
		ch <- SequencedDataPacket(payload)
	})
	return err
}

func (s *Session) sendHeartbeat() error {
	err := s.CloseErr()
	if err != nil {
		return err
	}
	s.packetChan.RApply(func(cp *chan Packet) {
		ch := *cp
		if ch == nil {
			err = s.CloseErr()
			return
		}
		ch <- ServerHeartbeatPacket()
	})
	return err
}

func (s *Session) End() error {
	err := s.CloseErr()
	if err != nil {
		return err
	}
	s.closeWithErr(ErrSessionEnded)
	return s.end()
}

func (s *Session) end() (err error) {
	s.running.Store(runningStopped)
	sm := s.manager.Swap(nil)
	if sm != nil {
		sm.RemoveSession(s.id, nil)
	} else {
		//return s.CloseErr()
	}
	s.packetChan.Apply(func(cp *chan Packet) {
		ch := *cp
		if ch == nil {
			err = s.CloseErr()
			return
		}
		close(ch)
		*cp = nil
	})
	if err != nil {
		return err
	}
	s.clients.Apply(func(sp **utils.Slice[*SessionClient]) {
		clients := *sp
		if clients == nil {
			err = s.CloseErr()
			return
		}
		for _, client := range clients.Data() {
			client.sendPacket(sequencedPacket{
				seqNum: 0,
				packet: EndOfSessionPacket(),
			})
			s.closeClient(client, ErrSessionEnded, -1, nil)
		}
		// TODO: set sp to nil?
		clients.SetData(nil)
	})
	return err
}

func (s *Session) run() {
	if !s.running.CompareAndSwap(runningNotStarted, runningRunning) {
		return
	}
	ch := *s.packetChan.RLock()
	s.packetChan.RUnlock()
	for packet := range ch {
		heartbeat, done := packet.PacketType() == PacketTypeServerHeartbeat, false
		seqNum := uint64(0)
		if packet.PacketType() == PacketTypeSequencedData {
			seqNum = s.incrSequenceNumber()
			err := s.store.Set(SequenceNumberFromUint64(seqNum), packet.Payload())
			if err != nil {
				// TODO
				s.closeWithErr(err)
				s.end()
				break
			}
		}
		s.clients.Apply(func(sp **utils.Slice[*SessionClient]) {
			clients := *sp
			if clients == nil {
				done = true
				return
			}
			var now time.Time
			if heartbeat {
				now = time.Now()
			}
			for i, client := range clients.Data() {
				if heartbeat {
					diff := now.Sub(time.Unix(client.lastHeartbeat.Load(), 0))
					if diff > s.clientTimeout {
						client.closeWithErr(ErrClientTimedout)
					}
				}
				spkt := sequencedPacket{
					seqNum: seqNum,
					packet: packet,
				}
				if client.sendPacket(spkt) != nil {
					s.closeClient(client, nil, i, clients)
				}
			}
		})
		if done {
			break
		}
	}
	s.clients.Apply(func(sp **utils.Slice[*SessionClient]) {
		clients := *sp
		if clients == nil {
			return
		}
		for _, client := range clients.Data() {
			s.closeClient(client, nil, -1, nil)
		}
		clients.SetData(nil)
	})
}

func (s *Session) closeClient(
	sc *SessionClient,
	err error,
	index int,
	clients *utils.Slice[*SessionClient],
) {
	if err != nil {
		sc.closeWithErr(err)
	}
	if sc.session.Swap(nil) != s {
		return
	}
	sc.close()
	if clients == nil {
		return
	}
	if index < 0 {
		index = clients.Index(func(c *SessionClient) bool {
			return c == sc
		})
	}
	if index != -1 {
		clients.Remove(index)
	}
}

func (s *Session) handle(conn *net.TCPConn, loginPacket Packet) {
	shouldClose := utils.NewT(true)
	defer utils.DeferClose(shouldClose, conn)

	seqNum, ok := loginPacket.SequenceNumber()
	if !ok {
		// TODO
		return
	}
	num, ok := seqNum.ToUint64Safe()
	if !ok {
		// TODO
		return
	}
	nextNum := s.NextSeqNum()
	if num != 0 && num < nextNum {
		nextNum = num
	}
	packet := LoginAcceptedPacket(s.id, SequenceNumberFromUint64(nextNum))
	if _, err := conn.Write(packet.Bytes()); err != nil {
		return
	}

	client := newSessionClient(s, nextNum, conn)
	client.start()
	// TODO
	s.clients.Apply(func(sp **utils.Slice[*SessionClient]) {
		(*sp).PushBack(client)
	})
	for ; nextNum < s.NextSeqNum(); nextNum++ {
		// TODO: payload or packet?
		payload, err := s.store.Get(SequenceNumberFromUint64(nextNum))
		if err != nil {
			// TODO: log or something?
			client.closeWithErr(err)
			client.close()
			return
		}
		client.sendPacket(sequencedPacket{
			seqNum: nextNum,
			packet: SequencedDataPacket(payload),
		})
	}
	*shouldClose = false
	// TODO: move up?
	if s.newClientHandler != nil {
		s.newClientHandler(client, loginPacket)
	}
}

func (s *Session) CloseErr() error {
	ev, _ := s.closeErr.LoadSafe()
	return ev.Error
}

func (s *Session) IsClosed() bool {
	return s.CloseErr() != nil
}

func (s *Session) Running() bool {
	return s.running.Load() == runningRunning
}

func (s *Session) closeWithErr(err error) error {
	s.closeErr.StoreIfEmpty(utils.NewErrorValue(err))
	return s.closeErr.Load().Error
}

type sessionsMngrInfo struct {
	// nil means the session is done
	sessions    *utils.Slice[*Session]
	currSession *Session
}

type SessionsManager struct {
	sessions *utils.RWMutex[sessionsMngrInfo]
	running  atomic.Int32
}

func NewSessionsManager() *SessionsManager {
	sm := &SessionsManager{
		sessions: utils.NewRWMutex(sessionsMngrInfo{
			sessions: utils.NewSlice[*Session](nil),
		}),
	}
	return sm
}

func (sm *SessionsManager) Start() bool {
	return sm.running.CompareAndSwap(runningNotStarted, runningRunning)
}

func (sm *SessionsManager) Running() bool {
	return sm.running.Load() == runningRunning
}

func (sm *SessionsManager) GetSession(id SessionId) (session *Session) {
	sm.sessions.RApply(func(smip *sessionsMngrInfo) {
		if smip.sessions == nil {
			return
		}
		if id.IsBlank() {
			session = smip.currSession
			return
		}
		for _, sess := range smip.sessions.Data() {
			if sess.id.Equal(id) {
				session = sess
				return
			}
		}
	})
	return
}

var (
	ErrSessionOwned  = fmt.Errorf("session already owned")
	ErrSessionExists = fmt.Errorf("session with ID already exists")
)

func (sm *SessionsManager) TryAddCurrent(session *Session) (err error) {
	if session.manager.Load() != nil {
		return ErrSessionOwned
	}
	// TODO: is there a case where session is running at this point?
	sm.sessions.Apply(func(smip *sessionsMngrInfo) {
		if smip.sessions == nil {
			err = ErrShutdown
			return
		}
		for _, sess := range smip.sessions.Data() {
			if sess.id.Equal(session.id) {
				err = ErrSessionExists
				return
			}
		}
		if !session.manager.CompareAndSwap(nil, sm) {
			err = ErrSessionOwned
			return
		}
		smip.sessions.PushBack(session)
		smip.currSession = session
	})
	if err != nil {
		return
	}
	if sm.Running() {
		go session.run()
	}
	return
}

func (sm *SessionsManager) TryAdd(session *Session) (err error) {
	if session.manager.Load() != nil {
		return ErrSessionOwned
	}
	// TODO: is there a case where session is running at this point?
	sm.sessions.Apply(func(smip *sessionsMngrInfo) {
		if smip.sessions == nil {
			err = ErrShutdown
			return
		}
		for _, sess := range smip.sessions.Data() {
			if sess.id.Equal(session.id) {
				err = ErrSessionExists
				return
			}
		}
		if !session.manager.CompareAndSwap(nil, sm) {
			err = ErrSessionOwned
			return
		}
		smip.sessions.PushBack(session)
	})
	if err != nil {
		return
	}
	if sm.Running() {
		go session.run()
	}
	return
}

func (sm *SessionsManager) CurrentSession() *Session {
	smip := sm.sessions.RLock()
	defer sm.sessions.RUnlock()
	if smip.sessions == nil {
		return nil
	}
	return smip.currSession
}

func (sm *SessionsManager) SetCurrentSession(id SessionId) (ok bool) {
	sm.sessions.Apply(func(smip *sessionsMngrInfo) {
		if smip.sessions == nil {
			return
		}
		for _, sess := range smip.sessions.Data() {
			if sess.id.Equal(id) {
				smip.currSession = sess
				ok = true
				return
			}
		}
	})
	return
}

func (sm *SessionsManager) RemoveSession(
	id SessionId,
	replacementId *SessionId,
) (*Session, bool) {
	smip := sm.sessions.Lock()
	defer sm.sessions.Unlock()
	if smip.sessions == nil {
		return nil, false
	}
	sess, _ := smip.sessions.RemoveFirst(func(sess *Session) bool {
		return sess.id.Equal(id)
	})
	if sess == nil {
		return nil, false
	}
	sess.manager.Store(nil)
	wasCurr := smip.currSession.id.Equal(id)
	if wasCurr {
		if replacementId != nil {
			rid := *replacementId
			if replacementId.Equal(SessionIdBlank()) {
				smip.currSession = smip.sessions.Get(smip.sessions.Len() - 1)
			} else {
				i := smip.sessions.Index(func(sess *Session) bool {
					return sess.id.Equal(rid)
				})
				if i != -1 {
					smip.currSession = smip.sessions.Get(i)
				} else {
					smip.currSession = nil
				}
			}
		} else {
			smip.currSession = nil
		}
	}
	return sess, wasCurr && smip.currSession != nil
}

func (sm *SessionsManager) Shutdown() (wasShutdown bool) {
	sm.sessions.Apply(func(smip *sessionsMngrInfo) {
		if smip.sessions == nil {
			return
		}
		wasShutdown = true
		for _, sess := range smip.sessions.Data() {
			// NOTE
			// Set manager to nil so that it won't attempt to remove itself from the
			// manager, which will result in a deadlock.
			sess.manager.Store(nil)
			sess.end()
		}
		smip.sessions = nil
		smip.currSession = nil
	})
	sm.running.Store(runningStopped)
	return wasShutdown
}

func (sm *SessionsManager) IsClosed() bool {
	defer sm.sessions.RUnlock()
	return sm.sessions.RLock().sessions == nil
}

type ServerOpts struct {
	Username Username
	Password Password
}

type lnsSlice = utils.Slice[*net.TCPListener]

type Server struct {
	opts     *ServerOpts
	sessions *SessionsManager
	closeErr *utils.AValue[utils.ErrorValue]
	// The Slice struct should ebe set to nil when shutdown
	listeners *utils.RWMutex[*lnsSlice]
}

func NewServer(sm *SessionsManager, opts *ServerOpts) *Server {
	if sm == nil {
		sm = NewSessionsManager()
	}
	return &Server{
		opts:     opts,
		sessions: sm,
		//closeErr: utils.NewAValue(utils.NewErrorValue(nil)),
		closeErr:  &utils.AValue[utils.ErrorValue]{},
		listeners: utils.NewRWMutex(utils.NewSlice[*net.TCPListener](nil)),
	}
}

func (s *Server) Addrs() (addrs []net.Addr) {
	s.listeners.RApply(func(sp **utils.Slice[*net.TCPListener]) {
		lns := *sp
		if lns == nil {
			return
		}
		addrs = make([]net.Addr, lns.Len())
		for i, ln := range lns.Data() {
			addrs[i] = ln.Addr()
		}
	})
	return
}

func (s *Server) SessionsManager() *SessionsManager {
	return s.sessions
}

func (s *Server) Run(addr string) error {
	if err := s.CloseErr(); err != nil {
		return err
	}
	taddr, err := net.ResolveTCPAddr("tcp", addr)
	if err != nil {
		return err
	}
	ln, err := net.ListenTCP("tcp", taddr)
	if err != nil {
		return err
	}
	return s.RunWithListener(ln)
}

func (s *Server) RunWithListener(ln *net.TCPListener) error {
	defer ln.Close()
	if err := s.CloseErr(); err != nil {
		return err
	}
	s.listeners.Apply(func(sp **lnsSlice) {
		lns := *sp
		if lns == nil {
			ln = nil
			return
		}
		lns.PushBack(ln)
	})
	if ln == nil {
		return s.CloseErr()
	}
	s.sessions.Start()
	for {
		conn, err := ln.AcceptTCP()
		if err != nil {
			ce := s.CloseErr()
			if ce != nil {
				err = ce
			}
			return err
		}
		go s.handle(conn)
	}
}

func (s *Server) handle(conn *net.TCPConn) {
	shouldClose := utils.NewT(true)
	defer utils.DeferClose(shouldClose, conn)
	packet, err := TryReadPacketFromAs(conn, PacketTypeLoginRequest)
	if err != nil {
		return
	}
	username, password, ok := packet.Credentials()
	if !ok {
		return
	}
	if !bytes.EqualFold(username[:], s.opts.Username[:]) ||
		!bytes.EqualFold(password[:], s.opts.Password[:]) {
		conn.Write(LoginRejectPacket(LoginRejectNotAuthorized).Bytes())
		return
	}

	sessionId, ok := packet.SessionId()
	if !ok {
		conn.Write(LoginRejectPacket(LoginRejectSessionNotAvail).Bytes())
		return
	}
	session := s.sessions.GetSession(sessionId)
	if session == nil {
		conn.Write(LoginRejectPacket(LoginRejectSessionNotAvail).Bytes())
		return
	}
	*shouldClose = false
	session.handle(conn, packet)
}

func (s *Server) Shutdown(endSessions bool) (wasShutdown bool) {
	if s.IsClosed() {
		return
	}
	s.closeWithErr(ErrShutdown)
	s.listeners.Apply(func(sp **utils.Slice[*net.TCPListener]) {
		lns := *sp
		if lns == nil {
			return
		}
		for _, ln := range lns.Data() {
			ln.Close()
		}
		*sp = nil
		wasShutdown = true
	})
	if endSessions {
		s.sessions.Shutdown()
	}
	return
}

func (s *Server) closeWithErr(err error) error {
	s.closeErr.StoreIfEmpty(utils.NewErrorValue(err))
	return s.closeErr.Load().Error
}

func (s *Server) CloseErr() error {
	ev, _ := s.closeErr.LoadSafe()
	return ev.Error
}

func (s *Server) IsClosed() bool {
	return s.CloseErr() != nil
}

var (
	ErrShutdown = fmt.Errorf("shutdown")
)

type globalTicker struct {
	funcs    *utils.RWMutex[*Slice[func(time.Time)]]
	shutdown chan utils.Unit
}

func newGlobalTicker() *globalTicker {
	return &globalTicker{
		funcs: utils.NewRWMutex(
			(*Slice[func(time.Time)])(utils.NewSlice[func(time.Time)](nil)),
		),
	}
}

func (sgh globalTicker) add(f func(time.Time)) {
	sgh.funcs.Apply(func(sp **Slice[func(time.Time)]) {
		(*sp).PushBack(f)
	})
}

func (sgh globalTicker) del(f func(time.Time)) {
	fp := reflect.ValueOf(f).Pointer()
	sgh.funcs.Apply(func(sp **Slice[func(time.Time)]) {
		i := (*sp).Index(func(f2 func(time.Time)) bool {
			return reflect.ValueOf(f).Pointer() == fp
		})
		if i != -1 {
			(*sp).SwapRemove(i)
		}
	})
}

func (sgh globalTicker) start() {
	go sgh.run()
}

func (sgh globalTicker) run() {
	ticker := time.NewTicker(time.Second)
	for {
		select {
		case t := <-ticker.C:
			go sgh.runFuncs(t)
			/*
			 */
		case <-sgh.shutdown:
			ticker.Stop()
			return
		}
	}
}

func (sgh globalTicker) runFuncs(t time.Time) {
	sgh.funcs.RApply(func(sp **Slice[func(time.Time)]) {
		for _, f := range (*sp).Data() {
			go f(t)
		}
	})
}

type Slice[T any] utils.Slice[T]

func (s *Slice[T]) SwapRemove(i int) T {
	l := s.Len()
	if i >= l {
		panic(fmt.Sprintf("index %d out of bounds of len %d", i, l))
	}
	data := s.Data()
	data[i], data[l-1] = data[l-1], data[i]
	return utils.First(s.PopBack())
}
