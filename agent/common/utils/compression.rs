// agent/common/utils/compression.rs
// This file handles compression of telemetry data

use anyhow::{Context, Result};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::{Read, Write};

// Compression level to use (0-9)
pub const DEFAULT_COMPRESSION_LEVEL: Compression = Compression::default();

// Compress data using zlib
pub fn compress_data(data: &[u8], level: Compression) -> Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), level);
    encoder
        .write_all(data)
        .context("Failed to write data to encoder")?;
    encoder.finish().context("Failed to finish compression")
}

// Decompress zlib data
pub fn decompress_data(compressed_data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(compressed_data);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .context("Failed to decompress data")?;
    Ok(decompressed)
}

// Compress a string
pub fn compress_string(s: &str, level: Compression) -> Result<Vec<u8>> {
    compress_data(s.as_bytes(), level)
}

// Decompress to a string
pub fn decompress_to_string(compressed_data: &[u8]) -> Result<String> {
    let decompressed = decompress_data(compressed_data)?;
    String::from_utf8(decompressed).context("Decompressed data is not valid UTF-8")
}

// Calculate compression ratio
pub fn compression_ratio(original: &[u8], compressed: &[u8]) -> f64 {
    if original.len() == 0 {
        return 1.0;
    }
    compressed.len() as f64 / original.len() as f64
}

// Determine if compression is beneficial (typically if ratio < 0.9)
pub fn is_compression_beneficial(original: &[u8], compressed: &[u8]) -> bool {
    compression_ratio(original, compressed) < 0.9
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress() {
        let original =
            b"This is a test string that should compress well because it has repeated patterns.";

        let compressed = compress_data(original, Compression::default()).unwrap();
        let decompressed = decompress_data(&compressed).unwrap();

        assert_eq!(original.to_vec(), decompressed);

        // Test with different compression levels
        let compressed_max = compress_data(original, Compression::best()).unwrap();
        let decompressed_max = decompress_data(&compressed_max).unwrap();

        assert_eq!(original.to_vec(), decompressed_max);
        assert!(
            compressed_max.len() <= compressed.len(),
            "Best compression should result in smaller or equal size"
        );
    }

    #[test]
    fn test_string_compression() {
        let original = "This is a string test with Unicode characters: こんにちは, 你好, مرحبا";

        let compressed = compress_string(original, Compression::default()).unwrap();
        let decompressed = decompress_to_string(&compressed).unwrap();

        assert_eq!(original, decompressed);
    }

    #[test]
    fn test_compression_ratio() {
        let original =
            b"This is a test string that should compress well because it has repeated patterns.";
        let compressed = compress_data(original, Compression::default()).unwrap();

        let ratio = compression_ratio(original, &compressed);
        assert!(
            ratio < 1.0,
            "Compression ratio should be less than 1.0 for compressible data"
        );

        // Test with incompressible data (already compressed)
        let re_compressed = compress_data(&compressed, Compression::default()).unwrap();
        let re_ratio = compression_ratio(&compressed, &re_compressed);

        // Re-compressing should result in a ratio close to or greater than 1.0
        assert!(
            re_ratio >= 0.9,
            "Re-compressing already compressed data should not be beneficial"
        );
    }
}
