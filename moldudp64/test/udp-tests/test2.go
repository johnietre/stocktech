package main

import (
  "fmt"
  "net"
)

func main() {
  go listen(1)
  listen(2)
}

func listen(num int) {
  gaddr, err := net.ResolveUDPAddr("udp4", "224.0.0.0:10000")
  //gaddr, err := net.ResolveUDPAddr("udp4", "224.0.0.0:0")
  if err != nil {
    panic(err)
  }
  conn, err := net.ListenMulticastUDP("udp4", nil, gaddr)
  if err != nil {
    panic(err)
  }
  fmt.Printf("%d is %s\t%s\n", num, conn.LocalAddr(), conn.RemoteAddr())
  for {
    b := make([]byte, 1000)
    n, addr, err := conn.ReadFromUDP(b)
    if err != nil {
      fmt.Println(err)
    } else {
      fmt.Printf("%d: %s (%d bytes) => %d\n", num, addr, n, b[0])
    }
  }
}
