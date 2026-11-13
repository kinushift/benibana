use crate::sanitize::sanitize;

mod char_rules;
mod clean;
mod normalize;
mod sanitize;

#[derive(Debug, PartialEq)]
pub enum BenibanaError {
    Empty,
    TooLong,
    InvalidCharacter,
}
impl std::fmt::Display for BenibanaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "text is empty"),
            Self::TooLong => write!(f, "text is too long"),
            Self::InvalidCharacter => write!(f, "text contains an invalid character"),
        }
    }
}
impl std::error::Error for BenibanaError {}

// 3つのタグ文字フラッグ
const TAG_FLAGS: [&str; 3] = [
    "\u{1F3F4}\u{E0067}\u{E0062}\u{E0065}\u{E006E}\u{E0067}\u{E007F}",
    "\u{1F3F4}\u{E0067}\u{E0062}\u{E0073}\u{E0063}\u{E0074}\u{E007F}",
    "\u{1F3F4}\u{E0067}\u{E0062}\u{E0077}\u{E006C}\u{E0073}\u{E007F}",
];

// スカラー値
const SCALARS_PER_CHARACTER: usize = 12;

// --------------------------------公開関数---------------------------------
// 改行も削除する
pub fn sanitize_line(text: String, max_chars: usize) -> Result<String, BenibanaError> {
    sanitize(text, max_chars, false, false, false)
}

pub fn sanitize_nfkc_line(text: String, max_chars: usize) -> Result<String, BenibanaError> {
    sanitize(text, max_chars, false, true, false)
}

pub fn sanitize_nfkc_jp_line(text: String, max_chars: usize) -> Result<String, BenibanaError> {
    sanitize(text, max_chars, false, true, true)
}

// 改行を残す
pub fn sanitize_text(text: String, max_chars: usize) -> Result<String, BenibanaError> {
    sanitize(text, max_chars, true, false, false)
}

pub fn sanitize_nfkc_text(text: String, max_chars: usize) -> Result<String, BenibanaError> {
    sanitize(text, max_chars, true, true, false)
}

pub fn sanitize_nfkc_jp_text(text: String, max_chars: usize) -> Result<String, BenibanaError> {
    sanitize(text, max_chars, true, true, true)
}

// --------------------------------テスト---------------------------------
#[cfg(test)]
mod tests {
    use crate::{
        sanitize_line, sanitize_nfkc_jp_line, sanitize_nfkc_jp_text, sanitize_nfkc_line,
        sanitize_nfkc_text, sanitize_text,
    };

    #[test]
    fn normal_test() {
        let true_text = "葛\u{E0100}";
        let flag_text = "\u{1F3F4}\u{E0067}\u{E0062}\u{E0065}\u{E006E}\u{E0067}\u{E007F}";
        let line_break = "a\u{2028}b";
        let normal_zwnj = "❤\u{FE0F}\u{200D}🔥";

        // 正規処理
        assert_eq!(
            sanitize_line(true_text.to_string(), 32),
            Ok(true_text.to_string())
        );

        // 旗正規処理
        assert_eq!(
            sanitize_line(flag_text.to_string(), 32),
            Ok(flag_text.to_string())
        );

        assert_eq!(
            sanitize_text(line_break.to_string(), 32),
            Ok("a\nb".to_string())
        );

        assert_eq!(
            sanitize_text(normal_zwnj.to_string(), 32),
            Ok(normal_zwnj.to_string())
        )
    }

    #[test]
    fn too_long() {
        let long_text = "a".repeat(33);
        let long_text2 = format!("あ{}い", " ".repeat(64));
        // エラー処理
        assert_eq!(
            sanitize_text(long_text.to_string(), 32),
            Err(crate::BenibanaError::TooLong)
        );

        assert_eq!(
            sanitize_text(long_text2, 32),
            Err(crate::BenibanaError::TooLong)
        );
    }

    #[test]
    fn empty() {
        let empty_text = "\u{3164}";

        assert_eq!(
            sanitize_text(empty_text.to_string(), 32),
            Err(crate::BenibanaError::Empty)
        );
    }

    #[test]
    fn invalid_tag() {
        let malicious_text = "\u{E0061}a\u{E0061}";
        // let malicious_text2 = "a\u{E0100}b";
        let malicious_text2 = "葛\u{E0100}\u{E0100}";
        let malicious_text3 = "\u{200C}a";

        // 悪意ある文字列(正常であればInvalidCharacterとなる)
        assert_eq!(
            sanitize_text(malicious_text.to_string(), 32),
            Err(crate::BenibanaError::InvalidCharacter)
        );

        assert_eq!(
            sanitize_text(malicious_text2.to_string(), 32),
            Err(crate::BenibanaError::InvalidCharacter)
        );

        assert_eq!(
            sanitize_text(malicious_text3.to_string(), 32),
            Err(crate::BenibanaError::InvalidCharacter)
        )
    }

    #[test]
    fn normalize_test() {
        let katakana_new_line = "スシ\n";
        let katakana = "スシ";
        let kumimozi = "㍿";
        let kiteimozi = "株式会社";

        assert_eq!(
            sanitize_nfkc_text(katakana_new_line.to_string(), 32),
            Ok(katakana.to_string())
        );

        assert_eq!(
            sanitize_nfkc_line(kumimozi.to_string(), 32),
            Ok(kiteimozi.to_string())
        )
    }

    #[test]
    fn normalize_jp_test() {
        let katakana_1 = "ｽｼ\n";
        let hiragana_1 = "すし";
        let katakana_2 = "テンプラ";
        let hiragana_2 = "てんぷら";

        assert_eq!(
            sanitize_nfkc_jp_text(katakana_1.to_string(), 32),
            Ok(hiragana_1.to_string())
        );

        assert_eq!(
            sanitize_nfkc_jp_line(katakana_2.to_string(), 32),
            Ok(hiragana_2.to_string())
        )
    }
}

// *1-1 Cf = Format charavters 書式文字, 表示や処理を制御する
// *1-2 Mn = Nospacing Mark 前の文字につく記号を表す(例: 日本語の濁点)
// *1-3 Lo = Other Letter 大文字小文字を分類しない文字の分類
// *1-4 Cn = Others, not assigned 未割り当て
// *1-5 So = Symbol other, 専門グループ出ない残りの記号をまとめた分類
// *2 書記素 = 見た目上1文字のもの。スカラー値がいくつかまとまったものである。(graphemes)
// *3 スカラー値 = サロゲートを含まない文字番号(chars)
//    サロゲート = Unicodeの、UTF-16で、文字でないもの。 RustのString, charはUTF-8サポートのためカウントすらされない。
// *4 タグ文字 = UnicodeのTagsブロック({E0000}'..='\u{E007F})に含まれる特殊文字(正式なものは「イングランド・スコットランド・ウェールズ」の3つのみ)
