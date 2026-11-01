use memchr::memmem::find;
use unicode_segmentation::UnicodeSegmentation;

use crate::{SCALARS_PER_CHARACTER, SumireError, TAG_FLAGS};

/// 重い処理の前にバイト数だけで明らかに大きすぎる入力を弾く。
/// 1文字は最大12パーツ×4バイト = 48バイトとし、上限×48を超えると異常である。（unicodeの絵文字の結合がMax10個&マージンのため12）
fn is_over_limit_bytes(raw_text: &str, max_chars: usize) -> Result<(), SumireError> {
    let raw_text_usize = raw_text.len();

    // オーバーしているなら
    if raw_text_usize > max_chars * 48 {
        return Err(SumireError::TooLong);
    }

    Ok(())
}

fn text_retain(text: &mut String, allow_new_line: bool) {
    // matches!を使用して制御文字の削除を行う
    // \nはsanitize_lineの場合に削除を行う
    // ZWJ(U+200D)は絵文字に、ZWNJ(U+200C)はペルシア語等に使用されているため残している。
    text.retain(|c| {
        allow_new_line && c == '\n'
            || (!c.is_control()
                && !matches!(c,
                    '\u{00AD}' | // SOFT HYPHEN: 行末で折り返すときだけハイフンとして出る
                    '\u{061C}' | // アラビア文字用の右から左へ(ALM)
                    '\u{115F}' | // 古いハングルの組み立て用パーツ
                    '\u{1160}' | // 同上
                    '\u{200B}' | // 幅のない空白
                    '\u{200E}' | // 左から右へ(LRM)
                    '\u{200F}' | // 右から左へ(RLM)
                    '\u{202A}'..= '\u{202E}' | // 埋め込み・上書き制御一覧
                    '\u{2060}'..= '\u{206F}' | // 0~4: 単語結合&見えない数学記号, 5: 今後追加される可能性あり, 6~9: 分離制御一覧, A~F: 古い書式制御
                    '\u{3164}' | // ハングル文字の透明文字（文字と分類されているためゲームの名前に悪用される）
                    '\u{FEFF}' | // 幅のない改行禁止の空白(BOM)
                    '\u{FFA0}' // 同上
                ))
    });
}

fn trim_in_place(text: &mut String) {
    let end = text.trim_end().len();
    text.truncate(end);

    let start = text.len() - text.trim_start().len();
    text.drain(..start);
}

// --------------------------------main---------------------------------
// core処理
pub(crate) fn sanitize(
    mut text: String,
    max_chars: usize,
    allow_new_line: bool,
) -> Result<String, SumireError> {
    is_over_limit_bytes(&text, max_chars)?;

    // &mut Stringで渡すと元のStringを書き換えるためメモリ確保がしなくてよい
    text_retain(&mut text, allow_new_line);
    // trim()は&strを戻すが、これにすると元のtextを直接書き換えるためメモリ確保の回数が減る
    trim_in_place(&mut text);

    //  サニタイズ後(悪意ない文字がない場合は空のためエラーを返す)
    if text.is_empty() {
        return Err(SumireError::Empty);
    }

    // 英数字であるなら即時リターンで終了
    if text.is_ascii() {
        // 英数字は1文字1バイトのため.lenで文字数となる
        if text.len() <= max_chars {
            return Ok(text);
        } else {
            return Err(SumireError::TooLong);
        }
    }
    // 以降は英数字のみではないものが含まれている（例: 日本語・絵文字・全角英数字・その他言語）

    // 初めにmemchrによってタグ文字があるかどうかを確認する。
    let has_tags = find(text.as_bytes(), b"\xF3\xA0").is_some();

    // 文字数を数える変数
    let mut count = 0;

    // graphemesは書記素という単位(ひとがみて1文字と同じカウントをする) = 人が1文字とするカウントごとにループが回る
    for g in text.graphemes(true) {
        // ループ1回につき1文字追加されていく
        count += 1;

        // 文字数を超えたらすぐにエラーにする
        if count > max_chars {
            return Err(SumireError::TooLong);
        }

        // 1文字gの構成されるスカラー値をカウントし、13個以上異常であれば除外をする(絵文字等)
        if g.chars().count() > SCALARS_PER_CHARACTER {
            return Err(SumireError::InvalidCharacter);
        }

        // タグ文字の初めのバイト列が確認されていて、なおかつタグ文字の範囲内のものが入っていて、3か国の国旗でないバイト列の場合はエラー
        if has_tags
            && g.contains(|c| matches!(c, '\u{E0000}'..='\u{E007F}'))
            && !TAG_FLAGS.contains(&g)
        {
            return Err(SumireError::InvalidCharacter);
        };

        // vs = Variation Selector(異体字セレクタ)
        let mut prev_vs = false;

        for c in g.chars() {
            let is_vs = matches!(c, '\u{FE00}'..= '\u{FE0F}' | '\u{E0100}'..= '\u{E01EF}');
            if is_vs && prev_vs {
                return Err(SumireError::InvalidCharacter);
            }
            prev_vs = is_vs;
        }
    }

    Ok(text)
}
