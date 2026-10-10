//! rustpdf-filters — Stream filters for standard PDF compression and encoding formats.

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum FilterError {
    #[error("invalid ASCIIHex character: {0}")]
    InvalidHexChar(char),
    #[error("unexpected end of stream in {0}")]
    UnexpectedEof(&'static str),
    #[error("corrupted run-length encoded stream")]
    InvalidRunLength,
    #[error("decompression error: {0}")]
    DecompressError(String),
}

/// Decodes ASCIIHex-encoded data. Ignores whitespace. Terminates on '>' or end of data.
pub fn ascii_hex_decode(data: &[u8]) -> Result<Vec<u8>, FilterError> {
    let mut out = Vec::new();
    let mut high_nibble: Option<u8> = None;

    for &b in data {
        if b.is_ascii_whitespace() {
            continue;
        }
        if b == b'>' {
            break;
        }

        let val = match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'f' => b - b'a' + 10,
            b'A'..=b'F' => b - b'A' + 10,
            _ => return Err(FilterError::InvalidHexChar(b as char)),
        };

        if let Some(h) = high_nibble.take() {
            out.push((h << 4) | val);
        } else {
            high_nibble = Some(val);
        }
    }

    if let Some(h) = high_nibble {
        // Odd number of digits: final digit assumed to have 0 low nibble.
        out.push(h << 4);
    }

    Ok(out)
}

/// Encodes binary data into ASCIIHex format terminated by '>'.
pub fn ascii_hex_encode(data: &[u8]) -> Vec<u8> {
    const HEX_CHARS: &[u8; 16] = b"0123456789ABCDEF";
    let mut out = Vec::with_capacity(data.len() * 2 + 1);
    for &b in data {
        out.push(HEX_CHARS[(b >> 4) as usize]);
        out.push(HEX_CHARS[(b & 0x0F) as usize]);
    }
    out.push(b'>');
    out
}

/// RunLengthDecode: standard PDF byte-oriented run length decompression (ISO 32000-1 §7.4.5).
pub fn run_length_decode(data: &[u8]) -> Result<Vec<u8>, FilterError> {
    let mut out = Vec::new();
    let mut i = 0;

    while i < data.len() {
        let length_byte = data[i];
        i += 1;

        if length_byte == 128 {
            // 128 indicates End of Data (EOD)
            break;
        } else if length_byte < 128 {
            // (1 to 128) means copy next (length_byte + 1) literal bytes
            let count = (length_byte as usize) + 1;
            if i + count > data.len() {
                return Err(FilterError::UnexpectedEof("RunLength literal run"));
            }
            out.extend_from_slice(&data[i..i + count]);
            i += count;
        } else {
            // (129 to 255) means copy next single byte (257 - length_byte) times
            let count = 257 - (length_byte as usize);
            if i >= data.len() {
                return Err(FilterError::UnexpectedEof("RunLength repeat byte"));
            }
            let byte = data[i];
            i += 1;
            out.resize(out.len() + count, byte);
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_hex_roundtrip() {
        let raw = b"Hello, PDF World!";
        let enc = ascii_hex_encode(raw);
        let dec = ascii_hex_decode(&enc).unwrap();
        assert_eq!(dec, raw);
    }

    #[test]
    fn test_ascii_hex_odd_digit() {
        let odd_hex = b"3>";
        let dec = ascii_hex_decode(odd_hex).unwrap();
        assert_eq!(dec, vec![0x30]);
    }

    #[test]
    fn test_run_length_decode() {
        // 2 literals "AB", 3 repeats of 'C' (257 - 255 + 1 = 3 -> byte 255-3+1 = 255 is 2 count: 257-255=2, 257-254=3)
        // length_byte: 1 means (1+1)=2 bytes: b"AB"
        // length_byte: 255 means (257-255)=2 bytes: b"C"
        // length_byte: 128 (EOD)
        let stream = vec![1, b'A', b'B', 255, b'C', 128];
        let dec = run_length_decode(&stream).unwrap();
        assert_eq!(dec, b"ABCC");
    }
}
