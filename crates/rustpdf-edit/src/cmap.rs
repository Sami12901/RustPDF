//! rustpdf-edit/cmap — ToUnicode CMap table parser for PDF font decoding.
//! Maps embedded character codes and CIDs to standard Unicode (UTF-8) strings.

use std::collections::HashMap;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ToUnicodeCMap {
    pub char_map: HashMap<u32, String>,
    pub is_16bit: bool,
}

impl ToUnicodeCMap {
    /// Parses a CMap stream content into a lookup table.
    pub fn parse(content: &[u8]) -> Self {
        let mut map = HashMap::new();
        let mut is_16bit = false;
        let text = String::from_utf8_lossy(content);

        let lines: Vec<&str> = text.lines().collect();
        let mut idx = 0;

        while idx < lines.len() {
            let line = lines[idx].trim();

            if line.ends_with("beginbfchar") {
                idx += 1;
                while idx < lines.len() {
                    let cur = lines[idx].trim();
                    if cur.ends_with("endbfchar") {
                        break;
                    }

                    // Format: <src_hex> <dst_hex>
                    let tokens: Vec<&str> = cur.split_whitespace().collect();
                    if tokens.len() >= 2 {
                        let clean_src = tokens[0].trim_matches(|c| c == '<' || c == '>');
                        if clean_src.len() >= 4 {
                            is_16bit = true;
                        }
                        if let (Some(src), Some(dst)) =
                            (parse_hex_token(tokens[0]), parse_unicode_hex(tokens[1]))
                        {
                            map.insert(src, dst);
                        }
                    }
                    idx += 1;
                }
            } else if line.ends_with("beginbfrange") {
                idx += 1;
                while idx < lines.len() {
                    let cur = lines[idx].trim();
                    if cur.ends_with("endbfrange") {
                        break;
                    }

                    // Format 1: <start_hex> <end_hex> <dst_start_hex>
                    // Format 2: <start_hex> <end_hex> [<dst1> <dst2> ...]
                    let tokens: Vec<&str> = cur.split_whitespace().collect();
                    if tokens.len() >= 3 {
                        let clean_src = tokens[0].trim_matches(|c| c == '<' || c == '>');
                        if clean_src.len() >= 4 {
                            is_16bit = true;
                        }
                        if let (Some(start), Some(end)) =
                            (parse_hex_token(tokens[0]), parse_hex_token(tokens[1]))
                        {
                            if !tokens[2].starts_with('[') {
                                if let Some(dst_start) = parse_hex_token(tokens[2]) {
                                    for (offset, code) in (start..=end).enumerate() {
                                        if let Some(ch) = char::from_u32(dst_start + offset as u32)
                                        {
                                            map.insert(code, ch.to_string());
                                        }
                                    }
                                }
                            } else {
                                // Array format: extract items between [ ... ]
                                let mut arr_items = Vec::new();
                                for &t in &tokens[2..] {
                                    let cleaned = t.trim_matches(|c| c == '[' || c == ']');
                                    if !cleaned.is_empty() {
                                        if let Some(s) = parse_unicode_hex(cleaned) {
                                            arr_items.push(s);
                                        }
                                    }
                                }
                                for (offset, code) in (start..=end).enumerate() {
                                    if let Some(dst_str) = arr_items.get(offset) {
                                        map.insert(code, dst_str.clone());
                                    }
                                }
                            }
                        }
                    }
                    idx += 1;
                }
            }

            idx += 1;
        }

        Self {
            char_map: map,
            is_16bit,
        }
    }

    /// Decodes a sequence of PDF character bytes into a Unicode string using the CMap.
    pub fn decode_bytes(&self, bytes: &[u8]) -> String {
        if self.char_map.is_empty() {
            return String::from_utf8_lossy(bytes).to_string();
        }

        let mut out = String::new();
        let mut i = 0;

        while i < bytes.len() {
            if self.is_16bit && i + 1 < bytes.len() {
                let code16 = u16::from_be_bytes([bytes[i], bytes[i + 1]]) as u32;
                if let Some(s) = self.char_map.get(&code16) {
                    out.push_str(s);
                    i += 2;
                    continue;
                }
            }

            let code8 = bytes[i] as u32;
            if let Some(s) = self.char_map.get(&code8) {
                out.push_str(s);
            } else {
                out.push(bytes[i] as char);
            }
            i += 1;
        }

        out
    }
}

fn parse_hex_token(tok: &str) -> Option<u32> {
    let clean = tok.trim_matches(|c| c == '<' || c == '>');
    u32::from_str_radix(clean, 16).ok()
}

fn parse_unicode_hex(tok: &str) -> Option<String> {
    let clean = tok.trim_matches(|c| c == '<' || c == '>');
    if clean.len() % 4 == 0 {
        let mut s = String::new();
        for chunk in clean.as_bytes().chunks(4) {
            if let Ok(c_str) = std::str::from_utf8(chunk) {
                if let Ok(val) = u32::from_str_radix(c_str, 16) {
                    if let Some(ch) = char::from_u32(val) {
                        s.push(ch);
                    }
                }
            }
        }
        if !s.is_empty() {
            return Some(s);
        }
    }

    // Single codepoint fallback
    if let Ok(val) = u32::from_str_radix(clean, 16) {
        if let Some(ch) = char::from_u32(val) {
            return Some(ch.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmap_bfchar_parsing() {
        let cmap_src = b"
/CIDInit /ProcSet findresource begin
12 dict begin
begincmap
1 beginbfchar
<0001> <0995>
endbfchar
endcmap
end
";
        let cmap = ToUnicodeCMap::parse(cmap_src);
        assert_eq!(cmap.char_map.get(&1), Some(&"ক".to_string()));

        let decoded = cmap.decode_bytes(&[0x00, 0x01]);
        assert_eq!(decoded, "ক");
    }

    #[test]
    fn test_cmap_bfrange_parsing() {
        let cmap_src = b"
beginbfrange
<0001> <0003> <0041>
endbfrange
";
        let cmap = ToUnicodeCMap::parse(cmap_src);
        assert_eq!(cmap.char_map.get(&1), Some(&"A".to_string()));
        assert_eq!(cmap.char_map.get(&2), Some(&"B".to_string()));
        assert_eq!(cmap.char_map.get(&3), Some(&"C".to_string()));
    }
}
