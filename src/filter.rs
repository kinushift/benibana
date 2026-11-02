use memchr::memmem::find;
use unicode_segmentation::UnicodeSegmentation;

use crate::{SCALARS_PER_CHARACTER, SumireError, TAG_FLAGS};

// 重い処理の前にバイト数だけで明らかに大きすぎる入力を弾く。
// 1文字は最大12パーツ×4バイト = 48バイトとし、上限×48を超えると異常である。(unicodeの絵文字の結合がMax10個だが、マージンのため12)
fn is_over_limit_bytes(raw_text: &str, max_chars: usize) -> Result<(), SumireError> {
    let raw_text_usize = raw_text.len();

    // オーバーしているなら
    if raw_text_usize > max_chars.saturating_mul(SCALARS_PER_CHARACTER * 4) {
        return Err(SumireError::TooLong);
    }

    Ok(())
}

fn text_replace_retain(text: &mut String, allow_new_line: bool) {
    // もし改行を残す処理で、なおかつ以下の改行にかかわるスカラー値があるのであればすべて\nに置き換える
    if allow_new_line
        && text.contains(|c| matches!(c, '\u{000B}' | '\u{000C}' | '\u{2028}' | '\u{2029}'))
    {
        *text = text.replace(
            |c| matches!(c, '\u{000B}' | '\u{000C}' | '\u{2028}' | '\u{2029}'),
            "\n",
        );
    };

    // matches!を使用して制御文字の削除を行う
    // \nはsanitize_lineの場合にのみ検出される
    // ZWJ(U+200D)は絵文字のために、ZWNJ(U+200C)はペルシア語等に使用されているために残している。
    text.retain(|c| {
        allow_new_line && c == '\n'
            || (!c.is_control()
                && !matches!(c,
                    // 改行の文字(sanitize_text()では上記の処理で \n に置き換え済み)
                    '\u{2028}' |// 行区切り
                    '\u{2029}' | // 段落区切り

                    // 文字の向きを変える文字(表示の順番をごまかす攻撃に使われる)
                    '\u{061C}' | // アラビア文字用の右から左へ(ALM)
                    '\u{200E}' | // 左から右へ(LRM)
                    '\u{200F}' | // 右から左へ(RLM)
                    '\u{202A}'..= '\u{202E}' | // 埋め込み・上書き制御(LRE・RLE・PDF・LRO・RLO)

                    // 見えない空白・書式
                    '\u{00AD}' | // ソフトハイフン(行末で折り返すときだけハイフンとして出る)
                    '\u{200B}' | // 幅のない空白
                    '\u{2060}'..= '\u{206F}' | // 0: 単語結合, 1~4: 見えない数学記号, 5: 予約済み, 6~9: 向きの分離制御, A~F: 古い書式制御
                    '\u{FEFF}' | // 幅のない改行禁止の空白(BOM)

                    // 空白に見える文字(文字に分類されるためゲームの名前で悪用されたりする)
                    '\u{3164}' | // ハングル文字の透明文字
                    '\u{FFA0}' | // ハングルの半角の透明文字

                    // 特殊な書式制御(普段では使用しない)
                    '\u{17B4}'..= '\u{17B5}' | // クメール文字の見えない母音(使用は非推奨)
                    '\u{FFF9}' ..= '\u{FFFB}' | // ルビに関するもの(プレーンテキストでの使用を強く非推奨されている)
                    '\u{1BCA0}'..= '\u{1BCA3}' | // 速記用の書式制御(文字を重ねたり・上下にずらしたり)
                    '\u{1D173}'..= '\u{1D17A}' | // 楽譜用の書式制御

                     // 予約済み(まだ文字がない。見えない扱いの枠)
                    '\u{FFF0}' ..= '\u{FFF8}' |
                    '\u{E0080}' ..= '\u{E00FF}' |
                    '\u{E01F0}' ..= '\u{E0FFF}'

                ))
    });
}

fn trim_in_place(text: &mut String) {
    let end = text.trim_end().len();
    text.truncate(end);

    let start = text.len() - text.trim_start().len();
    text.drain(..start);
}

// --------------------------------core処理---------------------------------
pub(crate) fn sanitize(
    mut text: String,
    max_chars: usize,
    allow_new_line: bool,
) -> Result<String, SumireError> {
    is_over_limit_bytes(&text, max_chars)?;

    // &mut Stringで渡すと元のStringを書き換えるためメモリ確保がしなくてよい
    text_replace_retain(&mut text, allow_new_line);
    // trim()は&strを戻すが、これにすると元のtextを直接書き換えるためメモリ確保の回数が減る
    trim_in_place(&mut text);

    //  悪意ある文字のみの場合を含め空の場合はここで終了
    if text.is_empty() {
        return Err(SumireError::Empty);
    }

    // ASCIIのみなら即時リターンで終了
    if text.is_ascii() {
        // 英数字は1文字1バイトのため.lenで文字数となる
        if text.len() <= max_chars {
            return Ok(text);
        } else {
            return Err(SumireError::TooLong);
        }
    }
    // 以降は英数字のみではないものが含まれている(例: 日本語・絵文字・全角英数字・その他言語)

    // 初めにmemchrによってタグ文字(*3)があるかどうかを確認する。
    let has_tags = find(text.as_bytes(), b"\xF3\xA0").is_some();

    // 文字数を数える変数
    let mut count = 0;

    // graphemesは書記素(*1)
    for g in text.graphemes(true) {
        // このカウントで文字数を数える
        count += 1;

        // 文字数を超えたらすぐにエラーにする
        if count > max_chars {
            return Err(SumireError::TooLong);
        }

        // 1文字gの構成されるスカラー値(*3)をカウントし、13個以上異常であれば除外をする(絵文字等)
        if g.chars().count() > SCALARS_PER_CHARACTER {
            return Err(SumireError::InvalidCharacter);
        }

        // タグ文字(*3)の初めの2バイトが確認されていて、なおかつタグ文字(*3)の範囲内のものが入っていて、正式な3つの旗でないバイト列の場合はエラー
        if has_tags
            && g.contains(|c| matches!(c, '\u{E0000}'..='\u{E007F}'))
            && !TAG_FLAGS.contains(&g)
        {
            return Err(SumireError::InvalidCharacter);
        };

        // vs = Variation Selector(異体字セレクタ)、2つ連続で並んでいる場合エラーとなる。また、初期値をtrueにすることで初めの文字がvsの場合も弾くことができる
        let mut prev_vs = true;

        for c in g.chars() {
            // スカラー値が異体字セレクタであるかを確認
            let is_vs = matches!(c, '\u{FE00}'..= '\u{FE0F}' | '\u{E0100}'..= '\u{E01EF}');
            if is_vs && prev_vs {
                return Err(SumireError::InvalidCharacter);
            }
            prev_vs = is_vs;
        }
    }

    Ok(text)
}
