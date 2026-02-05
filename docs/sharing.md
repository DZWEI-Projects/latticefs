# Sharing

Sharing is capability-based. The system issues UCAN tokens scoped to an object (or view snapshot) and a permission.

## Key properties
- Scoped permissions: `read`, `comment`, `write`, `share`, `admin`
- Time-bounded: every capability has an expiration
- Revocable: revocations are logged and enforced
- Share-only for objects or view snapshots (no folder shares)

## CLI sharing
```bash
# Share an object
lfs share <object-id> --to <did:key:...> --cap read --expires 7d

# Share a view snapshot
lfs share snapshot <view-name|view-id> --to <did:key:...> --cap read --expires 7d

# List stored capabilities
lfs shares list

# Revoke a capability
lfs revoke <cid-or-token>
```

## Share server (HTTP)
The share server exposes a minimal HTTP API.

### POST /share
Create a capability for an object:
```http
POST /share
Authorization: Bearer <ucan>
Content-Type: application/json

{
  "object_id": "<uuid>",
  "audience_did": "did:key:...",
  "capability": "read",
  "expires_in": 604800
}
```

### GET /fetch/{object_id}
Fetch content using a UCAN bearer token:
```http
GET /fetch/<uuid>
Authorization: Bearer <ucan>
```

### POST /revoke
Revoke a capability by CID:
```http
POST /revoke
Authorization: Bearer <ucan>
Content-Type: application/json

{
  "ucan_cid": "<cid>",
  "reason": "optional"
}
```

## Share server (gRPC)
The gRPC surface mirrors the HTTP actions using `ShareService`.
- `Share(ShareRequest) -> ShareResponse`
- `Fetch(FetchRequest) -> FetchResponse`
- `Revoke(RevokeRequest) -> RevokeResponse`

The request/response types are defined in `services/proto/share.proto` and `services/proto/ipc.proto`.

## Revocation logs
Revocations are stored in:
```
~/.latticefs/logs/revocations.jsonl
```

The share server loads this file to enforce revocations.
