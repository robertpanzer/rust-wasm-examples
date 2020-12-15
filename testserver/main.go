package main

import (
	"flag"
	"fmt"
	"io"
	"log"
	"net"
	"net/http"
	"regexp"
	"strconv"
	"time"
)

var (
	listenAddr     = flag.String("l", ":8086", "Listen address")
	durationRegex  = regexp.MustCompile(`^/sleep/(\d+)$`)
)

func main() {

    flag.Parse()
	http.HandleFunc("/sleep/", func(rw http.ResponseWriter, req *http.Request) {
		matches := durationRegex.FindStringSubmatch(req.URL.Path)
		if matches == nil {
			rw.WriteHeader(http.StatusBadRequest)
			return
		}
		duration, err := strconv.ParseInt(matches[1], 10, 64)
		if err != nil {
			rw.WriteHeader(http.StatusBadRequest)
			_, _ = io.WriteString(rw, fmt.Sprintf("Cannot convert duration %s", matches[1]))
		}
		sleepTime := time.Duration(duration) * time.Millisecond
		log.Printf("Sleep for %v for %s", sleepTime, req.RemoteAddr)
		time.Sleep(sleepTime)
		rw.WriteHeader(http.StatusOK)
	})

	l, err := net.Listen("tcp", *listenAddr)
	if err != nil {
		log.Fatal(err)
	}
    log.Printf("Testserver listening on %v", l.Addr())

	err = http.Serve(l, nil)
	if err != nil {
		log.Println(err)
	}
}
