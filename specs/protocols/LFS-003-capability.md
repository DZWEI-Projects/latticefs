# LFS-003: Capability-Based Security Protocol

**Status:** Draft
**Version:** 0.1.0
**Date:** 2026-02-03
**Authors:** NeuralFS Team

---

## Abstract

This document specifies the capability-based security model for NeuralFS, including UCAN (User Controlled Authorization Network) token format, delegation semantics, revocation mechanisms, and cryptographic operations. Capabilities replace traditional ACLs with cryptographically verifiable, delegatable bearer tokens.

---

## 1. Introduction

### 1.1 Motivation

Traditional access control uses:

- **ACLs**: Centralized, server-checked permissions
- **Bearer tokens**: Non-delegatable, opaque
- **Passwords**: Shared secrets, revocation requires coordination

NeuralFS requires:

- **Zero-trust sharing**: No central authority
- **Delegation**: Recipients can re-share with constraints
- **Revocation**: Instant capability invalidation
- **Offline verification**: Validate without contacting issuer
- **Auditability**: Cryptographic proof of authorization chain

### 1.2 UCAN Overview

UCAN (User Controlled Authorization Network) provides:

- JWT-like structure (header, payload, signature)
- Ed25519 signatures for authenticity
- Chained delegation (proof of authority)
- Time-bounded validity
- Revocable via revocation lists

### 1.3 Terminology

- **Issuer**: Entity creating the capability
- **Audience**: Entity receiving the capability
- **Subject**: Object being authorized
- **Capability**: Permission (read, write, comment, etc.)
- **Proof**: Parent UCAN(s) proving issuer's authority
- **Attenuation**: Reducing permissions during delegation

---

## 2. Cryptographic Primitives

### 2.1 Identity

Each actor has an Ed25519 keypair:

```rust
struct Identity {
    public_key: [u8; 32],   // Ed25519 public key
    secret_key: [u8; 64],   // Ed25519 secret key (stored securely)
}
```

**Public Key Encoding:** Multibase base58btc:

```
Example: z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK
         ^-- 'z' prefix indicates base58btc
```

### 2.2 Signing

All UCANs are signed using Ed25519:

```rust
fn sign_ucan(ucan_payload: &[u8], secret_key: &SecretKey) -> Signature {
    let signature = secret_key.sign(ucan_payload);
    signature  // 64 bytes
}
```

### 2.3 Verification

```rust
fn verify_ucan(ucan_payload: &[u8], signature: &Signature, public_key: &PublicKey) -> bool {
    public_key.verify(ucan_payload, signature).is_ok()
}
```

---

## 3. UCAN Token Format

### 3.1 Structure

A UCAN is a JWT-like token:

```
<base64url(header)>.<base64url(payload)>.<base64url(signature)>
```

### 3.2 Header

```json
{
  "alg": "EdDSA",
  "typ": "JWT",
  "ucv": "0.10.0"
}
```

**Fields:**

- `alg`: Algorithm (MUST be `EdDSA`)
- `typ`: Type (MUST be `JWT`)
- `ucv`: UCAN version (MUST be `0.10.0`)

### 3.3 Payload

```json
{
  "iss": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "aud": "did:key:z6Mkf5rGMoatrSj1f4CyvuHBeXJELe9RPdzo2PKGNCKVtZxP",
  "sub": "latticefs:object:01934e3a-7c5a-7b3c-8d2e-1f4a5b6c7d8e",
  "exp": 1738540800,
  "nbf": 1738454400,
  "nnc": "550e8400-e29b-41d4-a716-446655440000",
  "fct": {
    "lfs/version": "0.1",
    "lfs/device": "laptop-primary"
  },
  "att": [
    {
      "with": "latticefs:object:01934e3a-7c5a-7b3c-8d2e-1f4a5b6c7d8e",
      "can": "read"
    }
  ],
  "prf": [
    "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCIsInVjdiI6IjAuMTAuMCJ9..."
  ]
}
```

**Required Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `iss` | DID | Issuer's decentralized identifier |
| `aud` | DID | Audience's decentralized identifier |
| `exp` | Unix timestamp | Expiration time (seconds since epoch) |
| `att` | Array | Attenuations (capabilities) |

**Optional Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `sub` | String | Subject (object being authorized) |
| `nbf` | Unix timestamp | Not-before time |
| `nnc` | String | Nonce (prevents replay) |
| `fct` | Object | Facts (additional context) |
| `prf` | Array[String] | Proof chain (parent UCANs) |

### 3.4 Signature

```rust
let payload = format!("{}.{}",
    base64url(header),
    base64url(payload)
);
let signature = ed25519_sign(&payload, secret_key);
let token = format!("{}.{}", payload, base64url(signature));
```

---

## 4. Capabilities (Attenuations)

### 4.1 Capability Structure

```json
{
  "with": "latticefs:object:<object-id>",
  "can": "read"
}
```

**Fields:**

- `with`: Resource URI (object, view, or namespace)
- `can`: Permission (read, write, comment, admin)

### 4.2 Supported Permissions

| Permission | Allows |
|------------|--------|
| `read` | Read object content, metadata, versions |
| `write` | Create new versions, modify metadata |
| `comment` | Add comments (future) |
| `admin` | Change policies, revoke capabilities |
| `share` | Create new capabilities for this object |

### 4.3 Permission Hierarchy

```
admin > share > write > comment > read
```

**Attenuation Rule:** Delegated capabilities MUST NOT exceed issuer's capabilities.

### 4.4 Resource URIs

```
latticefs:object:<uuid>              # Specific object
latticefs:view:<view-name>           # View snapshot
latticefs:namespace:tag:project:*    # Namespace (wildcard)
```

### 4.5 Wildcards

```json
{
  "with": "latticefs:namespace:tag:project:phoenix",
  "can": "read"
}
```

Matches all objects with tags starting with `tag:project:phoenix`.

---

## 5. Delegation

### 5.1 Delegation Chain

Alice → Bob → Charlie:

```
1. Alice issues UCAN to Bob:
   {
     "iss": "did:key:alice",
     "aud": "did:key:bob",
     "att": [{"with": "latticefs:object:X", "can": "read"}]
   }

2. Bob delegates to Charlie:
   {
     "iss": "did:key:bob",
     "aud": "did:key:charlie",
     "att": [{"with": "latticefs:object:X", "can": "read"}],
     "prf": ["<Alice's UCAN>"]
   }
```

### 5.2 Proof Verification

To verify Charlie's UCAN:

1. Verify Charlie's UCAN signature (Bob signed it)
2. Extract proof (Alice's UCAN)
3. Verify Alice's UCAN signature
4. Check: Bob is audience of Alice's UCAN ✓
5. Check: Charlie's capabilities ⊆ Bob's capabilities ✓
6. Check: Expiration times valid ✓

### 5.3 Attenuation

Bob can reduce permissions when delegating:

```
Alice grants Bob: {can: "write"}
Bob grants Charlie: {can: "read"}  ← Valid (read < write)

Alice grants Bob: {can: "read"}
Bob grants Charlie: {can: "write"}  ← INVALID (write > read)
```

### 5.4 Reducing Scope

```
Alice grants Bob: {with: "latticefs:namespace:tag:project:*", can: "read"}
Bob grants Charlie: {with: "latticefs:object:01934e3a...", can: "read"}
                     ← Valid (specific object < wildcard namespace)
```

---

## 6. Revocation

### 6.1 Revocation List

Revoked capabilities are stored in a revocation list:

```rust
struct RevocationList {
    revocations: Vec<Revocation>,
}

struct Revocation {
    ucan_cid: String,       // CID of revoked UCAN
    revoked_at: Timestamp,  // When revoked
    revoked_by: DID,        // Who revoked it
    reason: Option<String>, // Optional reason
}
```

### 6.2 Revocation Check

```rust
fn is_revoked(ucan_cid: &str, revocation_list: &RevocationList) -> bool {
    revocation_list.revocations
        .iter()
        .any(|r| r.ucan_cid == ucan_cid)
}
```

### 6.3 Cascading Revocation

Revoking a UCAN MUST revoke all derived UCANs:

```
Alice → Bob → Charlie → Dave

If Bob's UCAN is revoked:
  - Charlie's UCAN becomes invalid (derives from Bob)
  - Dave's UCAN becomes invalid (derives from Charlie)
```

### 6.4 Revocation Distribution

Revocation lists MUST be:

- Append-only (for auditability)
- Signed by revoker (authenticity)
- Distributed via share service (sync)

---

## 7. Time-Bounded Validity

### 7.1 Expiration (`exp`)

UCANs MUST have an expiration time:

```json
{
  "exp": 1738540800  // Unix timestamp: 2025-02-03 00:00:00 UTC
}
```

**Validation:**

```rust
fn is_expired(ucan: &UCAN, now: Timestamp) -> bool {
    now >= ucan.exp
}
```

### 7.2 Not-Before (`nbf`)

Optional: UCAN not valid until specified time:

```json
{
  "nbf": 1738454400,  // Valid starting: 2025-02-02 00:00:00 UTC
  "exp": 1738540800   // Expires: 2025-02-03 00:00:00 UTC
}
```

### 7.3 Recommended Expiration Periods

| Use Case | Expiration |
|----------|------------|
| One-time share | 1 hour |
| Temporary collaboration | 7 days |
| Long-term access | 90 days |
| Permanent access | Use policies, not capabilities |

---

## 8. Facts (Contextual Constraints)

### 8.1 Device Binding

Bind capability to specific device:

```json
{
  "fct": {
    "lfs/device": "laptop-primary",
    "lfs/device-fingerprint": "sha256:af1349b9f5f9a1a6..."
  }
}
```

**Validation:** Verify device fingerprint matches current device.

### 8.2 IP Constraints

Restrict to specific network:

```json
{
  "fct": {
    "lfs/ip-range": "192.168.1.0/24"
  }
}
```

**Validation:** Check client IP against allowed range.

### 8.3 Custom Facts

```json
{
  "fct": {
    "lfs/version": "0.1",
    "lfs/purpose": "code-review",
    "lfs/max-operations": 100
  }
}
```

Facts are application-specific and MUST be validated by the service.

---

## 9. UCAN Lifecycle

### 9.1 Issuance

```rust
async fn issue_ucan(
    issuer: &Identity,
    audience: &PublicKey,
    object_id: &ObjectID,
    capability: Capability,
    expires_in: Duration,
) -> Result<UCAN> {
    let now = SystemTime::now();
    let exp = now + expires_in;

    let payload = UCANPayload {
        iss: did_from_key(&issuer.public_key),
        aud: did_from_key(audience),
        sub: format!("latticefs:object:{}", object_id),
        exp: exp.as_secs(),
        nbf: Some(now.as_secs()),
        nnc: Some(Uuid::new_v7().to_string()),
        att: vec![Attenuation {
            with: format!("latticefs:object:{}", object_id),
            can: capability,
        }],
        prf: vec![],
        fct: None,
    };

    let header = UCANHeader {
        alg: "EdDSA".to_string(),
        typ: "JWT".to_string(),
        ucv: "0.10.0".to_string(),
    };

    let token = encode_and_sign(header, payload, &issuer.secret_key)?;
    Ok(UCAN::from_token(token))
}
```

### 9.2 Validation

```rust
async fn validate_ucan(ucan: &UCAN) -> Result<()> {
    // 1. Verify signature
    verify_signature(ucan)?;

    // 2. Check expiration
    if is_expired(ucan, SystemTime::now()) {
        return Err(Error::UCANExpired);
    }

    // 3. Check not-before
    if let Some(nbf) = ucan.nbf {
        if SystemTime::now() < nbf {
            return Err(Error::UCANNotYetValid);
        }
    }

    // 4. Check revocation
    if is_revoked(&ucan.cid(), &load_revocation_list().await?) {
        return Err(Error::UCANRevoked);
    }

    // 5. Verify proof chain
    if !ucan.prf.is_empty() {
        verify_proof_chain(ucan).await?;
    }

    Ok(())
}
```

### 9.3 Delegation

```rust
async fn delegate_ucan(
    issuer_ucan: &UCAN,
    issuer_identity: &Identity,
    new_audience: &PublicKey,
    attenuated_capability: Capability,
) -> Result<UCAN> {
    // 1. Verify issuer has authority
    validate_ucan(issuer_ucan).await?;

    // 2. Check attenuation (new cap must be ≤ original)
    if !is_attenuated(&attenuated_capability, &issuer_ucan.att[0].can) {
        return Err(Error::InvalidAttenuation);
    }

    // 3. Create new UCAN with proof
    let payload = UCANPayload {
        iss: did_from_key(&issuer_identity.public_key),
        aud: did_from_key(new_audience),
        sub: issuer_ucan.sub.clone(),
        exp: issuer_ucan.exp.min(SystemTime::now() + Duration::from_days(7)),
        att: vec![Attenuation {
            with: issuer_ucan.att[0].with.clone(),
            can: attenuated_capability,
        }],
        prf: vec![issuer_ucan.encode()],
        ..Default::default()
    };

    let token = encode_and_sign(UCAN_HEADER, payload, &issuer_identity.secret_key)?;
    Ok(UCAN::from_token(token))
}
```

### 9.4 Revocation

```rust
async fn revoke_ucan(ucan_cid: &str, revoker: &Identity) -> Result<()> {
    // 1. Verify revoker is issuer or has admin capability
    let ucan = load_ucan(ucan_cid).await?;
    if ucan.iss != did_from_key(&revoker.public_key) {
        return Err(Error::Unauthorized);
    }

    // 2. Add to revocation list
    let revocation = Revocation {
        ucan_cid: ucan_cid.to_string(),
        revoked_at: SystemTime::now(),
        revoked_by: did_from_key(&revoker.public_key),
        reason: None,
    };

    append_to_revocation_list(revocation).await?;

    // 3. Broadcast revocation (via sync service)
    broadcast_revocation(ucan_cid).await?;

    Ok(())
}
```

---

## 10. Integration with NeuralFS

### 10.1 Object Access

```rust
async fn read_object(object_id: &ObjectID, ucan: &UCAN) -> Result<Object> {
    // 1. Validate UCAN
    validate_ucan(ucan).await?;

    // 2. Check capability
    if !has_capability(ucan, object_id, Capability::Read) {
        return Err(Error::Unauthorized);
    }

    // 3. Read object
    let object = storage::read_object(object_id).await?;

    // 4. Decrypt if necessary
    let decryption_key = derive_key_from_ucan(ucan)?;
    let decrypted = decrypt_object(&object, &decryption_key)?;

    Ok(decrypted)
}
```

### 10.2 Sharing Workflow

```bash
# 1. Alice shares with Bob
lfs share <object-id> --to <bob-pubkey> --cap read --expires 7d

# 2. Bob receives UCAN token
UCAN: eyJhbGciOiJFZERTQSIs...

# 3. Bob uses UCAN to access object
lfs get <object-id> --ucan <token>

# 4. Bob delegates to Charlie (read-only)
lfs delegate <token> --to <charlie-pubkey> --cap read

# 5. Alice revokes Bob's access
lfs revoke <token>
# → Bob and Charlie lose access immediately
```

---

## 11. Security Considerations

### 11.1 Bearer Token Risk

UCANs are bearer tokens: possession = authorization.

**Mitigations:**

- Short expiration times (default: 7 days)
- Device binding (optional)
- Revocation lists
- Audit logs

### 11.2 Proof Chain Length

Long delegation chains increase verification cost.

**Mitigation:** Limit proof chain depth to 10.

```rust
fn verify_proof_chain(ucan: &UCAN) -> Result<()> {
    if ucan.proof_chain_length() > 10 {
        return Err(Error::ProofChainTooLong);
    }
    // ... verify each link
}
```

### 11.3 Clock Skew

Expiration checks depend on accurate clocks.

**Mitigation:** Allow 5-minute clock skew:

```rust
fn is_expired(ucan: &UCAN, now: Timestamp) -> bool {
    now >= ucan.exp + Duration::from_secs(300)  // 5 min grace
}
```

### 11.4 Revocation Latency

Revocations propagate via sync service.

**Mitigation:**

- Max revocation check age: 1 hour
- Require online validation for sensitive operations

---

## 12. Performance Considerations

### 12.1 Caching

Validated UCANs SHOULD be cached:

```rust
struct UCANCache {
    cache: LruCache<CID, ValidationResult>,
    ttl: Duration,
}
```

**Cache Invalidation:**

- On revocation broadcast
- On expiration
- After TTL (max 1 hour)

### 12.2 Batch Validation

Validate multiple UCANs in parallel:

```rust
async fn validate_batch(ucans: &[UCAN]) -> Vec<Result<()>> {
    futures::future::join_all(
        ucans.iter().map(|u| validate_ucan(u))
    ).await
}
```

---

## 13. Test Vectors

### 13.1 Valid UCAN

```json
{
  "header": {
    "alg": "EdDSA",
    "typ": "JWT",
    "ucv": "0.10.0"
  },
  "payload": {
    "iss": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "aud": "did:key:z6Mkf5rGMoatrSj1f4CyvuHBeXJELe9RPdzo2PKGNCKVtZxP",
    "sub": "latticefs:object:01934e3a-7c5a-7b3c-8d2e-1f4a5b6c7d8e",
    "exp": 1738540800,
    "att": [
      {
        "with": "latticefs:object:01934e3a-7c5a-7b3c-8d2e-1f4a5b6c7d8e",
        "can": "read"
      }
    ]
  },
  "signature": "..."
}
```

Expected: VALID

### 13.2 Expired UCAN

```json
{
  "exp": 1609459200  // 2021-01-01 (expired)
}
```

Expected: INVALID (expired)

### 13.3 Invalid Attenuation

```json
// Parent UCAN
{"att": [{"can": "read"}]}

// Child UCAN (attempting escalation)
{"att": [{"can": "write"}], "prf": ["<parent>"]}
```

Expected: INVALID (escalation not allowed)

---

## Appendix A: DID Format

NeuralFS uses `did:key` for Ed25519 keys:

```
did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK
       └─ Multibase base58btc encoding of Ed25519 public key
```

**Conversion:**

```rust
fn did_from_key(public_key: &[u8; 32]) -> String {
    let multicodec_prefix = &[0xed, 0x01];  // Ed25519-pub
    let bytes = [multicodec_prefix, public_key.as_slice()].concat();
    format!("did:key:z{}", bs58::encode(bytes).into_string())
}
```

---

## Appendix B: References

- [UCAN Specification](https://github.com/ucan-wg/spec)
- [Ed25519 Signature Scheme](https://ed25519.cr.yp.to/)
- [DIDs (Decentralized Identifiers)](https://www.w3.org/TR/did-core/)
- [Multibase Encoding](https://github.com/multiformats/multibase)

---

**End of LFS-003**
