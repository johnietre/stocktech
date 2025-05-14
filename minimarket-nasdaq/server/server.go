package main

import (
	"fmt"
	"log"
	"net"
	"os"
	"os/signal"
	"path/filepath"
	"time"

	sbtcp "github.com/johnietre/stocktech/soupbintcp/v4"
	utils "github.com/johnietre/utils/go"
)

func main() {
  log.SetFlags(0)

  srvr := &Server{
    SrvrOpts: &sbtcp.ServerOpts{
      Username: sbtcp.UsernameFromStringTrunc("johnie"),
      Password: sbtcp.PasswordFromStringTrunc("rodgerstre"),
    },
    SessMngr: sbtcp.NewSessionsManager(),
  }
  srvr.SetCurrentSession(sbtcp.NewSession(
    sbtcp.SessionHandler(srvr.sessionHandler),
    sbtcp.NewMapDataStore(),
    &sbtcp.SessionOpts{
      Id: sbtcp.SessionIdFromStringTrunc("12345"),
      NewClientHandler: sbtcp.SessionHandler(srvr.newClientHandler),
      DebugHandler: sbtcp.SessionHandler(srvr.debugHandler),
    },
  ))

  timer, count := (*time.Timer)(nil), 1
  timer = time.AfterFunc(time.Second * 5, func() {
    defer timer.Reset(time.Second * 5)

    sess := srvr.SessMngr.CurrentSession()
    if sess == nil {
      log.Print("no current session")
      return
    }
    payload, err := sbtcp.PayloadFromString(fmt.Sprint("message #", count))
    if err != nil {
      log.Printf("error creating payload: %v", err)
      return
    }
    if err := sess.SendSequenced(payload); err != nil {
      log.Printf("error sending sequenced: %v", err)
      return
    }
    log.Printf("sequenced data #%d sent", count)
    count++
  })

  signalCh := make(chan os.Signal, 5)
  signal.Notify(signalCh, os.Interrupt)
  go func() {
    ran := false
    for range signalCh {
      fmt.Println("received interrupt")
      if ran {
        break
      }
      srvr.srvr.Shutdown(true)
      fmt.Println("sent")
      ran = true
    }
    fmt.Println("Exitting")
    os.Exit(0)
  }()

  log.Print("attempting to run on 127.0.0.1:9000")
  srvr.Init()
  go srvr.Run("127.0.0.1:9000")
  //srvr.srvr.Wait(sbtcp.NeverDoneContext())
  srvr.srvr.SessionsManager().Wait(sbtcp.NeverDoneContext())
}

type Server struct {
  SrvrOpts *sbtcp.ServerOpts
  SessMngr *sbtcp.SessionsManager
  srvr *sbtcp.Server
  clients *utils.SyncMap[net.Addr, *sbtcp.SessionClient]
  ranInit bool
}

func (s *Server) Init() {
  s.clients = utils.NewSyncMap[net.Addr, *sbtcp.SessionClient]()
  if s.SessMngr == nil {
    s.SessMngr = sbtcp.NewSessionsManager()
  }
  s.SessMngr.Start()
  s.srvr = sbtcp.NewServer(s.SessMngr, s.SrvrOpts)
  s.ranInit = true
}

func (s *Server) Run(addr string) error {
  if !s.ranInit {
    s.Init()
  }
  return s.srvr.Run(addr)
}

func (s *Server) SetCurrentSession(sess *sbtcp.Session) error {
  return s.SessMngr.TryAddCurrent(sess)
}

var (
  debugDir string
)

func (*Server) sessionHandler(client *sbtcp.SessionClient, pkt sbtcp.Packet) {
  log.Printf(
    "received packet from %s with payload length of %d",
    client.RemoteAddr(), len(pkt.Payload()),
  )
  client.SendUnsequenced(pkt.Payload())
}

func (srvr *Server) newClientHandler(client *sbtcp.SessionClient, _ sbtcp.Packet) {
  ts := time.Now().UnixNano()
  log.Printf("%s: CONNTECTED (%d)", client.RemoteAddr(), ts)
  srvr.clients.Store(client.RemoteAddr(), client)
  go func() {
    client.Wait(sbtcp.NeverDoneContext())
    ts := time.Now().UnixNano()
    log.Printf(
      "%s: CLOSED with reason: %v | (%d)",
      client.RemoteAddr(),
      client.CloseErr(),
      ts,
    )
  }()
}

func (*Server) debugHandler(client *sbtcp.SessionClient, pkt sbtcp.Packet) {
  ts := time.Now().UnixNano()
  filename := fmt.Sprintf("%s-%d.debug", client.RemoteAddr(), ts)
  err := os.WriteFile(filepath.Join(debugDir, filename), pkt.Payload(), 0777)
  if err != nil {
    log.Printf(
      "received debug packet from %s at %d but error writing file: %v",
      client.RemoteAddr(),
      ts,
      err,
    )
    return
  }
  log.Printf(
    "received debug packet from %s (see %s)",
    client.RemoteAddr(),
    filename,
  )
}
