use unicode_normalization::UnicodeNormalization;

use crate::{BenibanaError, SCALARS_PER_CHARACTER};

pub(crate) fn normalize(
    text: &mut String,
    katakana_to_hiragana: bool,
    max_chars: usize,
) -> Result<(), BenibanaError> {
    if text.is_ascii() {
        return Ok(());
    }

    let limit = max_chars.saturating_mul(SCALARS_PER_CHARACTER);

    let normalize_text: String = text
        .nfkc()
        // スカラー値を1個づつ数えるリミットのため上限値の+1した値で止める
        .take(limit.saturating_add(1))
        // カタカナから0x60を引くとちょうどひらがなになる
        .map(|c| match c {
            '\u{30A1}'..='\u{30F6}' | '\u{30FD}'..='\u{30FE}' if katakana_to_hiragana => {
                let c_value = c as u32 - 0x60;

                char::from_u32(c_value).unwrap_or(c)
            }
            _ => c,
        })
        .collect();

    if normalize_text.chars().count() > limit {
        return Err(BenibanaError::TooLong);
    }

    *text = normalize_text;

    Ok(())
}
