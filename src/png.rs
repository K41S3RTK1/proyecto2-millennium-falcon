use std::{io, path::Path};
fn crc(bytes: &[u8]) -> u32 {
    let mut c = 0xffffffffu32;
    for &b in bytes {
        c ^= b as u32;
        for _ in 0..8 {
            c = (c >> 1) ^ if c & 1 != 0 { 0xedb88320 } else { 0 };
        }
    }
    !c
}
fn chunk(out: &mut Vec<u8>, tag: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let start = out.len();
    out.extend_from_slice(tag);
    out.extend_from_slice(data);
    out.extend_from_slice(&crc(&out[start..]).to_be_bytes());
}
/// PNG RGB con DEFLATE sin compresión: formato escrito aquí, sin crates de imagen.
pub fn encode(width: usize, height: usize, rgba: &[u8]) -> Vec<u8> {
    assert_eq!(rgba.len(), width * height * 4);
    let mut raw = Vec::with_capacity((width * 3 + 1) * height);
    for row in rgba.chunks_exact(width * 4) {
        raw.push(0);
        for p in row.chunks_exact(4) {
            raw.extend_from_slice(&p[..3]);
        }
    }
    let mut z = vec![0x78, 0x01];
    let count = raw.len().div_ceil(65535);
    for (i, part) in raw.chunks(65535).enumerate() {
        z.push(if i + 1 == count { 1 } else { 0 });
        let n = part.len() as u16;
        z.extend_from_slice(&n.to_le_bytes());
        z.extend_from_slice(&(!n).to_le_bytes());
        z.extend_from_slice(part);
    }
    let (mut a, mut b) = (1u32, 0u32);
    for byte in &raw {
        a = (a + *byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    z.extend_from_slice(&((b << 16) | a).to_be_bytes());
    let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut header = Vec::new();
    header.extend_from_slice(&(width as u32).to_be_bytes());
    header.extend_from_slice(&(height as u32).to_be_bytes());
    header.extend_from_slice(&[8, 2, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &header);
    chunk(&mut out, b"IDAT", &z);
    chunk(&mut out, b"IEND", &[]);
    out
}
pub fn save(path: impl AsRef<Path>, width: usize, height: usize, pixels: &[u8]) -> io::Result<()> {
    std::fs::write(path, encode(width, height, pixels))
}
