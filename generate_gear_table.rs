// Generate BLAKE3-based Gear hash table per LFS-001 spec
fn main() {
    println!("// Generated BLAKE3 Gear hash table per LFS-001 spec");
    println!("// BLAKE3(b\"LatticeFS-Gear-v1\" || byte)");
    println!("const GEAR_TABLE: [u64; 256] = [");
    
    for i in 0..256u8 {
        let mut data = b"LatticeFS-Gear-v1".to_vec();
        data.push(i);
        
        let hash = blake3::hash(&data);
        let bytes = hash.as_bytes();
        
        // Take first 8 bytes as little-endian u64
        let value = u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        
        if i % 4 == 0 {
            print!("    ");
        }
        print!("0x{:016x}", value);
        if i < 255 {
            print!(", ");
        }
        if i % 4 == 3 {
            println!();
        }
    }
    
    println!("\n];");
}
