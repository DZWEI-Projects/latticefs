# LFS-006: Share and Sync Protocol

**Status:** Draft
**Version:** 0.1.0
**Date:** 2026-02-03
**Authors:** NeuralFS Team

---

## Abstract

This document specifies the NeuralFS share and sync protocols, including HTTP-based capability sharing (MVP), CRDT-based graph synchronization (future), conflict resolution semantics, and peer discovery. The protocol enables secure object sharing between users and eventual multi-device synchronization.

---

## 1. Introduction

### 1.1 Motivation

NeuralFS objects exist in a distributed graph:

- **Sharing**: Users need to share objects via capabilities (zero-trust)
- **Sync**: Users need multi-device access (laptop, phone, desktop)
- **Conflict Resolution**: Concurrent edits must merge gracefully
- **Offline-First**: Devices work offline, sync when online

### 1.2 Phases

**Phase 1 (MVP)**: HTTP share server

- Share objects via UCAN tokens
- HTTP GET to fetch objects
- No bi-directional sync

**Phase 2 (Future)**: CRDT sync

- Multi-device graph replication
- Conflict-free merges
- Peer-to-peer or relay topology

This document specifies both phases.

### 1.3 Design Principles

- **Capability-based**: All access via UCAN tokens (LFS-003)
- **Content-addressed**: Objects identified by hash
- **Immutable**: Versions never change
- **Eventually consistent**: Devices converge
- **Conflict-free**: CRDT semantics for graph operations

---

## 2. Phase 1: HTTP Share Server (MVP)

### 2.1 Overview

Simple HTTP server for sharing objects:

```
Alice                Share Server           Bob
  |                        |                  |
  |--- POST /share ------->|                  |
  |<-- UCAN token ---------|                  |
  |                        |                  |
  |                        |<-- GET /fetch ---|
  |                        |---- object ----->|
```

### 2.2 HTTP API

#### 2.2.1 POST /share

Create a share capability:

**Request:**

```http
POST /share HTTP/1.1
Host: share.latticefs.local:8771
Content-Type: application/json
Authorization: Bearer <alice-ucan>

{
  "object_id": "01934e3a-7c5a-7b3c-8d2e-1f4a5b6c7d8e",
  "audience_did": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "capability": "read",
  "expires_in": 604800
}
```

**Response:**

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "ucan_token": "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCIsInVjdiI6IjAuMTAuMCJ9...",
  "expires_at": "2025-02-10T00:00:00Z"
}
```

#### 2.2.2 GET /fetch

Fetch object using UCAN:

**Request:**

```http
GET /fetch/01934e3a-7c5a-7b3c-8d2e-1f4a5b6c7d8e HTTP/1.1
Host: share.latticefs.local:8771
Authorization: Bearer eyJhbGciOiJFZERTQSIs...
```

**Response:**

```http
HTTP/1.1 200 OK
Content-Type: application/octet-stream
X-NeuralFS-Version: 01934e3b-7c5a-7b3c-8d2e-1f4a5b6c7d8f
X-NeuralFS-Blake3: af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262

<binary object data>
```

#### 2.2.3 POST /revoke

Revoke a capability:

**Request:**

```http
POST /revoke HTTP/1.1
Host: share.latticefs.local:8771
Content-Type: application/json
Authorization: Bearer <alice-ucan>

{
  "ucan_cid": "bafyreif...",
  "reason": "Shared by mistake"
}
```

**Response:**

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "revoked": true,
  "revoked_at": "2025-02-03T10:30:00Z"
}
```

### 2.3 Server Implementation (Go)

```go
package main

import (
    "net/http"
    "github.com/gorilla/mux"
)

func main() {
    r := mux.NewRouter()

    r.HandleFunc("/share", handleShare).Methods("POST")
    r.HandleFunc("/fetch/{object_id}", handleFetch).Methods("GET")
    r.HandleFunc("/revoke", handleRevoke).Methods("POST")

    http.ListenAndServe(":8771", r)
}

func handleShare(w http.ResponseWriter, r *http.Request) {
    // 1. Authenticate request (verify Alice's UCAN)
    authToken := r.Header.Get("Authorization")
    issuerUCAN, err := parseAuthToken(authToken)
    if err != nil {
        http.Error(w, "Unauthorized", 401)
        return
    }

    // 2. Parse request
    var req ShareRequest
    json.NewDecoder(r.Body).Decode(&req)

    // 3. Call Rust IPC to create UCAN
    conn, _ := connectIPC()
    defer conn.Close()

    ipcReq := &pb.ShareRequest{
        ObjectId:    &pb.ObjectID{Uuid: req.ObjectID},
        AudienceDid: req.AudienceDID,
        Capability:  req.Capability,
        ExpiresAt:   timestampFromDuration(req.ExpiresIn),
    }

    sendMessage(conn, MSG_SHARE_REQUEST, ipcReq)
    msgType, payload, _ := recvMessage(conn)

    var ipcResp pb.ShareResponse
    proto.Unmarshal(payload, &ipcResp)

    // 4. Return UCAN token
    switch result := ipcResp.Result.(type) {
    case *pb.ShareResponse_UcanToken:
        json.NewEncoder(w).Encode(ShareResponse{
            UCANToken: result.UcanToken,
            ExpiresAt: expiresAtFromTimestamp(ipcReq.ExpiresAt),
        })
    case *pb.ShareResponse_Error:
        http.Error(w, result.Error.Message, 400)
    }
}

func handleFetch(w http.ResponseWriter, r *http.Request) {
    // 1. Extract object ID
    vars := mux.Vars(r)
    objectID, _ := uuid.Parse(vars["object_id"])

    // 2. Extract and verify UCAN
    ucanToken := r.Header.Get("Authorization")
    if !strings.HasPrefix(ucanToken, "Bearer ") {
        http.Error(w, "Missing UCAN", 401)
        return
    }
    ucanToken = strings.TrimPrefix(ucanToken, "Bearer ")

    // 3. Call Rust IPC to fetch object
    conn, _ := connectIPC()
    defer conn.Close()

    ipcReq := &pb.FetchRequest{
        ObjectId:  &pb.ObjectID{Uuid: objectID[:]},
        UcanToken: ucanToken,
    }

    sendMessage(conn, MSG_FETCH_REQUEST, ipcReq)
    msgType, payload, _ := recvMessage(conn)

    var ipcResp pb.FetchResponse
    proto.Unmarshal(payload, &ipcResp)

    // 4. Return object data
    switch result := ipcResp.Result.(type) {
    case *pb.FetchResponse_Data:
        w.Header().Set("Content-Type", "application/octet-stream")
        w.Header().Set("X-NeuralFS-Version", uuid.UUID(result.Data.VersionId.Uuid).String())
        w.Header().Set("X-NeuralFS-Blake3", hex.EncodeToString(result.Data.ContentHash.Blake3))
        w.Write(result.Data.Content)
    case *pb.FetchResponse_Error:
        http.Error(w, result.Error.Message, 400)
    }
}
```

### 2.4 Client Usage (CLI)

```bash
# Alice shares object with Bob
$ lfs share 01934e3a-7c5a-7b3c-8d2e-1f4a5b6c7d8e \
    --to did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK \
    --cap read \
    --expires 7d

UCAN Token: eyJhbGciOiJFZERTQSIs...

# Bob fetches object
$ lfs fetch 01934e3a-7c5a-7b3c-8d2e-1f4a5b6c7d8e \
    --ucan eyJhbGciOiJFZERTQSIs... \
    --output report.pdf

Fetched: report.pdf (1.2 MB)
```

---

## 3. Phase 2: CRDT Sync (Future)

### 3.1 Overview

Multi-device graph synchronization using CRDTs (Conflict-free Replicated Data Types):

```
Device A                Sync Relay              Device B
   |                         |                      |
   |--- Push graph ops ----->|                      |
   |                         |--- Push ops -------->|
   |                         |                      |
   |                         |<--- Pull ops --------|
   |<--- Pull ops -----------|                      |
```

### 3.2 CRDT Graph Model

NeuralFS graph operations are commutative:

**Operations:**

- `AddObject(id, version, content_hash)`
- `AddVersion(object_id, version_id, parent, manifest)`
- `AddTag(object_id, tag_key, tag_value)`
- `RemoveTag(object_id, tag_key)`
- `AddLink(source, target, link_type)`
- `RemoveLink(link_id)`

**CRDT Semantics:**

- **Add-wins**: Adding a tag always wins over removal (if concurrent)
- **Last-write-wins**: For mutable fields (e.g., metadata), timestamp determines winner
- **Version DAG**: Merges are implicit (versions can have multiple parents)

### 3.3 Operation Log

Each device maintains an append-only operation log:

```rust
struct Operation {
    id: OperationID,        // UUID v7 (time-ordered)
    device_id: DeviceID,    // Device that created operation
    timestamp: Timestamp,   // Lamport timestamp
    op_type: OperationType, // AddObject, AddTag, etc.
    payload: Vec<u8>,       // Operation-specific data
    dependencies: Vec<OperationID>, // Causal dependencies
    signature: Signature,   // Ed25519 signature
}
```

### 3.4 Sync Protocol

#### 3.4.1 Pull

Device requests operations after specific timestamp:

**Request:**

```json
{
  "device_id": "device-a",
  "since": "2025-02-03T10:00:00Z",
  "max_ops": 1000
}
```

**Response:**

```json
{
  "operations": [
    {
      "id": "01934e3a-7c5a-7b3c-8d2e-1f4a5b6c7d8e",
      "device_id": "device-b",
      "timestamp": 1738540800,
      "op_type": "AddTag",
      "payload": "<base64-encoded data>",
      "dependencies": [],
      "signature": "<base64-signature>"
    },
    ...
  ],
  "more": false
}
```

#### 3.4.2 Push

Device sends local operations to relay:

**Request:**

```json
{
  "device_id": "device-a",
  "operations": [
    {
      "id": "01934e3b-7c5a-7b3c-8d2e-1f4a5b6c7d8f",
      "device_id": "device-a",
      "timestamp": 1738540900,
      "op_type": "AddObject",
      "payload": "<base64-encoded data>",
      "dependencies": ["01934e3a-7c5a-7b3c-8d2e-1f4a5b6c7d8e"],
      "signature": "<base64-signature>"
    }
  ]
}
```

**Response:**

```json
{
  "accepted": 1,
  "rejected": 0
}
```

### 3.5 Conflict Resolution

#### 3.5.1 Concurrent Tag Additions

```
Device A: AddTag(obj, "priority", "high") @ t=100
Device B: AddTag(obj, "priority", "low")  @ t=100

Result: Both tags exist (multi-valued CRDT)
User resolves: lfs tag obj priority high (removes "low")
```

#### 3.5.2 Concurrent Version Creation

```
Device A: AddVersion(obj, v2, parent=v1) @ t=100
Device B: AddVersion(obj, v2', parent=v1) @ t=100

Result: DAG branches (both v2 and v2' are valid)

v1 ← v2
  ↖
    v2'

User merges: lfs merge v2 v2' (creates v3 with parents=[v2, v2'])
```

#### 3.5.3 Tag Removal vs. Addition

```
Device A: AddTag(obj, "draft", "true") @ t=100
Device B: RemoveTag(obj, "draft")       @ t=100

Result: Add-wins (tag exists)
Reason: Conservative (prefer preserving data)
```

### 3.6 Lamport Timestamps

Lamport timestamps ensure causal ordering:

```rust
struct LamportClock {
    counter: u64,
    device_id: DeviceID,
}

impl LamportClock {
    fn tick(&mut self) -> Timestamp {
        self.counter += 1;
        Timestamp {
            counter: self.counter,
            device_id: self.device_id,
        }
    }

    fn update(&mut self, remote: Timestamp) {
        self.counter = self.counter.max(remote.counter) + 1;
    }
}
```

### 3.7 Causal Dependencies

Operations reference dependencies:

```rust
struct Operation {
    dependencies: Vec<OperationID>,
    // ...
}

// Example: AddVersion depends on AddObject
let add_object_op = Operation {
    id: op1_id,
    op_type: OperationType::AddObject,
    dependencies: vec![],
    // ...
};

let add_version_op = Operation {
    id: op2_id,
    op_type: OperationType::AddVersion,
    dependencies: vec![op1_id],  // Depends on object creation
    // ...
};
```

Sync MUST apply operations in causal order.

### 3.8 Tombstones

Deleted objects use tombstones (not true deletion):

```rust
struct Tombstone {
    object_id: ObjectID,
    deleted_at: Timestamp,
    deleted_by: DeviceID,
}

// Operation: MarkDeleted
let delete_op = Operation {
    op_type: OperationType::MarkDeleted,
    payload: bincode::serialize(&Tombstone {
        object_id,
        deleted_at: clock.tick(),
        deleted_by: device_id,
    }),
    // ...
};
```

Tombstones prevent re-adding deleted objects.

---

## 4. Peer Discovery

### 4.1 mDNS/DNS-SD (Local Network)

Devices on same LAN discover via mDNS:

```
Service Type: _latticefs._tcp.local.
Instance Name: Alice's MacBook Pro
Port: 8771
TXT Records:
  - version=0.1
  - device-id=device-a
  - public-key=z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK
```

### 4.2 Relay Servers (Internet)

Devices register with relay for NAT traversal:

```
Device → Relay: "I am device-a, reachable at <relay>/device-a"
Relay → Other Devices: "device-a is online"
```

### 4.3 Direct Connections (WebRTC, future)

P2P connections via WebRTC data channels:

```
Device A                  STUN/TURN               Device B
   |                          |                       |
   |--- Offer --------------->|                       |
   |                          |--- Offer ------------>|
   |                          |<--- Answer -----------|
   |<--- Answer --------------|                       |
   |                                                  |
   |<========= Direct P2P Connection ===============>|
```

---

## 5. Security Considerations

### 5.1 Share Server Authentication

All requests MUST include valid UCAN:

```go
func authenticateRequest(r *http.Request) (*UCAN, error) {
    authHeader := r.Header.Get("Authorization")
    if !strings.HasPrefix(authHeader, "Bearer ") {
        return nil, errors.New("missing bearer token")
    }

    token := strings.TrimPrefix(authHeader, "Bearer ")
    ucan, err := parseUCAN(token)
    if err != nil {
        return nil, err
    }

    // Validate UCAN
    if err := validateUCAN(ucan); err != nil {
        return nil, err
    }

    return ucan, nil
}
```

### 5.2 Rate Limiting

```go
var rateLimiter = rate.NewLimiter(100, 200)  // 100 req/s, burst 200

func rateLimitMiddleware(next http.Handler) http.Handler {
    return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        if !rateLimiter.Allow() {
            http.Error(w, "Rate limit exceeded", 429)
            return
        }
        next.ServeHTTP(w, r)
    })
}
```

### 5.3 Operation Signing

All sync operations MUST be signed:

```rust
fn sign_operation(op: &Operation, secret_key: &SecretKey) -> Signature {
    let payload = bincode::serialize(op)?;
    secret_key.sign(&payload)
}

fn verify_operation(op: &Operation, public_key: &PublicKey) -> bool {
    let payload = bincode::serialize(op)?;
    public_key.verify(&payload, &op.signature).is_ok()
}
```

### 5.4 Sync Replay Protection

Devices track received operations:

```rust
struct SyncState {
    received_ops: HashSet<OperationID>,
}

fn apply_operation(op: &Operation, state: &mut SyncState) -> Result<()> {
    // Check for replay
    if state.received_ops.contains(&op.id) {
        return Ok(());  // Already applied
    }

    // Apply operation
    execute_op(op)?;

    // Record as received
    state.received_ops.insert(op.id);
    Ok(())
}
```

---

## 6. Performance Considerations

### 6.1 Chunk Transfer

Large objects: Transfer chunks, not full content:

```
GET /chunk/af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262
```

Client assembles object from chunks.

### 6.2 Operation Batching

Sync pulls multiple operations per request:

```json
{
  "since": "2025-02-03T10:00:00Z",
  "max_ops": 1000  // Batch size
}
```

### 6.3 Incremental Sync

Devices only pull operations after last sync:

```rust
struct SyncCheckpoint {
    last_sync: Timestamp,
    last_op_id: OperationID,
}
```

### 6.4 Compression

HTTP responses SHOULD use gzip compression:

```http
Content-Encoding: gzip
```

---

## 7. Test Vectors

### 7.1 Share Request

```http
POST /share HTTP/1.1
Content-Type: application/json

{
  "object_id": "01934e3a-7c5a-7b3c-8d2e-1f4a5b6c7d8e",
  "audience_did": "did:key:z6Mkf5rGMoatrSj1f4CyvuHBeXJELe9RPdzo2PKGNCKVtZxP",
  "capability": "read",
  "expires_in": 604800
}
```

Expected: `200 OK` with UCAN token

### 7.2 Fetch Request

```http
GET /fetch/01934e3a-7c5a-7b3c-8d2e-1f4a5b6c7d8e HTTP/1.1
Authorization: Bearer eyJhbGciOiJFZERTQSIs...
```

Expected: `200 OK` with binary content

### 7.3 Sync Pull

```json
{
  "device_id": "device-a",
  "since": "2025-02-03T10:00:00Z",
  "max_ops": 10
}
```

Expected: List of operations

---

## 8. Future Extensions

### 8.1 Selective Sync

Only sync specific views:

```json
{
  "sync_filters": [
    {"view": "Projects"},
    {"tag": "priority:high"}
  ]
}
```

### 8.2 Differential Sync

Only send operation deltas:

```json
{
  "since_checkpoint": "checkpoint-xyz",
  "diff_only": true
}
```

### 8.3 End-to-End Encryption

Encrypt sync payloads:

```json
{
  "operation": {
    "id": "...",
    "encrypted_payload": "<base64>",
    "encryption_key_ref": "key-id"
  }
}
```

---

## Appendix A: Share Server Configuration

```toml
# $LATTICE_HOME/share.toml

[server]
listen_addr = "0.0.0.0:8771"
tls_cert = "/path/to/cert.pem"
tls_key = "/path/to/key.pem"

[rate_limit]
requests_per_second = 100
burst = 200

[sync]
max_operations_per_pull = 1000
checkpoint_interval = 300  # 5 minutes

[storage]
ipc_socket = "/Users/alice/.latticefs/latticefs.sock"
```

---

## Appendix B: Operation Encoding

```rust
// AddObject operation payload
struct AddObjectPayload {
    object_id: ObjectID,
    version_id: VersionID,
    content_hash: Hash,
    created_at: Timestamp,
    object_type: ObjectType,
}

// AddTag operation payload
struct AddTagPayload {
    object_id: ObjectID,
    tag_key: String,
    tag_value: String,
    added_at: Timestamp,
}

// All payloads serialized with bincode
```

---

**End of LFS-006**
