package main

import (
    "fmt"
    "log"
    "net/http"
    "os"
    "strconv"

    "latticefs/services/internal/api"
    "latticefs/services/internal/share"
)

func main() {
    port := 8771
    if v := os.Getenv("LATTICE_SHARE_PORT"); v != "" {
        if p, err := strconv.Atoi(v); err == nil {
            port = p
        }
    }

    grpcPort := port + 1
    if v := os.Getenv("LATTICE_GRPC_PORT"); v != "" {
        if p, err := strconv.Atoi(v); err == nil {
            grpcPort = p
        }
    }

    if err := api.StartGrpcServer(grpcPort); err != nil {
        log.Printf("gRPC server failed to start: %v", err)
    } else {
        log.Printf("LatticeFS gRPC server listening on :%d", grpcPort)
    }

    mux := http.NewServeMux()
    server := share.NewServer()
    server.Register(mux)

    addr := fmt.Sprintf(":%d", port)
    log.Printf("LatticeFS share server listening on %s", addr)
    if err := http.ListenAndServe(addr, mux); err != nil {
        log.Fatalf("share server failed: %v", err)
    }
}
