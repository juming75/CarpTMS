//! / 压缩/解压缩模块(Deflate)
use anyhow::Result;
use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use flate2::Compression;
use std::io::{Read, Write};

/// 解压缩数据
pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = DeflateDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}

/// 压缩数据
pub fn compress(data: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    let compressed = encoder.finish()?;
    Ok(compressed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress() {
        let original = b"Hello, this is a test string for compression!";

        let compressed = compress(original).unwrap();
        println!(
            "Original size: {}, Compressed size: {}",
            original.len(),
            compressed.len()
        );

        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(original.as_slice(), decompressed.as_slice());
    }
}
