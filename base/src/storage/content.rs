use crate::error::Result;

/// Content address (BLAKE3 hash)
pub type Hash = [u8; 32];

/// Compute BLAKE3 hash of data
pub fn compute_hash(data: &[u8]) -> Hash {
    blake3::hash(data).into()
}

/// Encode hash as lowercase hexadecimal string
pub fn hash_to_hex(hash: &Hash) -> String {
    hex::encode(hash)
}

/// Decode hexadecimal string to hash
pub fn hex_to_hash(hex: &str) -> Result<Hash> {
    let bytes = hex::decode(hex)?;
    if bytes.len() != 32 {
        return Err(crate::error::LatticeError::Serialization(format!(
            "Invalid hash length: expected 32 bytes, got {}",
            bytes.len()
        )));
    }
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&bytes);
    Ok(hash)
}

/// Compute Merkle tree root from chunk hashes
/// Uses BLAKE3 to hash concatenated child hashes
pub fn compute_merkle_root(chunk_hashes: &[Hash]) -> Hash {
    if chunk_hashes.is_empty() {
        // Empty hash for no chunks
        return blake3::hash(b"").into();
    }

    if chunk_hashes.len() == 1 {
        // Single chunk is its own root
        return chunk_hashes[0];
    }

    // Build Merkle tree bottom-up
    let mut level = chunk_hashes.to_vec();

    while level.len() > 1 {
        let mut next_level = Vec::new();

        for pair in level.chunks(2) {
            let combined = if pair.len() == 2 {
                // Concatenate two hashes
                let mut combined = [0u8; 64];
                combined[..32].copy_from_slice(&pair[0]);
                combined[32..].copy_from_slice(&pair[1]);
                combined
            } else {
                // Odd number: just use the single hash
                let mut combined = [0u8; 64];
                combined[..32].copy_from_slice(&pair[0]);
                combined[32..].copy_from_slice(&pair[0]); // Duplicate for consistency
                combined
            };

            let parent_hash = blake3::hash(&combined).into();
            next_level.push(parent_hash);
        }

        level = next_level;
    }

    level[0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_roundtrip() {
        let data = b"test data";
        let hash = compute_hash(data);
        let hex = hash_to_hex(&hash);
        let decoded = hex_to_hash(&hex).unwrap();
        assert_eq!(hash, decoded);
    }

    #[test]
    fn test_hash_deterministic() {
        let data = b"Hello, NeuralFS!";
        let hash1 = compute_hash(data);
        let hash2 = compute_hash(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_merkle_root_empty() {
        let root = compute_merkle_root(&[]);
        let expected: Hash = blake3::hash(b"").into();
        assert_eq!(root, expected);
    }

    #[test]
    fn test_merkle_root_single() {
        let hash = compute_hash(b"test");
        let root = compute_merkle_root(&[hash]);
        assert_eq!(root, hash);
    }

    #[test]
    fn test_merkle_root_deterministic() {
        let hashes: Vec<Hash> = vec![
            compute_hash(b"chunk1"),
            compute_hash(b"chunk2"),
            compute_hash(b"chunk3"),
        ];

        let root1 = compute_merkle_root(&hashes);
        let root2 = compute_merkle_root(&hashes);
        assert_eq!(root1, root2);
    }
}
