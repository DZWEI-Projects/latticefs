# IPC Server

The IPC server provides local Rust<->Go communication for sharing and admin operations.

## Socket path
```
$LATTICE_HOME/latticefs.sock
```
Defaults to `~/.latticefs/latticefs.sock` if `LATTICE_HOME` is not set.

## Message framing
All IPC messages are length-prefixed:
- Length: 4 bytes, big-endian
- Message type: 2 bytes, big-endian
- Payload: Protocol Buffers message

## Message types
Share / revoke / fetch requests and responses are defined in `services/proto/ipc.proto`.

## Running the IPC server
The IPC server is run via the CLI:
```bash
lfs ipc
```

It is required by the share server for all share and fetch operations.
