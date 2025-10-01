package main

import (
	"flag"
	"fmt"
	"runtime"
	"sync"
	"time"
)

func splitmix64Step(x uint64) uint64 {
	z := x + 0x9E3779B97F4A7C15
	z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9
	z = (z ^ (z >> 27)) * 0x94D049BB133111EB
	return z ^ (z >> 31)
}

func worker(id, workers int, total, inner uint64, out chan<- uint64) {
	var acc uint64 = 0
	for i := uint64(id); i < total; i += uint64(workers) {
		x := i
		for j := uint64(0); j < inner; j++ {
			x = splitmix64Step(x)
		}
		acc ^= x
	}
	out <- acc
}

func main() {
	var total uint64
	var inner uint64
	var workers int
	var mode string
	var csv bool

	flag.Uint64Var(&total, "total", 10_000_000, "outer iterations")
	flag.Uint64Var(&inner, "inner", 5, "SplitMix64 steps per iteration")
	flag.IntVar(&workers, "workers", runtime.NumCPU(), "worker goroutines (used in multi mode)")
	flag.StringVar(&mode, "mode", "single", "single | multi")
	flag.BoolVar(&csv, "csv", true, "print CSV line")
	flag.Parse()

	logicalMode := "single"
	if mode == "single" {
		runtime.GOMAXPROCS(1)
		workers = 1
	} else {
		logicalMode = "multi"
		if workers <= 0 {
			workers = runtime.NumCPU()
		}
		runtime.GOMAXPROCS(workers)
	}

	start := time.Now()

	out := make(chan uint64, workers)
	var wg sync.WaitGroup
	wg.Add(workers)
	for w := 0; w < workers; w++ {
		go func(id int) {
			defer wg.Done()
			worker(id, workers, total, inner, out)
		}(w)
	}

	go func() {
		wg.Wait()
		close(out)
	}()

	var combined uint64 = 0
	for v := range out {
		combined ^= v
	}

	elapsed := time.Since(start)
	secs := elapsed.Seconds()
	iters := float64(total)
	ips := iters / secs

	fmt.Printf("[GO]   mode=%s workers=%d total=%d inner=%d elapsed_ms=%.3f ips=%.0f checksum=0x%016X\n",
		logicalMode, workers, total, inner, secs*1000.0, ips, combined)

	if csv {
		fmt.Printf("lang,mode,workers,total,inner,elapsed_ms,iterations_per_sec,checksum\n")
		fmt.Printf("go,%s,%d,%d,%d,%.3f,%.0f,0x%016X\n",
			logicalMode, workers, total, inner, secs*1000.0, ips, combined)
	}
}
