use unicode_normalization::UnicodeNormalization;

pub(crate) fn normalize(text: &mut String, katakana_to_hiragana: bool) {
    if text.is_ascii() {
        return;
    }

    let normalize_text: String = text
        .nfkc()
        .map(|c| match c {
            '\u{30A1}'..='\u{30F6}' | '\u{30FD}'..='\u{30FE}' if katakana_to_hiragana => {
                let c_value = c as u32 - 0x60;

                char::from_u32(c_value).unwrap_or(c)
            }
            _ => c,
        })
        .collect();

    *text = normalize_text
}
