# Rust学習ガイド - rustfeedで学ぶRustの基礎

このディレクトリは、rustfeedプロジェクトの実装を教材として、Rustプログラミングの基礎を学ぶための資料集です。

## 対象読者

- C++やTypeScriptなど他の言語の経験がある方
- Rustの基本的な文法は知っているが、設計思想をより深く理解したい方
- 実際のプロジェクトでRustがどう使われるか知りたい方

## 学習トピック

| # | トピック | 比較言語 | 内容 |
|---|----------|----------|------|
| 1 | [所有権とボローイング](./01-ownership-borrowing.md) | C++ | Rustの最も重要な概念。メモリ管理の革命 |
| 2 | [エラー処理](./02-error-handling.md) | C++/TS | Result, Option型による安全なエラー処理 |
| 3 | [トレイトとジェネリクス](./03-traits-generics.md) | C++/TS | 抽象化とポリモーフィズム |
| 4 | [非同期プログラミング](./04-async-await.md) | TS | async/awaitとtokioランタイム |
| 5 | [モジュールとクレート](./05-modules-crates.md) | C++/TS | コードの構造化とパッケージ管理 |
| 6 | [パターンマッチング](./06-pattern-matching.md) | C++/TS | match式と網羅性チェック |
| 7 | [メモリ安全性](./07-memory-safety.md) | C++ | ライフタイムとコンパイル時検証 |

## rustfeedプロジェクト構造

本学習ガイドでは、以下のrustfeedのコードを例として参照します：

```
crates/
├── rustfeed-core/     # 共有ライブラリ
│   ├── models.rs      # データ構造（Feed, Article）
│   ├── db.rs          # SQLiteデータベース操作
│   ├── feed.rs        # RSS/Atomフィード取得
│   └── config.rs      # 設定管理
├── rustfeed-cli/      # CLIアプリケーション
│   ├── main.rs        # エントリポイント
│   └── commands.rs    # コマンド実装
└── rustfeed-tui/      # TUIアプリケーション
    ├── app.rs         # アプリケーション状態
    └── ui/            # UI描画
```

## 学習の進め方

1. 各トピックを順番に読む（1から順に依存関係があります）
2. コード例を実際にrustfeedのソースコードで確認する
3. 各セクションの理解度確認問題に挑戦する
4. 必要に応じて`cargo doc --open`でAPIドキュメントを参照する

## 凡例

本ガイドでは以下のアイコンを使用します：

- **Rust** - Rustのコード例
- **C++** - C++との比較
- **TypeScript** - TypeScript/Reactとの比較
- **rustfeed** - rustfeedプロジェクトからの実例

---

それでは、[01-ownership-borrowing.md](./01-ownership-borrowing.md)から始めましょう！
