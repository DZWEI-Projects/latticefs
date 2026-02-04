# NeuralFS Protocol Specifications

This directory contains the formal protocol specifications for NeuralFS, written in RFC-style format. These documents serve as the single source of truth for implementation.

---

## Protocol Index

| ID | Name | Description | Status |
|----|------|-------------|--------|
| [LFS-001](LFS-001-storage.md) | Storage Protocol | Content-defined chunking (FastCDC), content addressing (BLAKE3), storage layout, deduplication | Draft |
| [LFS-002](LFS-002-lql.md) | Lattice Query Language | Query grammar, predicates, boolean logic, graph traversal, explainability | Draft |
| [LFS-003](LFS-003-capability.md) | Capability Security | UCAN tokens, delegation, revocation, time-bounded validity, facts | Draft |
| [LFS-004](LFS-004-object-model.md) | Object Model | Objects, versions, DAG, tags, links, states, metadata partitioning | Draft |
| [LFS-005](LFS-005-ipc.md) | Inter-Process Communication | Unix domain sockets, Protocol Buffers, message framing, Rust-Go IPC | Draft |
| [LFS-006](LFS-006-share-sync.md) | Share and Sync | HTTP share server, CRDT sync protocol, conflict resolution, peer discovery | Draft |

---

## Document Status

- **Draft**: Under active development, subject to change
- **Proposed**: Ready for review
- **Final**: Implementation complete, frozen except for clarifications
- **Deprecated**: Superseded by newer protocol

All protocols are currently in **Draft** status as the project is in pre-implementation phase.

---

## Dependencies Between Protocols

```
LFS-004 (Object Model)
   ├─→ LFS-001 (Storage) - Uses chunk store for versions
   └─→ LFS-003 (Capability) - Uses UCANs for access control

LFS-002 (LQL)
   └─→ LFS-004 (Object Model) - Queries over object graph

LFS-005 (IPC)
   ├─→ LFS-003 (Capability) - Shares via IPC
   └─→ LFS-004 (Object Model) - Transfers objects

LFS-006 (Share/Sync)
   ├─→ LFS-003 (Capability) - HTTP sharing uses UCANs
   ├─→ LFS-004 (Object Model) - Syncs graph operations
   └─→ LFS-005 (IPC) - HTTP server talks to Rust via IPC
```

---

## Implementation Order

Follow this order when implementing:

1. **LFS-001 (Storage)** - Foundation: chunking, hashing, chunk store
2. **LFS-004 (Object Model)** - Core data structures: objects, versions, tags
3. **LFS-003 (Capability)** - Security: UCAN tokens, signing, verification
4. **LFS-002 (LQL)** - Queries: parser, evaluator, views
5. **LFS-005 (IPC)** - Communication: Unix sockets, protobuf framing
6. **LFS-006 (Share/Sync)** - Networking: HTTP share server (MVP only)

---

## Protocol Versioning

Each protocol has:

- **Version**: Semantic versioning (0.1.0 = draft)
- **Date**: Last updated
- **Status**: Draft, Proposed, Final, Deprecated

### Breaking Changes

Breaking changes require:

- Version bump (e.g., 0.1.0 → 0.2.0)
- Migration guide in protocol document
- Support for old version (if deployed)

---

## Test Vectors

Each protocol includes test vectors in appendices:

- **Input/Output examples** for validation
- **Edge cases** (empty input, max size, etc.)
- **Error cases** (malformed data, invalid signatures)

Implementations MUST pass all test vectors.

---

## How to Use These Protocols

### For Implementers

1. Read protocols in dependency order (see diagram above)
2. Implement test vectors first (TDD approach)
3. Cross-reference with other protocols for integration points
4. Use Protocol Buffers schemas from LFS-005 for IPC

### For Reviewers

1. Check for ambiguities or missing details
2. Verify test vectors are comprehensive
3. Validate security considerations
4. Ensure backward compatibility story is clear

### For Users

These are technical specifications. For user-facing documentation, see:

- `../PRD.md` - Product requirements
- `../../README.md` - Project overview
- `../../docs/` - User guides (when available)

---

## Contributing

When updating protocols:

1. **Update the protocol document**
   - Increment version if breaking change
   - Update date
   - Add detailed changelog in document

2. **Update this README**
   - Update version in table
   - Note any new dependencies
   - Update implementation order if needed

3. **Update dependent protocols**
   - If your change affects other protocols, update them too
   - Ensure cross-references are accurate

4. **Add test vectors**
   - Every change should include test vectors
   - Cover both success and error cases

---

## Protocol Format Conventions

All protocols follow this structure:

```markdown
# LFS-XXX: Protocol Name

**Status:** Draft
**Version:** 0.1.0
**Date:** YYYY-MM-DD
**Authors:** NeuralFS Team

---

## Abstract
[One-paragraph summary]

## 1. Introduction
### 1.1 Motivation
### 1.2 Design Goals
### 1.3 Terminology

## 2-N. Technical Sections
[Core protocol specification]

## N+1. Security Considerations

## N+2. Performance Considerations

## N+3. Test Vectors

## Appendix A-Z: Supplementary Material

**End of LFS-XXX**
```

---

## Quick Reference

### Key Constants

```
FastCDC Average Chunk Size: 16 KiB
BLAKE3 Hash Size: 32 bytes (256 bits)
Max Message Size (IPC): 100 MiB
Max Proof Chain Depth: 10 hops
Max Traversal Depth (LQL): 10 hops
Default UCAN TTL: 7 days
HTTP Share Port: 8771
IPC Socket: $LATTICE_HOME/latticefs.sock
```

### Key Algorithms

```
Chunking: FastCDC with Gear hash
Hashing: BLAKE3 (256-bit)
Signing: Ed25519
Encryption: AES-256-GCM
IDs: UUID v7 (time-ordered)
Serialization: Protocol Buffers (IPC), bincode (storage)
```

---

## External Standards Referenced

- [RFC 4122](https://www.rfc-editor.org/rfc/rfc4122) - UUIDs
- [RFC 8610](https://www.rfc-editor.org/rfc/rfc8610) - CBOR (future)
- [Ed25519 Signature Scheme](https://ed25519.cr.yp.to/)
- [BLAKE3 Specification](https://github.com/BLAKE3-team/BLAKE3-specs)
- [UCAN Specification](https://github.com/ucan-wg/spec)
- [FastCDC Paper](https://www.usenix.org/system/files/conference/atc16/atc16-paper-xia.pdf)
- [Protocol Buffers](https://protobuf.dev/)
- [DID Core Specification](https://www.w3.org/TR/did-core/)

---

## License

These protocol specifications are released under the same license as the NeuralFS project.

---

**Questions or Feedback?**

File an issue in the main repository or reach out to the maintainers.

---

*Last Updated: 2026-02-03*
