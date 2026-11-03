use crate::filter::sanitize;

mod filter;

#[derive(Debug, PartialEq)]
pub enum SumireError {
    Empty,
    TooLong,
    InvalidCharacter,
}
impl std::fmt::Display for SumireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "text is empty"),
            Self::TooLong => write!(f, "text is too long"),
            Self::InvalidCharacter => write!(f, "text contains an invalid character"),
        }
    }
}
impl std::error::Error for SumireError {}

// 3つのタグ文字フラッグ
const TAG_FLAGS: [&str; 3] = [
    "\u{1F3F4}\u{E0067}\u{E0062}\u{E0065}\u{E006E}\u{E0067}\u{E007F}",
    "\u{1F3F4}\u{E0067}\u{E0062}\u{E0073}\u{E0063}\u{E0074}\u{E007F}",
    "\u{1F3F4}\u{E0067}\u{E0062}\u{E0077}\u{E006C}\u{E0073}\u{E007F}",
];

// スカラー値
const SCALARS_PER_CHARACTER: usize = 12;

// 改行も削除する
pub fn sanitize_line(text: String, max_chars: usize) -> Result<String, SumireError> {
    sanitize(text, max_chars, false)
}

// 改行を残す
pub fn sanitize_text(text: String, max_chars: usize) -> Result<String, SumireError> {
    sanitize(text, max_chars, true)
}

// --------------------------------テスト---------------------------------
#[cfg(test)]
mod tests {
    use crate::{sanitize_line, sanitize_text};

    #[test]
    fn normal_test() {
        let true_text = "葛\u{E0100}";
        let flag_text = "\u{1F3F4}\u{E0067}\u{E0062}\u{E0065}\u{E006E}\u{E0067}\u{E007F}";
        let malicious_text2 = "a\u{2028}b";

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
            sanitize_text(malicious_text2.to_string(), 32),
            Ok("a\nb".to_string())
        )
    }

    #[test]
    fn too_long() {
        let long_text = "a".repeat(33);
        // エラー処理
        assert_eq!(
            sanitize_text(long_text.to_string(), 32),
            Err(crate::SumireError::TooLong)
        );
    }

    #[test]
    fn empty() {
        let empty_text = "\u{3164}";

        assert_eq!(
            sanitize_text(empty_text.to_string(), 32),
            Err(crate::SumireError::Empty)
        );
    }

    #[test]
    fn invalid_tag() {
        let malicious_text = "\u{E0061}a\u{E0061}";
        // let malicious_text2 = "a\u{E0100}b";
        let malicious_text2 = "葛\u{E0100}\u{E0100}";

        // 悪意ある文字列(正常であればInvalidCharacterとなる)
        assert_eq!(
            sanitize_text(malicious_text.to_string(), 32),
            Err(crate::SumireError::InvalidCharacter)
        );

        assert_eq!(
            sanitize_text(malicious_text2.to_string(), 32),
            Err(crate::SumireError::InvalidCharacter)
        )
    }
}

// *1 書記素 = 見た目上1文字のもの。スカラー値がいくつかまとまったものである。(graphemes)
// *2 スカラー値 = サロゲートを含まない文字番号(chars)
//    サロゲート = Unicodeの、UTF-16で、文字でないもの。 RustのString, charはUTF-8サポートのためカウントすらされない。
//    バイト = スカラー値をUTF-8で表したもの 1つのスカラー値につき1~4バイトである。(.len)
// *3 タグ文字 = UnicodeのTagsブロック({E0000}'..='\u{E007F})に含まれる特殊文字(正式なものは「イングランド・スコットランド・ウェールズ」の3つのみ)
