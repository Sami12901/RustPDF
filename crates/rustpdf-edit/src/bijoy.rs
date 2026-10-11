//! rustpdf-edit/bijoy — Bidirectional Bijoy ANSI <-> Bengali Unicode converter.
//! Converts legacy 8-bit encoded Bengali text (e.g. SutonnyMJ, SolaimanLipi ANSI) to standard Unicode
//! and vice-versa for lossless PDF in-place text extraction and editing.

/// Checks whether a text string or font name is likely using Bijoy/ANSI Bengali encoding.
pub fn is_likely_bijoy(text: &str, font_name: &str) -> bool {
    let fn_lower = font_name.to_lowercase();
    if fn_lower.contains("sutonny")
        || fn_lower.contains("bijoy")
        || fn_lower.contains("matra")
        || fn_lower.contains("solaiman")
        || fn_lower.contains("bangla")
        || fn_lower.contains("boishakhi")
        || fn_lower.contains("probhat")
    {
        return true;
    }

    if text.is_empty() {
        return false;
    }

    // If text already contains Bengali Unicode characters, it is already Unicode!
    if text.chars().any(|c| ('\u{0980}'..='\u{09FF}').contains(&c)) {
        return false;
    }

    // Check for distinctive high-byte Bijoy markers or multi-char signatures
    let has_distinctive_bijoy = text.contains("cÖ")
        || text.contains("Kvi")
        || text.contains("KZ")
        || text.contains("wK")
        || text.contains("‡K")
        || text.contains("¯’")
        || text.contains("¯ú")
        || text.contains("¯^")
        || text.contains("kÖ")
        || text.contains("MÖ")
        || text.contains("eÖ");

    if has_distinctive_bijoy {
        return true;
    }

    // Check for high-ASCII conjunct codepoints typical of Bijoy fonts (0x80..0xFF)
    // Note: Do not include common Latin-1 symbols like © or ® which are standard copyright/trademark
    let bijoy_char_count = text
        .chars()
        .filter(|&c| {
            matches!(
                c,
                'Ö' | 'ª'
                    | '¨'
                    | '¯'
                    | '²'
                    | '³'
                    | 'µ'
                    | '¼'
                    | '½'
                    | '¾'
                    | 'Â'
                    | 'Ã'
                    | 'Ä'
                    | 'Å'
                    | 'Æ'
                    | 'Ç'
                    | 'È'
                    | 'É'
                    | 'Ê'
                    | 'Ë'
                    | 'Ì'
                    | 'Í'
                    | 'Î'
                    | 'Ï'
                    | 'Ð'
                    | 'Ñ'
                    | 'Ò'
                    | 'Ó'
                    | 'Ô'
                    | 'Õ'
                    | '×'
                    | 'Ø'
                    | 'Ù'
                    | 'Ú'
                    | 'Û'
                    | 'Ü'
                    | 'Ý'
                    | 'Þ'
                    | 'ß'
                    | 'à'
                    | 'á'
                    | 'â'
                    | 'ã'
                    | 'ä'
                    | 'å'
                    | 'æ'
                    | 'ç'
                    | 'è'
                    | 'é'
                    | 'ê'
                    | 'ë'
                    | 'ì'
                    | 'í'
                    | 'î'
                    | 'ï'
                    | 'ð'
                    | 'ñ'
                    | 'ò'
                    | 'ó'
                    | 'ô'
                    | 'õ'
                    | 'ö'
                    | '÷'
                    | 'ø'
                    | 'ù'
                    | 'ú'
                    | 'û'
                    | 'ü'
                    | 'ý'
                    | 'þ'
                    | '\u{0087}'
                    | '\u{0089}'
                    | '\u{008A}'
                    | '\u{0084}'
                    | '\u{0086}'
                    | '‡'
                    | '‰'
                    | 'Š'
                    | '„'
                    | '†'
            )
        })
        .count();

    bijoy_char_count >= 2
}

/// Converts a Bijoy ANSI encoded string into standard UTF-8 Bengali Unicode.
pub fn bijoy_to_unicode(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }

    // Step 1: Pre-process composite multi-char clusters & conjuncts
    let mut s = input.to_string();

    // Multi-char Bijoy patterns
    let replacements = [
        ("Av", "আ"),
        ("cÖ", "প্র"),
        ("MÖ", "গ্র"),
        ("eÖ", "ব্র"),
        ("kÖ", "শ্র"),
        ("k^", "শ্ব"),
        ("k¥", "শ্ম"),
        ("¯’", "স্থ"),
        ("¯ú", "স্প"),
        ("¯^", "স্ব"),
        ("¯‹", "স্ক"),
        ("¯Í", "স্ত"),
        ("¯¿", "স্ত্র"),
        ("¯§", "স্ম"),
        ("¯œ", "স্ন"),
        ("¯ø", "স্ল"),
    ];
    for (pat, repl) in replacements {
        s = s.replace(pat, repl);
    }

    // Step 2: Character-by-character conversion with post-kar / pre-kar reordering
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(chars.len() * 2);
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        // Visual pre-vowels: 'w' (ি), '‡' (ে), '‰' (ৈ), '„' (র্)
        // In Bijoy, these appear BEFORE the consonant. In Unicode, consonant appears first.
        if (c == 'w'
            || c == '‡'
            || c == '‰'
            || c == '\u{0087}'
            || c == '\u{0089}'
            || c == '„'
            || c == '\u{0084}')
            && i + 1 < chars.len()
        {
            let next_c = chars[i + 1];
            // If next character is a consonant or conjunct
            let next_uni = map_bijoy_single_char(next_c);
            let kar = match c {
                'w' => "ি",
                '‡' | '\u{0087}' => "ে",
                '‰' | '\u{0089}' => "ৈ",
                '„' | '\u{0084}' => "র্",
                _ => "",
            };

            // For Reph, it precedes consonant in Unicode phonetic order (র + ্ + consonant)
            if c == '„' || c == '\u{0084}' {
                out.push_str("র্");
                out.push_str(&next_uni);
            } else {
                out.push_str(&next_uni);
                out.push_str(kar);
            }
            i += 2;
            continue;
        }

        // Direct single character mapping
        out.push_str(&map_bijoy_single_char(c));
        i += 1;
    }

    out
}

/// Helper mapping individual Bijoy ANSI char to its Bengali Unicode string representation.
fn map_bijoy_single_char(c: char) -> String {
    match c {
        // Digits
        '0' => "০".into(),
        '1' => "১".into(),
        '2' => "২".into(),
        '3' => "৩".into(),
        '4' => "৪".into(),
        '5' => "৫".into(),
        '6' => "৬".into(),
        '7' => "৭".into(),
        '8' => "৮".into(),
        '9' => "৯".into(),

        // Sworoborno
        'A' => "অ".into(),
        'B' => "ই".into(),
        'C' => "ঈ".into(),
        'D' => "উ".into(),
        'E' => "ঊ".into(),
        'F' => "ঋ".into(),
        'G' => "এ".into(),
        'H' => "ঐ".into(),
        'I' => "ও".into(),
        'J' => "ঔ".into(),

        // Byanjonborno
        'K' => "ক".into(),
        'L' => "খ".into(),
        'M' => "গ".into(),
        'N' => "ঘ".into(),
        'O' => "ঙ".into(),
        'P' => "চ".into(),
        'Q' => "ছ".into(),
        'R' => "জ".into(),
        'S' => "ঝ".into(),
        'T' => "ঞ".into(),
        'U' => "ট".into(),
        'V' => "ঠ".into(),
        'W' => "ড".into(),
        'X' => "ঢ".into(),
        'Y' => "ণ".into(),
        'Z' => "ত".into(),
        '_' => "থ".into(),
        'b' => "দ".into(),
        'a' => "ধ".into(),
        'c' => "প".into(),
        'd' => "ফ".into(),
        'e' => "ব".into(),
        'f' => "ভ".into(),
        'g' => "ম".into(),
        'h' => "য".into(),
        'i' => "র".into(),
        'j' => "ল".into(),
        'k' => "শ".into(),
        'l' => "ষ".into(),
        'm' => "স".into(),
        'n' => "হ".into(),
        'o' => "ড়".into(),
        'p' => "ঢ়".into(),
        'q' => "য়".into(),
        'r' => "ৎ".into(),
        's' => "ং".into(),
        't' => "ঃ".into(),
        'u' => "ঁ".into(),

        // Vowel signs (Kar)
        'v' => "া".into(),
        'w' => "ি".into(),
        'x' => "ী".into(),
        'y' => "ু".into(),
        '~' => "ূ".into(),
        'ö' => "ৃ".into(),
        '‡' | '\u{0087}' => "ে".into(),
        '‰' | '\u{0089}' => "ৈ".into(),
        'Š' | '\u{008A}' => "ৌ".into(),
        'Ö' | 'ª' => "্র".into(),
        '¨' => "্য".into(),
        '„' | '†' | '\u{0084}' | '\u{0086}' => "র্".into(),

        // Conjuncts / Yuktakshar
        '²' => "ক্ষ".into(),
        '³' => "ক্ত".into(),
        'µ' => "ক্র".into(),
        '¼' => "ঙ্ক".into(),
        '½' => "ঙ্গ".into(),
        '¾' => "জ্ঞ".into(),
        'Â' => "ঞ্চ".into(),
        'Ã' => "ঞ্ছ".into(),
        'Ä' => "ঞ্জ".into(),
        'Å' | 'Æ' => "ট্ট".into(),
        'Ç' => "ড্ড".into(),
        'È' => "ণ্ট".into(),
        'É' => "ণ্ঠ".into(),
        'Ê' => "ণ্ড".into(),
        'Ë' => "স্ত".into(),
        'Ì' => "ত্ব".into(),
        'Í' => "ত্ম".into(),
        'Î' => "ত্র".into(),
        'Ï' => "থ্ব".into(),
        'Ð' => "দ্দ".into(),
        'Ñ' => "দ্ধ".into(),
        'Ò' => "দ্ব".into(),
        'Ó' => "দ্ম".into(),
        'Ô' => "ধ্ব".into(),
        'Õ' => "ন্ট্র".into(),
        '×' => "ন্দ".into(),
        'Ø' => "ন্ধ".into(),
        'Ù' => "ন্ন".into(),
        'Ú' => "ন্ব".into(),
        'Û' => "ন্ম".into(),
        'Ü' => "প্ত".into(),
        'Ý' => "প্স".into(),
        'Þ' => "ব্দ".into(),
        'ß' => "ব্ধ".into(),
        'à' => "ব্ব".into(),
        'á' => "ব্ল".into(),
        'â' => "ভ্ল".into(),
        'ã' => "ম্ভ".into(),
        'ä' => "ম্ম".into(),
        'å' => "ম্ল".into(),
        'æ' => "ল্ক".into(),
        'ç' => "ল্গ".into(),
        'è' => "ল্ট".into(),
        'é' => "ল্ড".into(),
        'ê' => "ল্প".into(),
        'ë' => "ল্ফ".into(),
        'ì' => "ল্ব".into(),
        'í' => "ল্ম".into(),
        'î' => "ল্ল".into(),
        'ï' => "শ্চ".into(),
        'ð' => "শ্ছ".into(),
        'ñ' => "শ্ন".into(),
        'ò' => "ষ্ণ".into(),
        'ó' => "ষ্ট".into(),
        'ô' => "ষ্ঠ".into(),
        'õ' => "ষ্ফ".into(),
        '®' => "ষ্প".into(),
        '¯' => "স".into(),
        '÷' => "হ্ণ".into(),
        'ø' => "হ্ন".into(),
        'ù' => "হ্ম".into(),
        'ú' => "হ্ল".into(),
        'û' => "হ্ব".into(),
        'ü' => "হু".into(),
        'ý' => "হৃ".into(),
        'þ' => "হূ".into(),

        // Passthrough for spaces, punctuation, Latin letters
        _ => c.to_string(),
    }
}

/// Converts standard UTF-8 Bengali Unicode back into Bijoy ANSI string.
pub fn unicode_to_bijoy(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }

    let mut s = input.to_string();

    // Step 1: Pre-process composite Unicode conjuncts to Bijoy equivalents
    let unicode_replacements = [
        ("প্র", "cÖ"),
        ("গ্র", "MÖ"),
        ("ব্র", "eÖ"),
        ("শ্র", "kÖ"),
        ("শ্ব", "k^"),
        ("শ্ম", "k¥"),
        ("স্থ", "¯’"),
        ("স্প", "¯ú"),
        ("স্ব", "¯^"),
        ("স্ক", "¯‹"),
        ("স্ত", "¯Í"),
        ("স্ত্র", "¯¿"),
        ("স্ম", "¯§"),
        ("স্ন", "¯œ"),
        ("স্ল", "¯ø"),
        ("ক্ষ", "²"),
        ("ক্ত", "³"),
        ("ক্র", "µ"),
        ("ঙ্ক", "¼"),
        ("ঙ্গ", "½"),
        ("জ্ঞ", "¾"),
        ("ঞ্চ", "Â"),
        ("ঞ্ছ", "Ã"),
        ("ঞ্জ", "Ä"),
        ("ট্ট", "Å"),
        ("ড্ড", "Ç"),
        ("ণ্ট", "È"),
        ("ণ্ঠ", "É"),
        ("ণ্ড", "Ê"),
        ("ত্ব", "Ì"),
        ("ত্ম", "Í"),
        ("ত্র", "Î"),
        ("থ্ব", "Ï"),
        ("দ্দ", "Ð"),
        ("দ্ধ", "Ñ"),
        ("দ্ব", "Ò"),
        ("দ্ম", "Ó"),
        ("ধ্ব", "Ô"),
        ("ন্দ", "×"),
        ("ন্ধ", "Ø"),
        ("ন্ন", "Ù"),
        ("ন্ব", "Ú"),
        ("ন্ম", "Û"),
        ("প্ত", "Ü"),
        ("প্স", "Ý"),
        ("ব্দ", "Þ"),
        ("ব্ধ", "ß"),
        ("ব্ব", "à"),
        ("ব্ল", "á"),
        ("ভ্ল", "â"),
        ("ম্ভ", "ã"),
        ("ম্ম", "ä"),
        ("ম্ল", "å"),
        ("ল্ক", "æ"),
        ("ল্গ", "ç"),
        ("ল্ট", "è"),
        ("ল্ড", "é"),
        ("ল্প", "ê"),
        ("ল্ফ", "ë"),
        ("ল্ব", "ì"),
        ("ল্ম", "í"),
        ("ল্ল", "î"),
        ("শ্চ", "ï"),
        ("শ্ছ", "ð"),
        ("শ্ন", "ñ"),
        ("ষ্ণ", "ò"),
        ("ষ্ট", "ó"),
        ("ষ্ঠ", "ô"),
        ("ষ্ফ", "õ"),
        ("ষ্প", "®"),
        ("হ্ণ", "÷"),
        ("হ্ন", "ø"),
        ("হ্ম", "ù"),
        ("হ্ল", "ú"),
        ("হ্ব", "û"),
        ("হু", "ü"),
        ("হৃ", "ý"),
        ("হূ", "þ"),
        ("আ", "Av"),
        ("\u{9cd}\u{9af}", "¨"),
        ("\u{9cd}\u{9b0}", "Ö"),
    ];

    for (u, b) in unicode_replacements {
        s = s.replace(u, b);
    }

    // Step 2: Swap visual pre-kars ('ি', 'ে', 'ৈ') so they precede their consonant in Bijoy
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(chars.len() * 2);
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        // Check if next character is a pre-kar: 'ি', 'ে', 'ৈ'
        if i + 1 < chars.len() {
            let next_c = chars[i + 1];
            if next_c == 'ি' || next_c == 'ে' || next_c == 'ৈ' {
                let bijoy_kar = match next_c {
                    'ি' => "w",
                    'ে' => "‡",
                    'ৈ' => "‰",
                    _ => "",
                };
                let consonant_bijoy = map_unicode_single_char(c);
                out.push_str(bijoy_kar);
                out.push_str(&consonant_bijoy);
                i += 2;
                continue;
            }
        }

        // Direct conversion
        out.push_str(&map_unicode_single_char(c));
        i += 1;
    }

    out
}

fn map_unicode_single_char(c: char) -> String {
    match c {
        // Digits
        '০' => "0".into(),
        '১' => "1".into(),
        '২' => "2".into(),
        '৩' => "3".into(),
        '৪' => "4".into(),
        '৫' => "5".into(),
        '৬' => "6".into(),
        '৭' => "7".into(),
        '৮' => "8".into(),
        '৯' => "9".into(),

        // Sworoborno
        'অ' => "A".into(),
        'আ' => "Av".into(),
        'ই' => "B".into(),
        'ঈ' => "C".into(),
        'উ' => "D".into(),
        'ঊ' => "E".into(),
        'ঋ' => "F".into(),
        'এ' => "G".into(),
        'ঐ' => "H".into(),
        'ও' => "I".into(),
        'ঔ' => "J".into(),

        // Byanjonborno
        'ক' => "K".into(),
        'খ' => "L".into(),
        'গ' => "M".into(),
        'ঘ' => "N".into(),
        'ঙ' => "O".into(),
        'চ' => "P".into(),
        'ছ' => "Q".into(),
        'জ' => "R".into(),
        'ঝ' => "S".into(),
        'ঞ' => "T".into(),
        'ট' => "U".into(),
        'ঠ' => "V".into(),
        'ড' => "W".into(),
        'ঢ' => "X".into(),
        'ণ' => "Y".into(),
        'ত' => "Z".into(),
        'থ' => "_".into(),
        'দ' => "b".into(),
        'ধ' => "a".into(),
        'প' => "c".into(),
        'ফ' => "d".into(),
        'ব' => "e".into(),
        'ভ' => "f".into(),
        'ম' => "g".into(),
        'য' => "h".into(),
        'র' => "i".into(),
        'ল' => "j".into(),
        'শ' => "k".into(),
        'ষ' => "l".into(),
        'স' => "m".into(),
        'হ' => "n".into(),
        'ড়' => "o".into(),
        'ঢ়' => "p".into(),
        'য়' => "q".into(),
        'ৎ' => "r".into(),
        'ং' => "s".into(),
        'ঃ' => "t".into(),
        'ঁ' => "u".into(),

        // Kar
        'া' => "v".into(),
        'ি' => "w".into(),
        'ী' => "x".into(),
        'ু' => "y".into(),
        'ূ' => "~".into(),
        'ৃ' => "ö".into(),
        'ে' => "‡".into(),
        'ৈ' => "‰".into(),
        'ৌ' => "Š".into(),
        '্' => "&".into(),

        _ => c.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bijoy_to_unicode_screenshot_example() {
        // "KZ cÖKvi ?" -> "কত প্রকার ?"
        let bijoy = "KZ cÖKvi ?";
        let uni = bijoy_to_unicode(bijoy);
        assert_eq!(uni, "কত প্রকার ?");
    }

    #[test]
    fn test_unicode_to_bijoy_screenshot_example() {
        let uni = "কত প্রকার ?";
        let bijoy = unicode_to_bijoy(uni);
        assert_eq!(bijoy, "KZ cÖKvi ?");
    }

    #[test]
    fn test_vowel_reordering() {
        // 'wK' -> 'কি'
        assert_eq!(bijoy_to_unicode("wK"), "কি");
        // 'কি' -> 'wK'
        assert_eq!(unicode_to_bijoy("কি"), "wK");

        // '‡K' -> 'কে'
        assert_eq!(bijoy_to_unicode("‡K"), "কে");
        // 'কে' -> '‡K'
        assert_eq!(unicode_to_bijoy("কে"), "‡K");
    }

    #[test]
    fn test_digits_and_conjuncts() {
        assert_eq!(bijoy_to_unicode("১ ২ ৩"), "১ ২ ৩");
        assert_eq!(bijoy_to_unicode("cÖ"), "প্র");
        assert_eq!(bijoy_to_unicode("²"), "ক্ষ");
        assert_eq!(bijoy_to_unicode("¯’"), "স্থ");
    }

    #[test]
    fn test_is_likely_bijoy() {
        assert!(is_likely_bijoy("KZ cÖKvi ?", "SutonnyMJ"));
        assert!(is_likely_bijoy("cÖKvi", ""));
        assert!(!is_likely_bijoy("Hello World", "Helvetica"));
    }
}
