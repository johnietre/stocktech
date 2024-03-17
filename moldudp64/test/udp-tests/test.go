package main

import (
	"fmt"
	"net"
	"time"
)

func main() {
	gaddr, err := net.ResolveUDPAddr("udp4", "224.0.0.0:10000")
	//gaddr, err := net.ResolveUDPAddr("udp4", "224.0.0.0:0")
	if err != nil {
		panic(err)
	}
	laddr, err := net.ResolveUDPAddr("udp4", "127.0.0.1:12345")
	if err != nil {
		panic(err)
	}
	laddr = nil
	conn, err := net.DialUDP("udp4", laddr, gaddr)
	if err != nil {
		panic(err)
	}
	fmt.Printf("%s\t%s\n", conn.LocalAddr(), conn.RemoteAddr())
	for i := 0; true; i++ {
		if _, err := conn.Write([]byte{byte(i % 255)}); err != nil {
			panic(err)
		}
		time.Sleep(time.Second)
	}
}
