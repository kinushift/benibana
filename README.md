# sumire
Rust製のUnicode対応入力検証ライブラリ

## 概要  
テキスト＆文字数制限(usize)を使用し、Unicodeの構造を考慮した入力検証を行う。

書記素クラスタ ・ Unicodeスカラー値を利用し、特殊な文字などを検証。

nfkc正規化と、カタカナを平仮名に変換することが可能。

## 関数
```rust
// 改行も削除する
pub fn sanitize_line(text: String, max_chars: usize) -> Result<String, SumireError>

pub fn sanitize_nfkc_line(text: String, max_chars: usize) -> Result<String, SumireError>

pub fn sanitize_nfkc_jp_line(text: String, max_chars: usize) -> Result<String, SumireError>

// 改行を残す
pub fn sanitize_text(text: String, max_chars: usize) -> Result<String, SumireError>

pub fn sanitize_nfkc_text(text: String, max_chars: usize) -> Result<String, SumireError>

pub fn sanitize_nfkc_jp_text(text: String, max_chars: usize) -> Result<String, SumireError>

```


| 関数 | 改行の扱い | nfkc変換 | ひらがなに変換 |
| --- | --- | --- | --- |
| `sanitize_line` | 削除する | しない | しない |
| `sanitize_nfkc_line` | 削除する | する | しない |
| `sanitize_nfkc_jp_line` | 削除する | する | する |
| `sanitize_text` | 残す。改行文字はすべて `\n` にそろえる | しない | しない |
| `sanitize_nfkc_text` | 残す。改行文字はすべて `\n` にそろえる | する | しない |
| `sanitize_nfkc_jp_text` | 残す。改行文字はすべて `\n` にそろえる | する | する |


## エラー
- `Empty` : テキストがありません
- `TooLong` : 上限を超えています
- `InvalidCharacter` : 使えない文字が含まれています

##  処理について
**処理されること** :
1. 制御文字・見えない文字 ・ 向きなどを変える文字を取り除く
2. (nfkc_のみ) nfkc正規化を行う
3. (jp_のみ) カタカナから0x60を引き、ひらがなにする
3. 前後の空白を取り除く
4. 書記素での上限確認
5. 異常な文字の検出
   - スカラー値13個以上でできた1文字(長すぎる絵文字の連結など)
   - 異体字セレクタの連続
   - ivsが漢字の後ろ以外に配置されている
   - イングランド ・ スコットランド ・ ウェールズの旗以外のタグ文字

**処理の限界** :
1. ZWJ, ZWNJ, および異体字セレクタについては正しい文章の表示のために残しています。そのため1文字につき異体字セレクタ1つはスルーされるようになっています。それにより、少量のデータを隠せてしまいます。\
3. `\r` だけの改行とタブは削除されます。
4. 文字の向きの制御を取り除くため、アラビア語・ヘブライ語に英数字や括弧が混ざる文章では、括弧や数字の位置が変わることがあります。（文字自体は崩れません）
5. モンゴル文字を使用するものには非推奨(入力が削除される場合があります)

## 依存
- memchr ・ unicode-segmentation ・ unicode-normalization　・ tinyvec(unicode-normalizationの依存) の4つ
- `Cargo.lock` をコミットして、依存のバージョンを固定し、`cargo audit` ・ `cargo vet` で確認しています。

## テスト
変更のたびに、蓄積したコーパスを引き継いで fuzzing を実行しています。

- 最新版での実行: 約3億回、クラッシュは0
- 検査項目: panicしないか、取り除いた文字が残らないか、前後に空白が残っていないか、2回実行しても同じ結果になるか(Displayを除く)
- テストについて: これらの結果は、試した範囲で問題が見つからなかったことを示すものです。すべての入力に対して正しく動くことや、安全であることを保証するものではありません。

## ライセンス
- 本リポジトリはロジックの透明性のために公開されています。
- このライブラリは現在開発中のため、ライセンスを付与しておらず、コード利用・改変・再配布は許可していません。

## コピーライト
Copyright © 2026 Kinushift. All rights reserved.
