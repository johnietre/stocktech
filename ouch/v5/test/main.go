package main

import (
  "unsafe"

  "github.com/johnietre/stocktech/ouch/v5"
)

func main() {
  println(unsafe.Sizeof(ouch.Price{}))
}
