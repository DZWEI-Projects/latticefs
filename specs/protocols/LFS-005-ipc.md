# LFS-005: Inter-Process Communication Protocol

**Status:** Draft
**Version:** 0.1.0
**Date:** 2026-02-03
**Authors:** NeuralFS Team

---

## Abstract

This document specifies the inter-process communication (IPC) protocol between NeuralFS Rust components (base library, CLI, FUSE) and Go services (share daemon, sync orchestrator). The protocol uses Unix domain sockets for fast local communication with length-prefixed message framing and Protocol Buffers for serialization.

---

## 1. Introduction

### 1.1 Motivation

NeuralFS is a hybrid Rust + Go system:

- **Rust**: Core storage, policy engine, crypto, FUSE mount (performance-critical)
- **Go**: Sync daemon, share server, admin APIs (network services)

These components need efficient, type-safe communication for:

- Share requests from CLI → share daemon
- Sync events from daemon → Rust storage
- Admin operations from Go APIs → Rust core

### 1.2 Design Goals

- **Fast**: Unix domain sockets (no network overhead)
- **Type-safe**: Protocol Buffers with code generation
- **Bi-directional**: Request-response + streaming
- **Versioned**: Protocol versioning for compatibility
- **Secure**: Socket permissions, no authentication needed (local-only)

### 1.3 Terminology

- **Client**: Process initiating connection (CLI, FUSE, Go daemon)
- **Server**: Process accepting connections (Rust base library runs server)
- **Message**: Single Protocol Buffers-encoded request or response
- **Stream**: Sequence of related messages (e.g., sync events)

---

## 2. Transport Layer

### 2.1 Unix Domain Sockets

**Socket Path:** `$LATTICE_HOME/latticefs.sock`

Example: `/Users/alice/.latticefs/latticefs.sock`

**Socket Type:** `SOCK_STREAM` (connection-oriented)

**Permissions:** `0600` (owner read/write only)

### 2.2 Server Lifecycle

```rust
async fn start_ipc_server() -> Result<()> {
    let socket_path = get_socket_path()?;

    // 1. Remove stale socket
    if socket_path.exists() {
        fs::remove_file(&socket_path).await?;
    }

    // 2. Create Unix listener
    let listener = UnixListener::bind(&socket_path)?;

    // 3. Set permissions (owner-only)
    let mut perms = fs::metadata(&socket_path).await?.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(&socket_path, perms).await?;

    // 4. Accept connections
    loop {
        let (stream, _addr) = listener.accept().await?;
        tokio::spawn(handle_connection(stream));
    }
}
```

### 2.3 Client Connection

```go
func connectIPC() (net.Conn, error) {
    socketPath := getSocketPath()
    conn, err := net.Dial("unix", socketPath)
    if err != nil {
        return nil, fmt.Errorf("failed to connect: %w", err)
    }
    return conn, nil
}
```

---

## 3. Message Framing

### 3.1 Frame Format

Messages use length-prefixed framing:

```
┌──────────────┬─────────────────────┬──────────────┐
│ Length (4B)  │ Message Type (2B)   │ Payload (N)  │
│ Big-endian   │ Big-endian          │ Protobuf     │
└──────────────┴─────────────────────┴──────────────┘

Length: Total bytes (message type + payload)
Message Type: Enum indicating protobuf message type
Payload: Serialized Protocol Buffers message
```

**Example:**

```
Length: 0x00000042 (66 bytes)
Message Type: 0x0001 (ShareRequest)
Payload: <66 bytes of protobuf data>
```

### 3.2 Framing Implementation (Rust)

```rust
async fn send_message<T: Message>(
    stream: &mut UnixStream,
    msg_type: MessageType,
    message: &T,
) -> Result<()> {
    // 1. Serialize protobuf
    let payload = message.encode_to_vec();

    // 2. Calculate length (msg_type + payload)
    let length = (2 + payload.len()) as u32;

    // 3. Write frame
    stream.write_u32(length).await?;  // Big-endian
    stream.write_u16(msg_type as u16).await?;
    stream.write_all(&payload).await?;

    stream.flush().await?;
    Ok(())
}

async fn recv_message(stream: &mut UnixStream) -> Result<(MessageType, Vec<u8>)> {
    // 1. Read length
    let length = stream.read_u32().await?;

    if length > MAX_MESSAGE_SIZE {
        return Err(Error::MessageTooLarge(length));
    }

    // 2. Read message type
    let msg_type = MessageType::from_u16(stream.read_u16().await?)?;

    // 3. Read payload
    let payload_len = length - 2;
    let mut payload = vec![0u8; payload_len as usize];
    stream.read_exact(&mut payload).await?;

    Ok((msg_type, payload))
}
```

### 3.3 Framing Implementation (Go)

```go
func sendMessage(conn net.Conn, msgType uint16, message proto.Message) error {
    // 1. Serialize protobuf
    payload, err := proto.Marshal(message)
    if err != nil {
        return err
    }

    // 2. Calculate length
    length := uint32(2 + len(payload))

    // 3. Write frame
    binary.Write(conn, binary.BigEndian, length)
    binary.Write(conn, binary.BigEndian, msgType)
    conn.Write(payload)

    return nil
}

func recvMessage(conn net.Conn) (uint16, []byte, error) {
    // 1. Read length
    var length uint32
    binary.Read(conn, binary.BigEndian, &length)

    if length > maxMessageSize {
        return 0, nil, fmt.Errorf("message too large: %d", length)
    }

    // 2. Read message type
    var msgType uint16
    binary.Read(conn, binary.BigEndian, &msgType)

    // 3. Read payload
    payload := make([]byte, length-2)
    io.ReadFull(conn, payload)

    return msgType, payload, nil
}
```

---

## 4. Message Types

### 4.1 Message Type Enum

```protobuf
enum MessageType {
  MSG_UNKNOWN = 0;

  // Share operations (1xx)
  MSG_SHARE_REQUEST = 101;
  MSG_SHARE_RESPONSE = 102;
  MSG_REVOKE_REQUEST = 103;
  MSG_REVOKE_RESPONSE = 104;
  MSG_FETCH_REQUEST = 105;
  MSG_FETCH_RESPONSE = 106;

  // Sync operations (2xx)
  MSG_SYNC_EVENT = 201;
  MSG_SYNC_ACK = 202;

  // Admin operations (3xx)
  MSG_STATUS_REQUEST = 301;
  MSG_STATUS_RESPONSE = 302;
  MSG_SHUTDOWN_REQUEST = 303;
  MSG_SHUTDOWN_RESPONSE = 304;

  // Errors (9xx)
  MSG_ERROR = 999;
}
```

---

## 5. Protocol Buffers Schema

### 5.1 Common Types

```protobuf
syntax = "proto3";

package latticefs.ipc;

// Timestamp (Unix microseconds)
message Timestamp {
  int64 micros = 1;
}

// Object identifier
message ObjectID {
  bytes uuid = 1;  // 16 bytes (UUID)
}

// Version identifier
message VersionID {
  bytes uuid = 1;
}

// Content hash (BLAKE3)
message Hash {
  bytes blake3 = 1;  // 32 bytes
}

// Error message
message Error {
  uint32 code = 1;
  string message = 2;
  map<string, string> details = 3;
}
```

### 5.2 Share Messages

```protobuf
// Share request: Create capability for object
message ShareRequest {
  ObjectID object_id = 1;
  string audience_did = 2;  // did:key:...
  string capability = 3;     // "read", "write", etc.
  Timestamp expires_at = 4;
  map<string, string> facts = 5;  // Optional constraints
}

message ShareResponse {
  oneof result {
    string ucan_token = 1;  // Success: UCAN token
    Error error = 2;         // Failure
  }
}

// Revoke request: Invalidate capability
message RevokeRequest {
  string ucan_cid = 1;  // CID of UCAN to revoke
  string reason = 2;     // Optional
}

message RevokeResponse {
  oneof result {
    bool success = 1;
    Error error = 2;
  }
}

// Fetch request: Retrieve object via UCAN
message FetchRequest {
  ObjectID object_id = 1;
  string ucan_token = 2;
  optional VersionID version_id = 3;  // Specific version, or latest
}

message FetchResponse {
  oneof result {
    ObjectData data = 1;
    Error error = 2;
  }
}

message ObjectData {
  ObjectID object_id = 1;
  VersionID version_id = 2;
  bytes content = 3;           // Object content
  map<string, string> metadata = 4;
  repeated Tag tags = 5;
}

message Tag {
  string key = 1;
  string value = 2;
}
```

### 5.3 Sync Messages

```protobuf
// Sync event: Notify of graph changes
message SyncEvent {
  Timestamp timestamp = 1;
  string event_type = 2;  // "object_created", "version_added", etc.
  bytes payload = 3;       // Event-specific data
}

// Event types

message ObjectCreatedEvent {
  ObjectID object_id = 1;
  VersionID version_id = 2;
  string actor_did = 3;
}

message VersionAddedEvent {
  ObjectID object_id = 1;
  VersionID version_id = 2;
  optional VersionID parent_version = 3;
  string actor_did = 4;
}

message TagAddedEvent {
  ObjectID object_id = 1;
  Tag tag = 2;
  string actor_did = 3;
}

message LinkCreatedEvent {
  ObjectID source = 1;
  ObjectID target = 2;
  string link_type = 3;
  string actor_did = 4;
}

// Sync acknowledgment
message SyncAck {
  Timestamp event_timestamp = 1;
  bool success = 2;
}
```

### 5.4 Admin Messages

```protobuf
// Status request: Get system status
message StatusRequest {
  bool include_stats = 1;
}

message StatusResponse {
  string version = 1;
  bool running = 2;
  optional Stats stats = 3;
}

message Stats {
  uint64 total_objects = 1;
  uint64 total_versions = 2;
  uint64 total_chunks = 3;
  uint64 storage_bytes = 4;
  uint64 dedup_ratio_x100 = 5;  // E.g., 150 = 1.5x
}

// Shutdown request
message ShutdownRequest {
  bool force = 1;
  uint32 timeout_seconds = 2;
}

message ShutdownResponse {
  bool success = 1;
}
```

---

## 6. Request-Response Pattern

### 6.1 Client Request Flow

```rust
async fn request_share(
    conn: &mut UnixStream,
    object_id: ObjectID,
    audience: &str,
) -> Result<String> {
    // 1. Build request
    let request = ShareRequest {
        object_id: Some(object_id.into()),
        audience_did: audience.to_string(),
        capability: "read".to_string(),
        expires_at: Some(Timestamp::now() + Duration::from_days(7)),
        facts: HashMap::new(),
    };

    // 2. Send request
    send_message(conn, MessageType::ShareRequest, &request).await?;

    // 3. Receive response
    let (msg_type, payload) = recv_message(conn).await?;

    if msg_type != MessageType::ShareResponse {
        return Err(Error::UnexpectedMessageType);
    }

    let response = ShareResponse::decode(&payload[..])?;

    // 4. Handle response
    match response.result {
        Some(share_response::Result::UcanToken(token)) => Ok(token),
        Some(share_response::Result::Error(err)) => {
            Err(Error::Remote(err.message))
        }
        None => Err(Error::EmptyResponse),
    }
}
```

### 6.2 Server Response Flow

```rust
async fn handle_share_request(
    conn: &mut UnixStream,
    request: ShareRequest,
) -> Result<()> {
    // 1. Validate request
    let object_id = ObjectID::try_from(request.object_id.ok_or(Error::MissingField)?)?;
    let audience = PublicKey::from_did(&request.audience_did)?;

    // 2. Create UCAN
    let ucan = capability::issue_ucan(
        &get_identity()?,
        &audience,
        &object_id,
        request.capability.parse()?,
        request.expires_at.ok_or(Error::MissingField)?.into(),
    ).await;

    // 3. Build response
    let response = match ucan {
        Ok(token) => ShareResponse {
            result: Some(share_response::Result::UcanToken(token.encode())),
        },
        Err(err) => ShareResponse {
            result: Some(share_response::Result::Error(Error {
                code: err.code(),
                message: err.to_string(),
                details: HashMap::new(),
            })),
        },
    };

    // 4. Send response
    send_message(conn, MessageType::ShareResponse, &response).await?;

    Ok(())
}
```

---

## 7. Streaming Pattern

### 7.1 Event Streaming

The Go sync daemon streams events to Rust storage:

```go
// Go daemon sends events
func streamSyncEvents(conn net.Conn) error {
    for event := range eventChannel {
        msg := &SyncEvent{
            Timestamp: timestampNow(),
            EventType: event.Type,
            Payload:   event.Payload,
        }

        if err := sendMessage(conn, MSG_SYNC_EVENT, msg); err != nil {
            return err
        }

        // Wait for ack
        msgType, payload, err := recvMessage(conn)
        if err != nil {
            return err
        }

        var ack SyncAck
        proto.Unmarshal(payload, &ack)

        if !ack.Success {
            return fmt.Errorf("sync failed for event %s", event.Type)
        }
    }

    return nil
}
```

### 7.2 Rust Event Handler

```rust
async fn handle_sync_stream(mut conn: UnixStream) -> Result<()> {
    loop {
        let (msg_type, payload) = recv_message(&mut conn).await?;

        match msg_type {
            MessageType::SyncEvent => {
                let event = SyncEvent::decode(&payload[..])?;

                // Process event
                let result = process_sync_event(&event).await;

                // Send ack
                let ack = SyncAck {
                    event_timestamp: event.timestamp,
                    success: result.is_ok(),
                };

                send_message(&mut conn, MessageType::SyncAck, &ack).await?;

                if let Err(e) = result {
                    eprintln!("Sync event failed: {}", e);
                }
            }
            _ => {
                eprintln!("Unexpected message type: {:?}", msg_type);
                break;
            }
        }
    }

    Ok(())
}
```

---

## 8. Error Handling

### 8.1 Error Codes

```rust
pub enum ErrorCode {
    Unknown = 0,
    InvalidRequest = 1,
    ObjectNotFound = 2,
    Unauthorized = 3,
    InvalidUCAN = 4,
    Expired = 5,
    QuotaExceeded = 6,
    InternalError = 99,
}
```

### 8.2 Error Response

```protobuf
message Error {
  uint32 code = 1;           // ErrorCode
  string message = 2;         // Human-readable
  map<string, string> details = 3;  // Additional context
}
```

**Example:**

```json
{
  "code": 2,
  "message": "Object not found: 01934e3a-7c5a-7b3c-8d2e-1f4a5b6c7d8e",
  "details": {
    "object_id": "01934e3a-7c5a-7b3c-8d2e-1f4a5b6c7d8e",
    "searched_at": "2025-02-03T10:30:00Z"
  }
}
```

### 8.3 Connection Errors

- **Connection Refused**: Server not running
- **Permission Denied**: Socket permissions wrong
- **Broken Pipe**: Server crashed during request
- **Timeout**: No response within timeout period

**Mitigation:** Clients SHOULD retry with exponential backoff.

---

## 9. Security Considerations

### 9.1 Socket Permissions

Socket MUST be `0600` (owner-only):

```bash
$ ls -l ~/.latticefs/latticefs.sock
srw-------  1 alice  staff  0 Feb  3 10:00 latticefs.sock
```

**Verification:**

```rust
fn verify_socket_permissions(path: &Path) -> Result<()> {
    let metadata = fs::metadata(path)?;
    let perms = metadata.permissions();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if perms.mode() & 0o777 != 0o600 {
            return Err(Error::InsecureSocketPermissions);
        }
    }

    Ok(())
}
```

### 9.2 No Authentication Required

Unix domain sockets provide OS-level authentication (UID/GID).

Only the socket owner can connect → no additional auth needed.

### 9.3 Message Size Limits

```rust
const MAX_MESSAGE_SIZE: u32 = 100 * 1024 * 1024;  // 100 MiB
```

Prevents memory exhaustion attacks.

### 9.4 Input Validation

All protobuf messages MUST be validated:

```rust
fn validate_share_request(req: &ShareRequest) -> Result<()> {
    // Check required fields
    if req.object_id.is_none() {
        return Err(Error::MissingField("object_id"));
    }

    // Validate DID format
    if !req.audience_did.starts_with("did:key:") {
        return Err(Error::InvalidDID);
    }

    // Validate capability
    let cap: Capability = req.capability.parse()
        .map_err(|_| Error::InvalidCapability)?;

    // Validate expiration (not in past)
    let exp = req.expires_at.ok_or(Error::MissingField("expires_at"))?;
    if exp.micros < Timestamp::now().micros {
        return Err(Error::InvalidExpiration);
    }

    Ok(())
}
```

---

## 10. Performance Considerations

### 10.1 Connection Pooling

Clients SHOULD reuse connections:

```go
type ConnectionPool struct {
    conn   net.Conn
    mu     sync.Mutex
    active bool
}

func (p *ConnectionPool) Get() (net.Conn, error) {
    p.mu.Lock()
    defer p.mu.Unlock()

    if p.conn == nil || !p.active {
        conn, err := connectIPC()
        if err != nil {
            return nil, err
        }
        p.conn = conn
        p.active = true
    }

    return p.conn, nil
}
```

### 10.2 Parallel Requests

Multiple clients can connect simultaneously:

```rust
// Server handles concurrent connections
loop {
    let (stream, _addr) = listener.accept().await?;
    tokio::spawn(handle_connection(stream));  // Spawn per-connection task
}
```

### 10.3 Buffering

Use buffered I/O for small messages:

```rust
let stream = BufStream::new(stream);  // Tokio BufStream
```

---

## 11. Versioning

### 11.1 Protocol Version

Every message includes a protocol version:

```protobuf
message MessageHeader {
  uint32 protocol_version = 1;  // Currently 1
  MessageType message_type = 2;
}
```

### 11.2 Backward Compatibility

- New fields: Add with default values (Protocol Buffers semantics)
- Removed fields: Mark as `reserved`
- Breaking changes: Bump protocol version

```protobuf
// Version 2 (hypothetical)
message ShareRequest {
  ObjectID object_id = 1;
  string audience_did = 2;
  string capability = 3;
  Timestamp expires_at = 4;
  map<string, string> facts = 5;

  // Version 2 addition
  optional string delegation_proof = 6;
}
```

Servers MUST reject unsupported protocol versions:

```rust
if header.protocol_version > SUPPORTED_VERSION {
    return Err(Error::UnsupportedProtocolVersion(header.protocol_version));
}
```

---

## 12. CLI Integration

### 12.1 CLI Commands via IPC

```bash
# lfs share <object-id> --to <did> --cap read --expires 7d
```

CLI flow:

1. Connect to Unix socket (`~/.latticefs/latticefs.sock`)
2. Send `ShareRequest`
3. Receive `ShareResponse`
4. Print UCAN token to stdout

```rust
async fn cli_share(args: ShareArgs) -> Result<()> {
    // 1. Connect
    let mut conn = connect_ipc().await?;

    // 2. Build request
    let request = ShareRequest {
        object_id: Some(args.object_id.into()),
        audience_did: args.to_did.clone(),
        capability: args.capability.to_string(),
        expires_at: Some(Timestamp::now() + args.expires),
        facts: args.facts,
    };

    // 3. Send request
    send_message(&mut conn, MessageType::ShareRequest, &request).await?;

    // 4. Receive response
    let (msg_type, payload) = recv_message(&mut conn).await?;
    let response = ShareResponse::decode(&payload[..])?;

    // 5. Display result
    match response.result {
        Some(share_response::Result::UcanToken(token)) => {
            println!("{}", token);
            Ok(())
        }
        Some(share_response::Result::Error(err)) => {
            eprintln!("Error: {}", err.message);
            Err(Error::Remote(err.message))
        }
        None => Err(Error::EmptyResponse),
    }
}
```

---

## 13. Go Service Integration

### 13.1 Share Server

Go HTTP server proxies to Rust via IPC:

```go
func handleShareHTTP(w http.ResponseWriter, r *http.Request) {
    var req ShareRequestHTTP
    json.NewDecoder(r.Body).Decode(&req)

    // 1. Connect to Rust IPC
    conn, err := connectIPC()
    if err != nil {
        http.Error(w, err.Error(), 500)
        return
    }
    defer conn.Close()

    // 2. Build IPC request
    ipcReq := &pb.ShareRequest{
        ObjectId:    &pb.ObjectID{Uuid: req.ObjectID},
        AudienceDid: req.AudienceDID,
        Capability:  req.Capability,
        ExpiresAt:   timestampFromDuration(req.TTL),
    }

    // 3. Send IPC request
    sendMessage(conn, MSG_SHARE_REQUEST, ipcReq)

    // 4. Receive IPC response
    msgType, payload, _ := recvMessage(conn)
    var ipcResp pb.ShareResponse
    proto.Unmarshal(payload, &ipcResp)

    // 5. Return HTTP response
    switch result := ipcResp.Result.(type) {
    case *pb.ShareResponse_UcanToken:
        json.NewEncoder(w).Encode(map[string]string{
            "ucan": result.UcanToken,
        })
    case *pb.ShareResponse_Error:
        http.Error(w, result.Error.Message, 400)
    }
}
```

---

## 14. Test Vectors

### 14.1 Valid Share Request

```
Frame:
  Length: 0x0000005A (90 bytes)
  Message Type: 0x0065 (101 = ShareRequest)
  Payload: <protobuf data>

Decoded:
{
  "object_id": {"uuid": "01934e3a-7c5a-7b3c-8d2e-1f4a5b6c7d8e"},
  "audience_did": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "capability": "read",
  "expires_at": {"micros": 1738540800000000},
  "facts": {}
}
```

Expected Response:

```
{
  "ucan_token": "eyJhbGciOiJFZERTQSIs..."
}
```

### 14.2 Error Response

```
Frame:
  Length: 0x00000032 (50 bytes)
  Message Type: 0x03E7 (999 = Error)
  Payload: <protobuf data>

Decoded:
{
  "code": 2,
  "message": "Object not found",
  "details": {"object_id": "01934e3a-7c5a-7b3c-8d2e-1f4a5b6c7d8e"}
}
```

---

## Appendix A: Full Protobuf Schema

See: `specs/protocols/latticefs-ipc.proto`

---

## Appendix B: Socket Path Conventions

```
Linux:   $XDG_RUNTIME_DIR/latticefs.sock or ~/.latticefs/latticefs.sock
macOS:   ~/.latticefs/latticefs.sock
Windows: \\.\pipe\latticefs (Named pipe, not Unix socket)
```

---

**End of LFS-005**
