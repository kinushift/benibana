use crate::{
    BenibanaError,
    clean::{check_grapheme, is_over_limit_bytes, text_replace_retain, trim_in_place},
    normalize::normalize_in_place,
};

// --------------------------------core処理---------------------------------
pub(crate) fn sanitize(
    mut text: String,
    max_chars: usize,
    allow_new_line: bool,
    apply_nfkc: bool,
    katakana_to_hiragana: bool,
) -> Result<String, BenibanaError> {
    is_over_limit_bytes(&text, max_chars)?;

    // &mut Stringで渡すと元のStringを書き換えるためメモリ確保がしなくてよい
    text_replace_retain(&mut text, allow_new_line);
    // nfkc変換がtrueの場合にノーマライズされる
    if apply_nfkc {
        normalize_in_place(&mut text, katakana_to_hiragana);
    }
    trim_in_place(&mut text);

    if text.is_empty() {
        return Err(BenibanaError::Empty);
    }

    // ASCIIのみなら即時リターンで終了
    if text.is_ascii() {
        // 英数字は1文字1バイトのため.lenで文字数となる
        if text.len() <= max_chars {
            return Ok(text);
        } else {
            return Err(BenibanaError::TooLong);
        }
    }

    let res_text = check_grapheme(text, max_chars)?;

    Ok(res_text)
}
