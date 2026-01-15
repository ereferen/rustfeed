# Claude Code Skills for Rustfeed

このディレクトリには rustfeed プロジェクト専用の Claude Code Skills が含まれています。

## Skills とは？

**Skills** は Claude に特定のタスクを実行する方法を教えるマークダウンファイルです。
Claude が自動的に判断して使用するため、手動で呼び出す必要はありません。

## 利用可能な Skills

### 1. rustfeed-architecture

**目的**: プロジェクトのアーキテクチャと構造を説明

**起動タイミング**:
- アーキテクチャに関する質問時
- 新機能追加時の配置場所確認
- データベーススキーマの参照

**提供情報**:
- Cargo ワークスペース構成
- クレート間の依存関係
- データベーススキーマ
- 新機能追加時の配置ルール
- 依存関係管理のベストプラクティス

**例**:
```
Q: "新しいコマンドを追加したいのですが、どこに配置すべきですか？"
→ rustfeed-architecture skill が起動し、配置ルールを説明
```

---

### 2. rustfeed-quality

**目的**: コード品質チェックの実行手順を提供

**起動タイミング**:
- コード品質確認が必要な時
- コミット前のチェック
- CI/CD 検証の実行

**提供コマンド**:
- `cargo fmt --check` - フォーマットチェック
- `cargo clippy` - Lint チェック
- `cargo test` - テスト実行
- `cargo doc` - ドキュメント生成

**例**:
```
Q: "コミット前に品質チェックを実行してください"
→ rustfeed-quality skill が起動し、全チェックを実行
```

---

### 3. rustfeed-feed-ops

**目的**: フィード管理と記事操作のコマンド実行をサポート

**起動タイミング**:
- フィード追加・削除・更新
- 記事取得（フェッチ）
- 記事の既読管理・お気に入り管理
- データベース操作

**提供コマンド**:
- `cargo run --bin rustfeed-cli -- add <URL>` - フィード追加
- `cargo run --bin rustfeed-cli -- fetch` - 記事取得
- `cargo run --bin rustfeed-cli -- articles --unread` - 未読記事表示
- TUI キーバインド情報

**例**:
```
Q: "新しいフィードを追加して記事を取得してください"
→ rustfeed-feed-ops skill が起動し、コマンドを実行
```

---

## Skills と MCP の違い

| 項目 | Skills | MCP |
|------|--------|-----|
| **目的** | Claude に「やり方」を教える | Claude に「ツール」を提供する |
| **実行方法** | 自動発見と実行 | Claude がツールを呼び出し |
| **形式** | マークダウンファイル | JSON + サーバー |
| **例** | "品質チェックの方法" | "GitHub API 接続" |

### 使い分け

- **Skills を使う**: プロジェクト固有の知識、ワークフロー、ベストプラクティス
- **MCP を使う**: 外部システムへのアクセス（API、データベース等）

---

## Skills の使い方

### 自動起動（推奨）

Skills は自動的に起動されるため、通常は何もする必要はありません。

例えば、以下のような質問をするだけで適切な Skill が起動します：

```
- "rustfeed のアーキテクチャについて教えてください"
  → rustfeed-architecture が起動

- "コード品質チェックを実行してください"
  → rustfeed-quality が起動

- "新しいフィードを追加したい"
  → rustfeed-feed-ops が起動
```

### 手動確認

Skills が正しく配置されているか確認：

```bash
ls -l .claude/skills/*/SKILL.md
```

### デバッグモード

Skills の起動状況を確認したい場合：

```bash
claude --debug
```

---

## Skills の追加・カスタマイズ

### 新しい Skill の追加

1. `.claude/skills/<skill-name>/` ディレクトリを作成
2. `SKILL.md` ファイルを作成
3. メタデータ（name, description, allowed-tools 等）を記述
4. Skill の内容を記述

### Skill のカスタマイズ

既存の Skill を編集する場合は、該当する `SKILL.md` ファイルを直接編集してください。

**例**: rustfeed-quality に新しいチェックコマンドを追加
```bash
# .claude/skills/rustfeed-quality/SKILL.md を編集
vim .claude/skills/rustfeed-quality/SKILL.md
```

---

## トラブルシューティング

### Skill が起動しない

**原因**: description が曖昧

**対処**: `SKILL.md` の `description` フィールドを具体的なキーワードを含めて記述

```yaml
# 悪い例
description: プロジェクト情報

# 良い例
description: rustfeed プロジェクトのアーキテクチャ、データベーススキーマ、クレート構成について説明
```

### Skill がロードされない

**確認事項**:
1. ファイルパスが正しいか: `.claude/skills/<name>/SKILL.md`
2. YAML シンタックスが正しいか（先頭に `---`、タブではなくスペース）
3. メタデータの必須フィールド（`name`, `description`）が存在するか

---

## 参考リソース

- [Claude Code Skills 公式ドキュメント](https://code.claude.com/docs/en/skills.md)
- [Claude Code MCP ドキュメント](https://code.claude.com/docs/en/mcp.md)
- [rustfeed プロジェクト規約](.claude/rules/)

---

## 貢献

新しい Skill のアイデアがある場合は、Issue を作成するか PR を送ってください。

### Skill 追加のガイドライン

- **明確な目的**: 各 Skill は単一の明確な目的を持つこと
- **適切な説明**: description にはキーワードを含め、起動タイミングを明確に
- **実用的な内容**: 実際のタスクで使える具体的な情報を提供
- **ドキュメント更新**: 新しい Skill を追加したらこのファイルも更新
