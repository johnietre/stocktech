package main

import (
	"bufio"
	"fmt"
	"io"
	"log"
	"os"
	"os/signal"
	"strconv"
	"strings"

	sbtcp "github.com/johnietre/stocktech/soupbintcp/v4"
)

var (
  reader = bufio.NewReader(os.Stdin)
)

func main() {
  log.SetFlags(0)

  fmt.Print("Sequence Number: ")
  line, err := reader.ReadString('\n')
  if err != nil {
    if err != io.EOF {
      log.Print(err)
    }
    return
  }
  num, _ := strconv.ParseUint(strings.TrimSpace(line), 10, 64)

  /*
  client, err := sbtcp.Connect(
    "127.0.0.1:9000",
    sbtcp.UsernameFromStringTrunc("johnie"),
    sbtcp.PasswordFromStringTrunc("rodgerstre"),
    sbtcp.ClientHandler(clientHandler),
  )
  */
  client, err := sbtcp.ConnectWithOpts(
    "127.0.0.1:9000",
    sbtcp.ClientHandler(clientHandler),
    &sbtcp.ConnectOpts{
      Username: sbtcp.UsernameFromStringTrunc("johnie"),
      Password: sbtcp.PasswordFromStringTrunc("rodgerstre"),
      SequenceNumber: sbtcp.SequenceNumberFromUint64(num),
    },
  )
  if err != nil {
    log.Fatal("error connecting: ", err)
  }

  signalCh := make(chan os.Signal, 5)
  signal.Notify(signalCh, os.Interrupt)
  go func() {
    <-signalCh
    log.Print("EXITING")
    client.Logout()
    flushPackets()
    log.Println("GOODBYE")
    os.Exit(0)
  }()

  for {
    fmt.Print("Message: ")
    line, err := reader.ReadString('\n')
    if err != nil {
      if err != io.EOF {
        log.Printf("error reading line: %v", err)
      }
      break
    }
    line = strings.TrimSpace(line)
    if len(line) == 0 {
      flushPackets()
      continue
    }
    payload, err := sbtcp.PayloadFromString(line)
    if err != nil {
      log.Print("error creating payload: ", err)
      continue
    }
    if err := client.SendUnsequenced(payload); err != nil {
      log.Print("error sending unsequenced packet: ", err)
    }
  }
  client.Logout()
  flushPackets()
}

var (
  packets []sbtcp.Packet
)

func flushPackets() {
  for _, pkt := range packets {
    switch pt := pkt.PacketType(); pt {
    case sbtcp.PacketTypeSequencedData:
      fallthrough
    case sbtcp.PacketTypeUnsequencedData:
      fallthrough
    case sbtcp.PacketTypeDebug:
      log.Printf(
        "received %s packet from server with payload:\n%s",
        pt, pkt.Payload(),
      )
    case sbtcp.PacketTypeEndOfSession:
      log.Print("session ended")
    default:
      log.Printf(
        "received unexpected packet type (%s) from server with payload:\n%s",
        pt, pkt.Payload(),
      )
    }
  }
  packets = packets[:0]
}

func clientHandler(client *sbtcp.Client, pkt sbtcp.Packet) {
  //fmt.Println("RECEIVED:", string(pkt.Payload()))
  packets = append(packets, pkt)
}
