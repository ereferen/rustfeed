# Rustコードスタイル規約

このファイルは、rustfeedプロジェクトのRustコードスタイルを定義します。

## 命名規則

| 対象 | 規則 | 例 |
|------|------|-----|
| 関数・変数 | snake_case | `fetch_feed`, `article_count` |
| 型・トレイト | PascalCase | `Feed`, `Article`, `Database` |
| 定数 | SCREAMING_SNAKE_CASE | `MAX_RETRIES`, `DEFAULT_TIMEOUT` |
| モジュール | snake_case | `models`, `commands` |
| ライフタイム | 短い小文字 | `'a`, `'db` |

## ドキュメンテーション

### rustdocコメント
- 公開関数には必ずrustdocを記述
- 日本語で教育的な説明を含める
- 所有権・借用の概念を説明に含める

```rust
/// フィードからすべての記事を取得する
///
/// # 引数
/// * `db` - データベース接続への参照（借用）
/// * `feed_id` - 取得対象のフィードID
///
/// # 戻り値
/// 記事のベクターを返す。所有権は呼び出し元に移動する。
///
/// # エラー
/// データベースアクセスに失敗した場合にエラーを返す
pub fn get_articles(db: &Database, feed_id: i64) -> Result<Vec<Article>> {
    // ...
}
```

### インラインコメント
- 複雑なロジックには説明を追加
- Rust特有の概念（所有権、ライフタイム等）は積極的に解説

## エラーハンドリング

- `anyhow::Result`を使用
- `.context()`でエラーに文脈を追加
- `unwrap()`は避け、`?`演算子を使用
- パニックは本当に回復不能な場合のみ

```rust
// 良い例
let feed = fetch_feed(&url)
    .await
    .context("フィードの取得に失敗しました")?;

// 悪い例
let feed = fetch_feed(&url).await.unwrap();
```

## インポート順序

1. 標準ライブラリ (`std::`)
2. 外部クレート
3. プロジェクト内モジュール (`crate::`, `super::`)

各グループは空行で区切る。

```rust
use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};

use crate::db::Database;
use crate::models::Feed;
```

## フォーマット

- `cargo fmt`に従う
- インデント: 4スペース
- 行の最大長: 100文字（推奨）
- 末尾カンマ: 複数行の場合は必ず付ける

## Option/Resultの扱い

- `if let`や`match`を活用
- `map`, `and_then`, `unwrap_or_default`を適切に使用
- 早期リターンでネストを浅く保つ

```rust
// 良い例：早期リターン
fn process_article(article: Option<Article>) -> Result<()> {
    let article = match article {
        Some(a) => a,
        None => return Ok(()), // 早期リターン
    };
    // articleを処理
    Ok(())
}
```
