package main

import (
	"fmt"
	"math/rand"
	"net"
	"os"
	"sync/atomic"
	"time"

	sbt "github.com/johnietre/stocktech/soupbintcp/v4"
	utils "github.com/johnietre/utils/go"
)

var (
	usernameStr = "utest"
	passwordStr = "ptest"
	unseqParity atomic.Int32
)

func main() {
	srvr, done := startGoServer()
	time.Sleep(time.Second)
	addrs := srvr.Addrs()
	if len(addrs) == 0 {
		panic("no server addrs")
	}
	clients := startGoClients(addrs[0].String(), 50)

	runTest(srvr, clients)
	nextSeqNum := srvr.SessionsManager().CurrentSession().NextSeqNum()

	if !srvr.Shutdown(true) {
		fmt.Println("server wasn't shutdown")
	}
	time.Sleep(time.Second)
	clients.Range(func(c *sbt.Client) bool {
		if nsn := c.NextSeqNum(); nsn != nextSeqNum {
			fmt.Printf(
				"%s: expected next seq num %d, got %d\n",
				c.LocalAddr(), nextSeqNum, nsn,
			)
		}
		if err := c.CloseErr(); err != sbt.ErrSessionEnded {
			fmt.Printf(
				"%s: expected close err of %v, got %v\n",
				c.LocalAddr(), sbt.ErrSessionEnded, err,
			)
		}
		return false
	})
	_, _ = <-done
	if p := unseqParity.Load(); p != 0 {
		panic(fmt.Sprintf("unsequenced parity: %d", p))
	}
	fmt.Println("DONE")
}

func runTest(srvr *sbt.Server, clientsSet *utils.SyncSet[*sbt.Client]) {
	clients := make([]*sbt.Client, 0, clientsSet.SizeHint())
	clientsSet.Range(func(c *sbt.Client) bool {
		clients = append(clients, c)
		return true
	})
	rng := rand.New(rand.NewSource(time.Now().UnixNano()))
	for i := 1; i <= 10; i++ {
		sess := srvr.SessionsManager().CurrentSession()
		if sess == nil {
			panic("no current session")
		}
		payload := orDie(sbt.PayloadFromString(fmt.Sprint("SEQUENCED ", i)))
		if err := sess.SendSequenced(payload); err != nil {
			panic(fmt.Sprint("error sending sequenced: ", err))
		}

		i := rng.Intn(len(clients))
		payload = orDie(sbt.PayloadFromString(fmt.Sprint(i)))
		if err := clients[i].SendUnsequenced(payload); err != nil {
			panic(fmt.Sprintf("%s: %v", clients[i].LocalAddr(), err))
		}
		unseqParity.Add(1)

		time.Sleep(time.Second)
	}
}

func startGoServer() (srvr *sbt.Server, done <-chan utils.Unit) {
	fmt.Println("starting server")
	sm := sbt.NewSessionsManager()
	sess := sbt.NewSession(
		func(sc *sbt.SessionClient, packet sbt.Packet) {
			if pt := packet.PacketType(); pt != sbt.PacketTypeUnsequencedData {
				panic(fmt.Sprintf("received unexpected packet type: %s", pt))
			}
			unseqParity.Add(-1)
			load := packet.Payload()
			fmt.Printf("payload len: %d\nunsequenced payload: %s\n", len(load), load)
		},
		nil,
		&sbt.SessionOpts{
			Id: orDie(sbt.SessionIdFromString("1")),
			NewClientHandler: func(sc *sbt.SessionClient, packet sbt.Packet) {
				fmt.Printf("new connection from %s\n", sc.RemoteAddr())
			},
			DebugHandler: func(sc *sbt.SessionClient, packet sbt.Packet) {
				fmt.Printf(
					"received debug from: %s\ndebug payload: %s\n",
					sc.RemoteAddr(), packet.Payload(),
				)
			},
		},
	)
	if err := sm.TryAddCurrent(sess); err != nil {
		panic(fmt.Sprint("error adding session: ", err))
	}

	srvr = sbt.NewServer(sm, &sbt.ServerOpts{
		Username: sbt.UsernameFromStringTrunc(usernameStr),
		Password: sbt.PasswordFromStringTrunc(passwordStr),
	})

	ln, err := newTcpListener()
	if err != nil {
		panic(err)
	}

	doneChan := make(chan utils.Unit, 1)
	go func() {
		err = srvr.RunWithListener(ln)
		if err != nil {
			if err != sbt.ErrShutdown {
				die(err)
			}
		}
		doneChan <- utils.Unit{}
		close(doneChan)
	}()
	fmt.Println("started server")
	return srvr, doneChan
}

func startGoClients(addr string, numClients int) *utils.SyncSet[*sbt.Client] {
	fmt.Println("starting clients")
	clients := utils.NewSyncSet[*sbt.Client]()
	for i := 0; i < numClients; i++ {
		ch := func(c *sbt.Client, packet sbt.Packet) {
			switch pt := packet.PacketType(); pt {
			case sbt.PacketTypeUnsequencedData:
				if err := c.SendUnsequenced(packet.Payload()); err != nil {
					panic(fmt.Sprintf("%s: %v", c.LocalAddr(), err))
				}
			case sbt.PacketTypeSequencedData:
			default:
				panic(fmt.Sprintf(
					"%s: received unexpected packet type: %s",
					c.LocalAddr(), pt,
				))
			}
			load := packet.Payload()
			fmt.Printf(
				"%s: packet type: %s\npayload len: %d\npayload: %s\n",
				c.LocalAddr(), packet.PacketType(), len(load), load,
			)
		}
		if i%2 == 1 {
			ch = nil
		}
		c, err := sbt.ConnectWithOpts(
			addr,
			ch,
			&sbt.ConnectOpts{
				Session:  sbt.SessionIdFromStringTrunc("1"),
				Username: sbt.UsernameFromStringTrunc(usernameStr),
				Password: sbt.PasswordFromStringTrunc(passwordStr),
				DebugHandler: func(c *sbt.Client, packet sbt.Packet) {
					fmt.Printf(
						"%s: received debug\ndebug payload: %s\n",
						c.LocalAddr(), packet.Payload(),
					)
				},
			},
		)
		if err != nil {
			panic(err)
		}
		if c.Handler() == nil {
			go func() {
				for {
					packet, err, ok := c.ReadPacket()
					if err != nil {
						panic(fmt.Sprintf(
							"%s: error reading packet: %v",
							c.LocalAddr(), err,
						))
					} else if !ok {
						panic("expected ok reading packet")
					}
					switch packet.PacketType() {
					case sbt.PacketTypeServerHeartbeat:
						c.ResetServerHeartbeat()
					case sbt.PacketTypeUnsequencedData:
					case sbt.SequenceNumberLen:
						c.IncrSequenceNumber()
					case sbt.PacketTypeEndOfSession:
						return
					}
				}
			}()
		}
		clients.Insert(c)
	}
	fmt.Println("started clients")
	return clients
}

func newTcpListener() (*net.TCPListener, error) {
	ln, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		return nil, err
	}
	return ln.(*net.TCPListener), nil
}

func orDie[T any](t T, err error) T {
	if err != nil {
		die(err)
	}
	return t
}

func die(args ...any) {
	fmt.Fprintln(os.Stderr, args...)
	os.Exit(1)
}
