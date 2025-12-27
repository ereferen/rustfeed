# アーキテクチャ方針

このファイルは、rustfeedプロジェクトのアーキテクチャ方針を定義します。

## Cargoワークスペース構成

```
rustfeed/
├── Cargo.toml              # ワークスペース定義
├── rustfeed-core/          # 共有ライブラリ
│   └── src/
│       ├── lib.rs
│       ├── models/         # データモデル (Feed, Article)
│       ├── db/             # データベース操作
│       ├── feed/           # フィード取得・パース
│       └── config/         # 設定管理
├── rustfeed-cli/           # CLIバイナリ
│   └── src/
│       ├── main.rs
│       └── commands/       # CLIコマンド実装
└── rustfeed-tui/           # TUIバイナリ
    └── src/
        ├── main.rs
        ├── app.rs          # アプリケーション状態
        ├── ui.rs           # UI描画
        └── event.rs        # イベント処理
```

## クレート間の依存関係

```
rustfeed-cli ──────┐
                   ├──► rustfeed-core
rustfeed-tui ──────┘
```

- `rustfeed-core`は他のクレートに依存しない
- `rustfeed-cli`と`rustfeed-tui`は`rustfeed-core`に依存
- CLI/TUI間で直接の依存関係は持たない

## 新機能追加時の配置

| 機能の種類 | 配置先 |
|-----------|--------|
| データモデル | `rustfeed-core/src/models/` |
| データベース操作 | `rustfeed-core/src/db/` |
| フィード処理 | `rustfeed-core/src/feed/` |
| 設定関連 | `rustfeed-core/src/config/` |
| CLIコマンド | `rustfeed-cli/src/commands/` |
| TUI画面/コンポーネント | `rustfeed-tui/src/` |

### 判断基準

- **両方のUIで使う** → `rustfeed-core`
- **CLIのみで使う** → `rustfeed-cli`
- **TUIのみで使う** → `rustfeed-tui`

## データベーススキーマ変更ルール

1. `rustfeed-core/src/db/mod.rs`の`init_db()`を更新
2. 既存データとの互換性を考慮
3. マイグレーションが必要な場合は別途検討
4. スキーマ変更後は`CLAUDE.md`のスキーマセクションも更新

## 依存関係追加のガイドライン

### 追加前の確認事項
- 必要最小限の依存に留める
- 既存の依存で代替できないか検討
- ライセンスの確認（MIT/Apache-2.0推奨）
- メンテナンス状況の確認

### features指定
必要な機能のみを有効化：

```toml
# 良い例
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }

# 悪い例（不要な機能も含む）
tokio = { version = "1", features = ["full"] }
```

### クレート別の方針
- **rustfeed-core**: 依存追加は慎重に。CLI/TUI両方に影響
- **rustfeed-cli**: CLI固有の依存のみ
- **rustfeed-tui**: TUI固有の依存（ratatui等）

## 主要な依存クレート

| クレート | 用途 |
|---------|------|
| `tokio` | 非同期ランタイム |
| `anyhow` | エラーハンドリング |
| `rusqlite` | SQLiteデータベース |
| `feed-rs` | RSS/Atomパース |
| `reqwest` | HTTP通信 |
| `chrono` | 日時処理 |
| `clap` | CLIパーサー |
| `ratatui` | TUIフレームワーク |
| `colored` | ターミナル出力の色付け |

## 非同期処理の方針

- I/O操作（ネットワーク、ファイル）は非同期
- データベース操作は同期（rusqliteの制約）
- `tokio::spawn`でバックグラウンド処理
- `async`関数は`_async`サフィックスを付けない
