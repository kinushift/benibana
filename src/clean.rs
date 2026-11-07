use memchr::memmem::find;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    SCALARS_PER_CHARACTER, SumireError, TAG_FLAGS,
    char_rules::{is_line_break, is_removable, is_unified_ideograph},
};

// 重い処理の前にバイト数だけで明らかに大きすぎる入力を弾く。
// 1文字は最大12パーツ×4バイト = 48バイトとし、上限×48を超えると異常である。(unicodeの絵文字の結合がMax10個だが、マージンのため12)
pub(crate) fn is_over_limit_bytes(raw_text: &str, max_chars: usize) -> Result<(), SumireError> {
    let raw_text_usize = raw_text.len();

    // オーバーしているなら
    if raw_text_usize > max_chars.saturating_mul(SCALARS_PER_CHARACTER * 4) {
        return Err(SumireError::TooLong);
    }

    Ok(())
}

pub(crate) fn text_replace_retain(text: &mut String, allow_new_line: bool) {
    // もし改行を残す処理で、なおかつ以下の改行にかかわるスカラー値があるのであればすべて\nに置き換える
    if allow_new_line && text.contains(is_line_break) {
        *text = text.replace(is_line_break, "\n");
    };

    // \nはsanitize_lineの場合にのみ検出される
    text.retain(|c| allow_new_line && c == '\n' || (!c.is_control() && !is_removable(c)));
}

// 新たにメモリ確保をせずにtrimを行う
pub(crate) fn trim_in_place(text: &mut String) {
    let end = text.trim_end().len();
    text.truncate(end);

    let start = text.len() - text.trim_start().len();
    text.drain(..start);
}

// 1文字ずつ処理を開始していく
pub(crate) fn check_grapheme(text: String, limit: usize) -> Result<String, SumireError> {
    // 初めにmemchrによってタグ文字(*3)があるかどうかを確認する。
    let has_tags = find(text.as_bytes(), b"\xF3\xA0").is_some();

    // 文字数を数える変数
    let mut count = 0;

    // graphemesは書記素(*2)
    for g in text.graphemes(true) {
        // このカウントで文字数を数える
        count += 1;
        // 文字数を超えたらすぐにエラーにする
        if count > limit {
            return Err(SumireError::TooLong);
        }

        // 1文字gの構成されるスカラー値(*3)をカウントし、13個以上異常であれば除外をする(絵文字等)
        if g.chars().count() > SCALARS_PER_CHARACTER {
            return Err(SumireError::InvalidCharacter);
        }

        // タグ文字(*4)の初めの2バイトが確認されていて、なおかつタグ文字(*3)の範囲内のものが入っていて、正式な3つの旗でないバイト列の場合はエラー
        if has_tags
            && g.contains(|c| matches!(c, '\u{E0000}'..='\u{E007F}'))
            && !TAG_FLAGS.contains(&g)
        {
            return Err(SumireError::InvalidCharacter);
        };

        let mut prev_scalar: char = '\0';
        // vs = Variation Selector(異体字セレクタ)、2つ連続で並んでいる場合エラーとなる。また、初期値をtrueにすることで初めの文字がvsの場合も弾くことができる
        let mut prev_vs = true;

        // スカラー値(*3)ごとのループ
        for c in g.chars() {
            // 漢字の後ろでしか使用できない異体字セレクタであるか
            let is_ivs = matches!(c, '\u{E0100}'..='\u{E01EF}');
            // is_ivsを含むいずれかの異体字セレクタであるか
            let is_vs = is_ivs || matches!(c, '\u{FE00}'..='\u{FE0F}');

            if is_vs && prev_vs {
                return Err(SumireError::InvalidCharacter);
            }

            if is_ivs && !is_unified_ideograph(prev_scalar) {
                return Err(SumireError::InvalidCharacter);
            }

            prev_scalar = c;
            prev_vs = is_vs;
        }
    }

    Ok(text)
}
