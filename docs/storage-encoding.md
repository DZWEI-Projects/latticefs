# Storage Encoding Map

This document maps each core data structure to its on-disk encoding and storage location.

## Encoding conventions
- **Binary serialization**: `bincode` for Rust structs
- **Hashes**: BLAKE3, stored as raw 32 bytes in structs, hex when used as filenames/keys
- **Key-value store**: `sled` trees under `~/.latticefs/meta/`

## Objects and versions
- **Object** (`base::model::Object`)
  - Stored in sled tree: `meta/objects`
  - Key: `ObjectID` raw bytes (UUID v7, 16 bytes)
  - Value: `bincode(Object)`

- **Version** (`base::model::Version`)
  - Stored in sled tree: `meta/versions`
  - Key: `VersionID` raw bytes (UUID v7, 16 bytes)
  - Value: `bincode(Version)`

## Chunk manifests
- **ChunkManifest** (`base::storage::ChunkManifest`)
  - Stored in sled tree: `meta/manifests`
  - Key: hex(BLAKE3(manifest_bytes))
  - Value: `bincode(ChunkManifest)`

## Chunk data
- **Chunk bytes**
  - Stored as files under `chunks/aa/bb/<full_hash>`
  - Filename: hex(BLAKE3(chunk))
  - Content: raw chunk bytes

## Tag index
- **Tag index entries**
  - Stored in sled tree: `meta/tags`
  - Key: UTF-8 bytes of `tag:<namespace>:<key>:<value>`
  - Value: `bincode(Vec<ObjectID bytes>)`

## Links
- **Link** (`base::model::Link`)
  - Stored in sled tree: `meta/links`
  - Key: `LinkID` raw bytes (UUID v7, 16 bytes)
  - Value: `bincode(Link)`

## Policies
- **Policy** (`base::model::Policy`)
  - Stored in sled tree: `meta/policies`
  - Key: policy name (UTF-8 bytes)
  - Value: `bincode(Policy)`

## Views
- **View** (`base::views::View`)
  - Stored in sled tree: `meta/views`
  - Key: view name (UTF-8 bytes)
  - Value: `bincode(View)`

- **ViewSnapshot** (`base::views::ViewSnapshot`)
  - Stored in sled tree: `meta/snapshots`
  - Key: snapshot ID as UTF-8 string
  - Value: `bincode(ViewSnapshot)`

## Extracted text
- **Text content**
  - Stored in sled tree: `meta/text`
  - Key: `ObjectID` raw bytes
  - Value: UTF-8 text bytes

## FUSE inode mapping
- **Inode -> ObjectID**
  - Stored in sled tree: `meta/inodes`
  - Key: inode as big-endian `u64` bytes
  - Value: ObjectID raw bytes

## Capabilities and revocations
- **Capability (UCAN)**
  - Stored in sled tree: `meta/capabilities`
  - Key: CID (hex BLAKE3 of token) as UTF-8 bytes
  - Value: raw UCAN token string bytes

- **Revocation** (`base::crypto::Revocation`)
  - Stored in sled tree: `meta/revocations`
  - Key: UCAN CID (UTF-8 bytes)
  - Value: JSON bytes (`serde_json`)

## Aliases
- **Alias -> ObjectID**
  - Stored in sled tree: `meta/aliases`
  - Key: alias string (UTF-8 bytes)
  - Value: ObjectID raw bytes
