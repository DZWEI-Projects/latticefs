use crate::error::{LatticeError, Result};
use crate::storage::content::{compute_hash, compute_merkle_root, hash_to_hex, Hash};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// FastCDC parameters per LFS-001 spec
pub const MIN_CHUNK_SIZE: usize = 8192; // 8 KiB
pub const AVG_CHUNK_SIZE: usize = 16384; // 16 KiB
pub const MAX_CHUNK_SIZE: usize = 65536; // 64 KiB
pub const MASK: u64 = (1 << 13) - 1; // 13 bits for avg 16KB

/// Chunk boundary information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkBoundary {
    pub offset: usize,
    pub length: usize,
}

/// Reference to a chunk within an object
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChunkRef {
    pub hash: Hash,
    pub offset: u64,
    pub length: u32,
}

/// Manifest describing chunk tree structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkManifest {
    pub version: u32,        // Protocol version (1)
    pub total_size: u64,     // Total object size
    pub chunk_size_avg: u32, // Average chunk size
    pub chunks: Vec<ChunkRef>,
    pub merkle_root: Hash,
}

/// Gear hash table for FastCDC (deterministic, per LFS-001)
/// Generated via: BLAKE3(b"LatticeFS-Gear-v1" || byte)
const GEAR_TABLE: [u64; 256] = [
    0xa120aeeaa4f77e94, 0x19cea902d55f222e, 0xbd0ee54454fb3417, 0xaf7485d934f09354,
    0x4aa734f8a840e431, 0x54a475ed786a997b, 0x9b7c7007d5dd675d, 0xf3f0223bc0da7eb9,
    0x7a307a4d47c5badb, 0x99aeafc1526cd770, 0x71b80fa1b525c5df, 0x4b3d139b79b514f5,
    0x718073325970f7f3, 0x6f8193645be60e63, 0xd9dc1815fab16af6, 0x4b2afb9854951c2c,
    0x098ac0a611cf3510, 0xbd419a870c8d8d1b, 0x3a7dcdd3d1a9f30e, 0xe80bdb95355b42a2,
    0x375cffb7342f1e19, 0x8de5baf0b7aeabea, 0x238240680b731654, 0x19c950b44f8c6841,
    0xe8270b9a06763ef0, 0xedabc70367191a8f, 0x79475ca80c1c2b97, 0xb1fecc4c3370dc8e,
    0x83eb257c2ee7de3f, 0x80dafa32d1c5cd2c, 0x308bcc99105fcde5, 0xd7e3b78956e3f6f7,
    0x6ad05c740bac274b, 0xf79ba156d0c4b756, 0x837ace3e1783849d, 0x229d9fdc01d38801,
    0x568672338e24f233, 0x1b08e5cdd7326af0, 0xe3ba1a4c474383dd, 0xa65256d1105d6299,
    0x3dc9bf3e083dcbce, 0xc1701f9e8c23c323, 0xd04a82a69cc3b3e9, 0x4b761fe9128d52bf,
    0xe82f794d2fd6bec6, 0xd6a31c37185dc22e, 0x336a41f290f16573, 0xc5dbc1d171d1d505,
    0x44db5b7bc0b8b3e5, 0x8beb0587e0e576a4, 0x3be3052daa5354f2, 0x72b8bd99fd4b1a9a,
    0xdf8375527141c59d, 0x0b6dbe4e2878c39f, 0x29846ad9f9a055f6, 0xfee5ad2f3b6397f8,
    0xe25c5b54b72388cc, 0xd5fd8d33e21a7fd2, 0x0cb101d6c4bbfb41, 0xc4fd219b3b5e5337,
    0x56a422a62c5f785d, 0x7c18f720af199209, 0xd5ba2c0117e4fb3b, 0x96bf5f7efec5ec26,
    0x6146d324b9343571, 0xc23cba32a3e3c314, 0x72c333994669785c, 0xd5027735118e5c30,
    0xa86b8a197d7774cf, 0xf735a26204eb094c, 0xf1131ba4759e3c99, 0xb49ce0f25e65976a,
    0x92b147c4d281c9e5, 0x7504e48340d19b48, 0x4cdd3ae536e01ed2, 0x27a9e51b4d087a88,
    0xc02f0928cc3e53fb, 0x9e6160b3aff3d645, 0xa214409f2669b90e, 0x26e59ed692cec5cf,
    0xe83f8fc00d986957, 0x1fee79dc0f2af1df, 0xde45568d4f746038, 0x7236a70b4e88778a,
    0xa5825c6094117355, 0x64cdf6d97a50ca8b, 0xee1c0460f267a599, 0x5473c4757d14ace1,
    0xd2e93e5e32106d20, 0xc592a71ea0e562a2, 0x64e0b6a83eacf55d, 0xce4aa49875ca0c37,
    0x9eff43e428331e04, 0x133828cc56934c82, 0x1b9b934e63b3d9db, 0xbbe5f71e4262c2ff,
    0x9a5679d41b667283, 0x49a26e825c4f9b04, 0x49b2efdacd217906, 0x3624bf17107cb060,
    0x595b5e8314384303, 0xf44d9ffba16d01b5, 0x2721f02887ba6d0d, 0x12870fad9719b765,
    0xadb5a38a1d522605, 0xba9c71da3ec5904b, 0xb12f366462af733a, 0xaac6e08b64ca60da,
    0x0d826e8d460ce720, 0x3f01e1c2c8c98e78, 0x69ffea610109680f, 0xdf1dbc049381861d,
    0x5c724067d568c0f2, 0x1ba47164a4bf6d88, 0xa8b9a0e8dca93bef, 0xeec088468dc64526,
    0x86a481a593e2d4a3, 0xd24eb992cb8d029b, 0x87941b4a96b61b82, 0xb4bf6dd04d91aa65,
    0x5841ed1b2d01b25c, 0xcc3986b5611be726, 0xe8d7bef92685cf9a, 0x28ec918efbe4ce5d,
    0xf06ec68ea2afb69c, 0x886d058ff9a754f1, 0xf38354454794d1af, 0x66ef8c6e13395b86,
    0x88a69b781537cd10, 0x0b83994d50652048, 0xe7d36765f259321b, 0x30b38e435060803c,
    0x137ddb0258e47b26, 0x9027ae44d029f2af, 0xe42d819d985f3975, 0x20f4a7e1e009ec25,
    0xaf99f2a0d887a782, 0x2de098f822035a99, 0x1cecdf9b9e11fec6, 0x5ce73bfe4afac4d7,
    0xb98ef305701d68dc, 0xa4dca7923dc4a3c0, 0xd4dd45d38bcc94ec, 0xcce7fce40d31d216,
    0x8315e64a06354f41, 0xa40af68a09ae98f5, 0xacd18b04b53ed524, 0xc5b072387476ef29,
    0x5e769ec932fa5a60, 0x6dcf53e66874f8ad, 0x914405af08793672, 0xbd66f7673faa1f10,
    0xe63f0cbd5219a35b, 0xc5c927659cd530f4, 0xc0d8d505a9df4f1c, 0xd9c27ecb568f9367,
    0x028288031742694d, 0x9d68d46f2af3db02, 0xedf8a63868d238d8, 0x29c2b357c9f041a5,
    0x6f63431f0538695f, 0x2bb19ae07d4de965, 0x191cb20eb36a8300, 0x86dfd19fb5512d90,
    0x980a48ce545db6b3, 0x2d8ac0ec57e3b37a, 0x009f5deceb5693da, 0x431957459bb918cc,
    0xf7186f6c6ff3940d, 0x94b0854470adb845, 0x3fc89b6febc91245, 0x32390f01c541e3f8,
    0x885c8ad9935c3260, 0x75b77344a49621ce, 0x84d80938ae8c156e, 0x6857b9ff31c10f05,
    0x29e64b2388784856, 0x3838fb663e13e644, 0x8425b9fcec6f0ec9, 0x448060557ea4112a,
    0x57598d6e9d756d5c, 0x8ee9d776924dda66, 0xcc0bdfc8732636cb, 0xc8723c80c39abdc5,
    0xacd8246bd63c71fb, 0x436885e1123274b0, 0xe2fb7b9436d150d3, 0xe2b863925be5ccf4,
    0x5f86853a568ef6f7, 0xf236404fe151e705, 0x1d9437680de00b6f, 0x2c294cfd80f15584,
    0x7f989cf2a4f25286, 0xaf35a35bbd0c2600, 0x56ca1cd9592afd18, 0x9fa8c3d5e2176363,
    0x246e71bf35165d3d, 0xd7554bbb39af224b, 0x55985619ebbf4746, 0xa55c853bc4d0466e,
    0x8e3a85b4c7871b47, 0x08a48ca29db41388, 0x231477731cd26540, 0x4260d6296a80bc9f,
    0x1da22151c4106a2c, 0x318bfeff3e0f83e8, 0xe3b61b67c48687aa, 0x20956090449c96db,
    0x31a486254ea39034, 0x598769a07047fafd, 0x86773d0b4ad45d14, 0x02e6ab855e3db571,
    0x1813c9c5e3c86146, 0x0a0191cafdd98442, 0xa128e97bdac45b5a, 0x339cdb9f14d4551f,
    0xeb4c9814c67e2d1d, 0xc8012775a119cb5d, 0xb4a086293b9792a9, 0x37e1f12ef46da377,
    0xa175db8d473d0da0, 0x4dea4a2cfa14c7e7, 0x298ba2caf0646b10, 0xe042339e3318addb,
    0x9d169ec5f4ca1180, 0x345b8bee750a70c7, 0x4fcf320257505716, 0x5c5ac1a375d8d37d,
    0x73bd2677126c79f2, 0x24e94149f80ceb5b, 0x01e535a8180ff7a9, 0x236bcdf45e2adf2a,
    0xa6001b88139e7e86, 0x24a9d1dfb236e164, 0x636d58287b743992, 0xfc05e14bb4a980f2,
    0x17b6ec67ec897c3d, 0x78c058d48fe466e4, 0x5abed414a07d4017, 0x870dc5e854798eed,
    0x7f1ed5ccd340ede3, 0x179e123f39fca908, 0xe2094b5948f38bd1, 0x66924c4179ed03d3,
    0x2e0fc4e38a9c487f, 0xd180cdc43273f186, 0xdc1749fe12261907, 0x22b28c53dbc8e771,
    0xa46f6e044e6b9fea, 0x919f6770f112cd02, 0xedf0ecd52b27cc99, 0x32f86082647ee451,
    0x194133e12e1290fc, 0x7521d37ce116ab19, 0xdb388d76ea765ed6, 0x6acd033a263c993f,
];

/// Chunk data using FastCDC algorithm
pub fn chunk_data(data: &[u8]) -> Vec<ChunkBoundary> {
    if data.is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut pos = 0;

    while pos < data.len() {
        let end = (pos + MAX_CHUNK_SIZE).min(data.len());
        let chunk_start = pos;

        // Always include at least MIN_CHUNK_SIZE if available
        pos += MIN_CHUNK_SIZE.min(data.len() - pos);

        if pos >= data.len() {
            // Last chunk smaller than MIN_CHUNK_SIZE
            chunks.push(ChunkBoundary {
                offset: chunk_start,
                length: data.len() - chunk_start,
            });
            break;
        }

        // Find cut point using Gear hash
        let mut hash: u64 = 0;

        for byte in &data[pos..end] {
            hash = (hash << 1).wrapping_add(GEAR_TABLE[*byte as usize]);
            pos += 1;

            // Check for cut point or end
            if (hash & MASK) == 0 || pos >= end {
                break;
            }
        }

        chunks.push(ChunkBoundary {
            offset: chunk_start,
            length: pos - chunk_start,
        });
    }

    chunks
}

/// Content-addressed chunk store
pub struct ChunkStore {
    root: PathBuf,
}

impl ChunkStore {
    /// Create a new chunk store at the given root path
    pub fn new(root: PathBuf) -> Self {
        ChunkStore { root }
    }

    /// Get the file path for a chunk by its hash
    pub fn chunk_path(&self, hash: &Hash) -> PathBuf {
        let hex = hash_to_hex(hash);
        // Layout: chunks/aa/bb/<full_hash>
        self.root
            .join("chunks")
            .join(&hex[0..2])
            .join(&hex[2..4])
            .join(&hex)
    }

    /// Check if a chunk exists in the store
    pub fn chunk_exists(&self, hash: &Hash) -> bool {
        self.chunk_path(hash).exists()
    }

    /// Write a chunk to the store (with deduplication)
    pub async fn write_chunk(&self, hash: &Hash, data: &[u8]) -> Result<()> {
        let path = self.chunk_path(hash);

        // 1. Check if chunk already exists (deduplication)
        if path.exists() {
            return Ok(());
        }

        // 2. Create parent directories
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // 3. Write to temporary file
        let temp_path = path.with_extension("tmp");
        tokio::fs::write(&temp_path, data).await?;

        // 4. Verify hash
        let computed = compute_hash(data);
        if computed != *hash {
            tokio::fs::remove_file(&temp_path).await?;
            return Err(LatticeError::HashMismatch);
        }

        // 5. Atomic rename
        tokio::fs::rename(&temp_path, &path).await?;

        // 6. Set read-only permissions
        let mut perms = tokio::fs::metadata(&path).await?.permissions();
        perms.set_readonly(true);
        tokio::fs::set_permissions(&path, perms).await?;

        Ok(())
    }

    /// Read a chunk from the store (with verification)
    pub async fn read_chunk(&self, hash: &Hash) -> Result<Vec<u8>> {
        let path = self.chunk_path(hash);

        if !path.exists() {
            return Err(LatticeError::ChunkNotFound {
                hash: hash_to_hex(hash),
            });
        }

        let data = tokio::fs::read(&path).await?;

        // Verify integrity on read
        let computed = compute_hash(&data);
        if computed != *hash {
            return Err(LatticeError::CorruptedChunk {
                expected: hash_to_hex(hash),
                computed: hash_to_hex(&computed),
            });
        }

        Ok(data)
    }

    /// Read a chunk from the store (blocking, with verification).
    pub fn read_chunk_sync(&self, hash: &Hash) -> Result<Vec<u8>> {
        let path = self.chunk_path(hash);

        if !path.exists() {
            return Err(LatticeError::ChunkNotFound {
                hash: hash_to_hex(hash),
            });
        }

        let data = std::fs::read(&path)?;

        let computed = compute_hash(&data);
        if computed != *hash {
            return Err(LatticeError::CorruptedChunk {
                expected: hash_to_hex(hash),
                computed: hash_to_hex(&computed),
            });
        }

        Ok(data)
    }

    /// Store an object by chunking and writing all chunks
    pub async fn store_object(&self, data: &[u8]) -> Result<ChunkManifest> {
        // 1. Chunk the data
        let boundaries = chunk_data(data);

        // 2. Store each chunk and collect references
        let mut chunk_refs = Vec::new();
        let mut chunk_hashes = Vec::new();

        for boundary in &boundaries {
            let chunk_data = &data[boundary.offset..boundary.offset + boundary.length];
            let hash = compute_hash(chunk_data);

            // Write chunk
            self.write_chunk(&hash, chunk_data).await?;

            chunk_refs.push(ChunkRef {
                hash,
                offset: boundary.offset as u64,
                length: boundary.length as u32,
            });

            chunk_hashes.push(hash);
        }

        // 3. Compute Merkle root
        let merkle_root = compute_merkle_root(&chunk_hashes);

        // 4. Build manifest
        let manifest = ChunkManifest {
            version: 1,
            total_size: data.len() as u64,
            chunk_size_avg: AVG_CHUNK_SIZE as u32,
            chunks: chunk_refs,
            merkle_root,
        };

        Ok(manifest)
    }

    /// Retrieve an object by reading and assembling all chunks
    pub async fn retrieve_object(&self, manifest: &ChunkManifest) -> Result<Vec<u8>> {
        let mut data = Vec::with_capacity(manifest.total_size as usize);

        for chunk_ref in &manifest.chunks {
            let chunk_data = self.read_chunk(&chunk_ref.hash).await?;

            // Verify chunk length
            if chunk_data.len() != chunk_ref.length as usize {
                return Err(LatticeError::LengthMismatch);
            }

            data.extend_from_slice(&chunk_data);
        }

        // Verify total size
        if data.len() != manifest.total_size as usize {
            return Err(LatticeError::LengthMismatch);
        }

        // Verify Merkle root
        let chunk_hashes: Vec<Hash> = manifest.chunks.iter().map(|c| c.hash).collect();
        let computed_root = compute_merkle_root(&chunk_hashes);
        if computed_root != manifest.merkle_root {
            return Err(LatticeError::MerkleRootMismatch);
        }

        Ok(data)
    }

    /// Retrieve an object by reading and assembling all chunks (blocking).
    pub fn retrieve_object_sync(&self, manifest: &ChunkManifest) -> Result<Vec<u8>> {
        let mut data = Vec::with_capacity(manifest.total_size as usize);

        for chunk_ref in &manifest.chunks {
            let chunk_data = self.read_chunk_sync(&chunk_ref.hash)?;

            if chunk_data.len() != chunk_ref.length as usize {
                return Err(LatticeError::LengthMismatch);
            }

            data.extend_from_slice(&chunk_data);
        }

        if data.len() != manifest.total_size as usize {
            return Err(LatticeError::LengthMismatch);
        }

        let chunk_hashes: Vec<Hash> = manifest.chunks.iter().map(|c| c.hash).collect();
        let computed_root = compute_merkle_root(&chunk_hashes);
        if computed_root != manifest.merkle_root {
            return Err(LatticeError::MerkleRootMismatch);
        }

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunking_deterministic() {
        let data = b"Hello, LatticeFS! ".repeat(1000);
        let chunks1 = chunk_data(&data);
        let chunks2 = chunk_data(&data);
        assert_eq!(chunks1, chunks2);
    }

    #[test]
    fn test_chunk_boundaries() {
        let data = vec![0u8; 100_000];
        let chunks = chunk_data(&data);

        // Should have multiple chunks
        assert!(chunks.len() > 1);

        // Verify all chunks are within bounds
        for (i, chunk) in chunks.iter().enumerate() {
            let is_last = i == chunks.len() - 1;

            if !is_last {
                assert!(chunk.length >= MIN_CHUNK_SIZE);
            }
            assert!(chunk.length <= MAX_CHUNK_SIZE);
        }

        // Verify chunks cover entire data
        let total: usize = chunks.iter().map(|c| c.length).sum();
        assert_eq!(total, data.len());
    }

    #[test]
    fn test_chunking_empty() {
        let data = b"";
        let chunks = chunk_data(data);
        assert_eq!(chunks.len(), 0);
    }

    #[test]
    fn test_chunking_small() {
        let data = b"small";
        let chunks = chunk_data(data);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].offset, 0);
        assert_eq!(chunks[0].length, 5);
    }

    #[tokio::test]
    async fn test_chunk_store_write_read() {
        let temp_dir = tempfile::tempdir().unwrap();
        let store = ChunkStore::new(temp_dir.path().to_path_buf());

        let data = b"test chunk data";
        let hash = compute_hash(data);

        // Write chunk
        store.write_chunk(&hash, data).await.unwrap();

        // Verify it exists
        assert!(store.chunk_exists(&hash));

        // Read chunk
        let read_data = store.read_chunk(&hash).await.unwrap();
        assert_eq!(read_data, data);
    }

    #[tokio::test]
    async fn test_chunk_store_deduplication() {
        let temp_dir = tempfile::tempdir().unwrap();
        let store = ChunkStore::new(temp_dir.path().to_path_buf());

        let data = b"duplicate data";
        let hash = compute_hash(data);

        // Write same chunk twice
        store.write_chunk(&hash, data).await.unwrap();
        store.write_chunk(&hash, data).await.unwrap();

        // Should only be stored once
        assert!(store.chunk_exists(&hash));
    }

    #[tokio::test]
    async fn test_store_retrieve_object() {
        let temp_dir = tempfile::tempdir().unwrap();
        let store = ChunkStore::new(temp_dir.path().to_path_buf());

        let data = b"Hello, LatticeFS! ".repeat(5000); // ~90KB

        // Store object
        let manifest = store.store_object(&data).await.unwrap();

        // Should have multiple chunks
        assert!(manifest.chunks.len() > 1);

        // Retrieve object
        let retrieved = store.retrieve_object(&manifest).await.unwrap();
        assert_eq!(retrieved, data);
    }
}
