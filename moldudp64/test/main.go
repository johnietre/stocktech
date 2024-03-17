package main

import (
	"fmt"
	"log"
	"os"
	"time"

	"moldudp64"
)

const mcAddr = "224.0.0.0:10000"

var (
	session = [10]byte{0, 1, 2, 3, 4, 5, 6, 7, 8, 9}
)

func main() {
	if len(os.Args) == 1 || os.Args[1] != "no" {
		go runTransmitter()
	}

	go runReceiver(1)
	go runReceiver(2)

	time.Sleep(time.Second * 15)
}

func runTransmitter() {
	tm, err := moldudp64.NewTransmitter(mcAddr, session)
	if err != nil {
		log.Fatal(err)
	}
	tm.SetHeartbeat(time.Second * 2)

	time.Sleep(time.Second * 3)

	for i := 1; i <= 10; i++ {
		time.Sleep(time.Second)
		blocks := make([]moldudp64.MessageBlock, 0, i+i)
		for j := 1; j <= cap(blocks); j++ {
			data := []byte(fmt.Sprintf("%d.%d", i, j))
			blocks = append(blocks, must(moldudp64.NewMessageBlock(data)))
		}
		err := tm.SendMessageBlocks(blocks)
		if err != nil {
			log.Fatalf("error sending packet %d: %v", i, err)
		}
	}

	time.Sleep(time.Second)
	if err := tm.SendEndSession(); err != nil {
		log.Fatal("error sending end session: ", err)
	}

	time.Sleep(time.Second)
}

func runReceiver(num int) {
	rcvr, err := moldudp64.NewReceiver(
		mcAddr,
		nil,
		func(packet moldudp64.DownstreamPacket) {
			log.Printf("%d received packet", num)
			for _, block := range packet.MessageBlocks {
				log.Printf("%d received message: %s", num, block.MessageData)
			}
		},
		session,
	)
	if err != nil {
		log.Fatalf("error creating receiver %d: %v", num, err)
	}
	_, _ = <-rcvr.ClosedChan()
	log.Printf("%d closed: %v", num, rcvr.Err())
}

func must[T any](t T, err error) T {
	if err != nil {
		log.Fatal(err)
	}
	return t
}
