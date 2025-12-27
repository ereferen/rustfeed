# 05. モジュールとクレート

Rustのモジュールシステムは、コードを論理的に整理し、可視性を制御する仕組みです。C++のヘッダーファイルやTypeScriptのES Modulesとは異なるアプローチを取っています。

## 目次

1. [モジュールの基本](#1-モジュールの基本)
2. [ファイル構造とモジュール](#2-ファイル構造とモジュール)
3. [可視性とpub](#3-可視性とpub)
4. [クレートとワークスペース](#4-クレートとワークスペース)
5. [rustfeedでの実例](#5-rustfeedでの実例)

---

## 1. モジュールの基本

### モジュールの宣言

```rust
// モジュールをインラインで定義
mod utils {
    pub fn helper() {
        println!("Helper function");
    }

    fn private_helper() {
        // これはモジュール外からアクセス不可
    }
}

// 使用
fn main() {
    utils::helper();
}
```

### C++との比較

**C++** の名前空間とヘッダー：

```cpp
// utils.h
#ifndef UTILS_H
#define UTILS_H

namespace utils {
    void helper();
    // private_helperはヘッダーに書かなければ隠蔽可能
}

#endif

// utils.cpp
#include "utils.h"

namespace utils {
    void helper() {
        std::cout << "Helper function" << std::endl;
    }

    // 無名名前空間で隠蔽
    namespace {
        void private_helper() { }
    }
}

// main.cpp
#include "utils.h"

int main() {
    utils::helper();
}
```

**違い**:

| 特徴 | Rust | C++ |
|------|------|-----|
| 宣言と実装 | 同じ場所 | ヘッダー/ソース分離 |
| インクルードガード | 不要 | 必要 |
| デフォルト可視性 | private | public |
| コンパイル単位 | クレート | 翻訳単位 |

### TypeScriptとの比較

**TypeScript** のモジュール：

```typescript
// utils.ts
export function helper() {
    console.log("Helper function");
}

function privateHelper() {
    // exportしなければモジュール外からアクセス不可
}

// main.ts
import { helper } from './utils';

helper();
```

**Rust**：

```rust
// utils.rs または mod utils { ... }
pub fn helper() {
    println!("Helper function");
}

fn private_helper() {
    // デフォルトで非公開
}

// main.rs
mod utils;

fn main() {
    utils::helper();
}
```

**違い**:
- TypeScript: `export` で公開
- Rust: `pub` で公開（デフォルトは非公開）

---

### 理解度確認 1

**問題**: 以下のRustコードで、`main` から呼び出せる関数はどれでしょうか？

```rust
mod outer {
    pub mod inner {
        pub fn public_fn() {}
        fn private_fn() {}
    }

    fn outer_private() {}
    pub fn outer_public() {}
}

fn main() {
    // ここから呼び出せる関数は？
}
```

<details>
<summary>回答を見る</summary>

**呼び出せる関数**:
- `outer::inner::public_fn()` - `outer` と `inner` と関数自体がすべて `pub`
- `outer::outer_public()` - `outer` と関数が `pub`

**呼び出せない関数**:
- `outer::inner::private_fn()` - 関数が `pub` でない
- `outer::outer_private()` - 関数が `pub` でない

Rustのモジュール可視性は「パス上のすべてが公開されている」必要があります。

```rust
fn main() {
    outer::inner::public_fn();  // OK
    outer::outer_public();      // OK
    // outer::inner::private_fn();  // エラー
    // outer::outer_private();      // エラー
}
```

</details>

---

## 2. ファイル構造とモジュール

### ファイルとモジュールの対応

```
src/
├── main.rs          // クレートルート（バイナリ）
├── lib.rs           // クレートルート（ライブラリ）
├── utils.rs         // mod utils;
└── db/
    ├── mod.rs       // mod db;
    ├── queries.rs   // mod queries;（db/mod.rs内で宣言）
    └── schema.rs    // mod schema;（db/mod.rs内で宣言）
```

### 2つのスタイル

**スタイル1**: `mod.rs` を使う（従来の方法）

```
src/
└── db/
    ├── mod.rs       // モジュール定義
    └── queries.rs
```

**スタイル2**: ファイル名と同名のディレクトリ（Rust 2018以降）

```
src/
├── db.rs            // モジュール定義
└── db/
    └── queries.rs
```

### モジュールの宣言と使用

```rust
// src/lib.rs または src/main.rs
mod db;       // src/db.rs または src/db/mod.rs を読み込む
mod utils;    // src/utils.rs を読み込む

pub use db::Database;  // 再エクスポート
```

### TypeScriptとの比較

**TypeScript** のインポート：

```typescript
// ディレクトリインポート（index.tsが必要）
import { Database } from './db';

// または明示的なパス
import { Database } from './db/database';
```

**Rust** のモジュール宣言：

```rust
// モジュールを「宣言」する（インポートとは異なる概念）
mod db;

// 使用
use db::Database;
```

**重要な違い**:
- TypeScript: ファイルが自動的にモジュール
- Rust: `mod` で明示的に宣言が必要

---

### 理解度確認 2

**問題**: 以下のファイル構造で、`main.rs` から `Query` 構造体を使うにはどう書けばよいでしょうか？

```
src/
├── main.rs
└── db/
    ├── mod.rs
    └── queries.rs  // pub struct Query { ... }
```

<details>
<summary>回答を見る</summary>

まず、各ファイルの内容：

```rust
// src/db/queries.rs
pub struct Query {
    pub sql: String,
}

// src/db/mod.rs
pub mod queries;  // queriesモジュールを公開

// src/main.rs
mod db;

use db::queries::Query;

fn main() {
    let q = Query { sql: "SELECT * FROM users".to_string() };
}
```

または、`db/mod.rs` で再エクスポートする場合：

```rust
// src/db/mod.rs
mod queries;
pub use queries::Query;  // 再エクスポート

// src/main.rs
mod db;

use db::Query;  // より短いパス

fn main() {
    let q = Query { sql: "SELECT * FROM users".to_string() };
}
```

</details>

---

## 3. 可視性とpub

### 可視性の種類

```rust
mod outer {
    // デフォルト（private）: 同じモジュール内のみ
    fn private_fn() {}

    // pub: どこからでもアクセス可能
    pub fn public_fn() {}

    // pub(crate): 同じクレート内のみ
    pub(crate) fn crate_fn() {}

    // pub(super): 親モジュールからアクセス可能
    pub(super) fn super_fn() {}

    // pub(in path): 指定したパスからアクセス可能
    pub(in crate::outer) fn path_fn() {}
}
```

### 構造体フィールドの可視性

```rust
pub struct User {
    pub name: String,      // 公開
    email: String,         // 非公開（同じモジュール内のみ）
    pub(crate) id: i64,    // クレート内のみ
}

impl User {
    // コンストラクタがないと外部から作成できない
    pub fn new(name: String, email: String) -> Self {
        Self { name, email, id: 0 }
    }
}
```

### C++との比較

**C++** のアクセス修飾子：

```cpp
class User {
public:
    std::string name;

private:
    std::string email;

protected:
    int id;

public:
    User(std::string name, std::string email)
        : name(std::move(name)), email(std::move(email)), id(0) {}
};
```

**違い**:
- C++は `class` 単位、Rustは `mod` 単位
- Rustには `protected` 相当がない（継承がないため）
- Rustはより細かい可視性制御が可能（`pub(crate)` など）

---

### 理解度確認 3

**問題**: 以下のRustコードで、`User::new()` を使わずに `User` インスタンスを作成できますか？

```rust
mod user {
    pub struct User {
        pub name: String,
        id: i64,
    }

    impl User {
        pub fn new(name: String) -> Self {
            Self { name, id: 1 }
        }
    }
}

fn main() {
    // User インスタンスを作成するには？
}
```

<details>
<summary>回答を見る</summary>

**`User::new()` を使う以外に作成する方法はありません。**

理由：
- `id` フィールドが非公開（`pub` がない）
- 構造体リテラルを使うには、すべてのフィールドにアクセスできる必要がある

```rust
fn main() {
    // これは動作する
    let user = user::User::new("Alice".to_string());

    // これはコンパイルエラー
    // let user = user::User {
    //     name: "Alice".to_string(),
    //     id: 1,  // エラー: field `id` is private
    // };
}
```

これはRustの重要なパターンです：
- 非公開フィールドを持つ構造体は「コンストラクタ」経由でのみ作成可能
- 不変条件（invariant）を強制できる
- 将来的なフィールド追加に対して後方互換性を保てる

</details>

---

## 4. クレートとワークスペース

### クレートとは

クレートはRustのコンパイル単位であり、パッケージです：

- **バイナリクレート**: 実行可能ファイルを生成
- **ライブラリクレート**: 他のクレートから使用される

```
my_project/
├── Cargo.toml       # パッケージ設定
└── src/
    ├── main.rs      # バイナリクレートのルート
    └── lib.rs       # ライブラリクレートのルート
```

### ワークスペース

複数のクレートをまとめて管理：

```toml
# Cargo.toml（ワークスペースルート）
[workspace]
members = [
    "crates/my-core",
    "crates/my-cli",
    "crates/my-tui",
]

[workspace.dependencies]
# 共通の依存関係を定義
tokio = { version = "1", features = ["full"] }
```

### TypeScriptとの比較

**TypeScript/Node.js** のモノレポ：

```json
// package.json（ルート）
{
  "workspaces": [
    "packages/*"
  ]
}
```

```
my_project/
├── package.json
└── packages/
    ├── core/
    │   ├── package.json
    │   └── src/
    ├── cli/
    │   ├── package.json
    │   └── src/
    └── web/
        ├── package.json
        └── src/
```

**Rust** のワークスペース：

```
my_project/
├── Cargo.toml          # ワークスペース定義
└── crates/
    ├── my-core/
    │   ├── Cargo.toml
    │   └── src/
    ├── my-cli/
    │   ├── Cargo.toml
    │   └── src/
    └── my-tui/
        ├── Cargo.toml
        └── src/
```

**類似点**:
- 複数パッケージを1つのリポジトリで管理
- 依存関係の共有

**違い**:
- Rustは `Cargo.lock` がワークスペースで1つ
- Rustの方がビルドの一貫性が高い

---

### 理解度確認 4

**問題**: 以下のワークスペース構成で、`my-cli` から `my-core` を使うには `Cargo.toml` にどう書けばよいでしょうか？

```
my_project/
├── Cargo.toml
└── crates/
    ├── my-core/
    │   ├── Cargo.toml
    │   └── src/lib.rs
    └── my-cli/
        ├── Cargo.toml
        └── src/main.rs
```

<details>
<summary>回答を見る</summary>

**ワークスペースルートの Cargo.toml**:

```toml
[workspace]
members = [
    "crates/my-core",
    "crates/my-cli",
]

[workspace.dependencies]
my-core = { path = "crates/my-core" }
```

**crates/my-cli/Cargo.toml**:

```toml
[package]
name = "my-cli"
version = "0.1.0"
edition = "2021"

[dependencies]
my-core.workspace = true  # ワークスペースから継承
```

または、直接パスを指定：

```toml
[dependencies]
my-core = { path = "../my-core" }
```

**crates/my-cli/src/main.rs**:

```rust
use my_core::SomeType;

fn main() {
    // my-coreの機能を使用
}
```

</details>

---

## 5. rustfeedでの実例

### ワークスペース構造

[Cargo.toml](../../Cargo.toml):

```toml
[workspace]
resolver = "2"
members = [
    "crates/rustfeed-core",
    "crates/rustfeed-cli",
    "crates/rustfeed-tui",
]

[workspace.package]
version = "0.5.0"
edition = "2021"

[workspace.dependencies]
# 共有依存関係
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
# ...

# 内部クレート
rustfeed-core = { path = "crates/rustfeed-core" }
```

**ポイント**:
- 3つのクレート: core（ライブラリ）、cli（バイナリ）、tui（バイナリ）
- `workspace.dependencies` で共通の依存関係を定義
- バージョンはワークスペースで統一

### rustfeed-coreのモジュール構造

[crates/rustfeed-core/src/lib.rs](../../crates/rustfeed-core/src/lib.rs):

```rust
//! # rustfeed-core
//!
//! rustfeed アプリケーションの共有ライブラリです。

pub mod config;
pub mod db;
pub mod feed;
pub mod models;

// 便利な再エクスポート
pub use config::AppConfig;
pub use db::Database;
pub use models::{Article, Feed};
```

**ポイント**:
- 4つのサブモジュールを公開
- よく使う型を再エクスポートして使いやすく
- ドキュメントコメントでクレートの説明

### 再エクスポートの効果

**再エクスポートなし**:
```rust
use rustfeed_core::db::Database;
use rustfeed_core::models::Feed;
use rustfeed_core::models::Article;
use rustfeed_core::config::AppConfig;
```

**再エクスポートあり**:
```rust
use rustfeed_core::{Database, Feed, Article, AppConfig};
```

### モジュール間の依存

```
rustfeed-core
├── models.rs     # 基本的なデータ型
├── db.rs         # models に依存
├── feed.rs       # models に依存
└── config.rs     # 独立

rustfeed-cli
└── commands.rs   # rustfeed-core の全モジュールに依存

rustfeed-tui
├── app.rs        # rustfeed-core に依存
└── ui/mod.rs     # app.rs に依存
```

### CLIからcoreを使用する例

[crates/rustfeed-cli/src/commands.rs](../../crates/rustfeed-cli/src/commands.rs):

```rust
use anyhow::{Context, Result};
use rustfeed_core::{feed, Database};  // 再エクスポートを使用

pub async fn add_feed(db: &Database, url: &str, name: Option<&str>) -> Result<()> {
    let (mut feed_info, _articles) = feed::fetch_feed(url)
        .await
        .with_context(|| format!("Failed to fetch feed from {}", url))?;

    // ...
}
```

**ポイント**:
- `rustfeed_core::Database` は再エクスポートされているので短いパスでアクセス
- `rustfeed_core::feed::fetch_feed` はモジュールパスを使用
- 各クレートは必要な依存のみをインポート

---

### 理解度確認 5

**問題**: rustfeedの設計を見て、以下の質問に答えてください。

1. なぜ `rustfeed-core` をライブラリクレートとして分離しているのでしょうか？
2. `pub use` で再エクスポートするメリットは何でしょうか？

<details>
<summary>回答を見る</summary>

**1. ライブラリ分離の理由**:

- **コードの再利用**: CLIとTUIの両方から同じロジックを使用
- **関心の分離**: ビジネスロジック（core）とUI（cli/tui）を分離
- **テスタビリティ**: coreは独立してテスト可能
- **将来の拡張**: Web API、Discord Bot など他のフロントエンドを追加しやすい
- **コンパイル最適化**: 変更がないクレートは再コンパイル不要

**2. 再エクスポートのメリット**:

- **API の簡略化**: ユーザーは内部構造を知らなくてよい
- **安定したインターフェース**: 内部構造を変更しても外部APIは維持
- **ドキュメントの見やすさ**: よく使う型がトップレベルに表示
- **インポート文の簡潔化**: 深いパスを書く必要がない

例えば、将来 `models.rs` を `models/` ディレクトリに分割しても：

```rust
// 内部変更前
pub use models::{Article, Feed};

// 内部変更後（models/article.rs, models/feed.rs に分割）
pub use models::article::Article;
pub use models::feed::Feed;

// 外部からの使用は変わらない
use rustfeed_core::{Article, Feed};
```

</details>

---

## まとめ

| 概念 | Rust | C++ | TypeScript |
|------|------|-----|------------|
| モジュール単位 | `mod` | namespace + ファイル | ファイル |
| 可視性デフォルト | private | public | export必要 |
| インポート | `use` | `#include` + `using` | `import` |
| パッケージ | クレート | ライブラリ（手動） | npm package |
| モノレポ | ワークスペース | CMake等 | npm/yarn workspaces |

次は [06-pattern-matching.md](./06-pattern-matching.md) でパターンマッチングについて学びます。
