package main

import (
	"fmt"
	"math"
)

func main() {
	fmt.Println("64:", math.Nextafter(999.9800, 199_999.9900))
	fmt.Println("32:", math.Nextafter32(999.9800, 199_999.9900))

	fmt.Println("64:", math.Nextafter(199_999.9900, 1e9))
	fmt.Println("32:", math.Nextafter32(199_999.9900, 1e9))

	fmt.Println(diffToNext(200_000_0000))
	fmt.Println(diffToNext(214_748_3647))
	fmt.Println(diffToNext(999_999_9999))
}

func diffToNext(f float64) float64 {
	next := math.Nextafter(f, math.Inf(1))
	return next - f
}
