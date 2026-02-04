package ipc

import (
    "encoding/binary"
    "fmt"
    "io"
    "net"
    "os"
    "path/filepath"

    "google.golang.org/protobuf/proto"
)

const (
    MaxMessageSize = 100 * 1024 * 1024
)

const (
    MsgShareRequest  uint16 = 101
    MsgShareResponse uint16 = 102
    MsgRevokeRequest uint16 = 103
    MsgRevokeResponse uint16 = 104
    MsgFetchRequest  uint16 = 105
    MsgFetchResponse uint16 = 106
    MsgSyncEvent     uint16 = 201
    MsgSyncAck       uint16 = 202
    MsgStatusRequest uint16 = 301
    MsgStatusResponse uint16 = 302
    MsgShutdownRequest uint16 = 303
    MsgShutdownResponse uint16 = 304
    MsgError         uint16 = 999
)

func socketPath() (string, error) {
    if home := os.Getenv("LATTICE_HOME"); home != "" {
        return filepath.Join(home, "latticefs.sock"), nil
    }
    dir, err := os.UserHomeDir()
    if err != nil {
        return "", err
    }
    return filepath.Join(dir, ".latticefs", "latticefs.sock"), nil
}

func Connect() (net.Conn, error) {
    path, err := socketPath()
    if err != nil {
        return nil, err
    }
    return net.Dial("unix", path)
}

func SendMessage(conn net.Conn, msgType uint16, message proto.Message) error {
    payload, err := proto.Marshal(message)
    if err != nil {
        return err
    }

    length := uint32(2 + len(payload))
    if length > MaxMessageSize {
        return fmt.Errorf("message too large: %d", length)
    }

    if err := binary.Write(conn, binary.BigEndian, length); err != nil {
        return err
    }
    if err := binary.Write(conn, binary.BigEndian, msgType); err != nil {
        return err
    }
    _, err = conn.Write(payload)
    return err
}

func RecvMessage(conn net.Conn) (uint16, []byte, error) {
    var length uint32
    if err := binary.Read(conn, binary.BigEndian, &length); err != nil {
        return 0, nil, err
    }
    if length > MaxMessageSize {
        return 0, nil, fmt.Errorf("message too large: %d", length)
    }

    var msgType uint16
    if err := binary.Read(conn, binary.BigEndian, &msgType); err != nil {
        return 0, nil, err
    }

    payload := make([]byte, length-2)
    if _, err := io.ReadFull(conn, payload); err != nil {
        return 0, nil, err
    }

    return msgType, payload, nil
}
