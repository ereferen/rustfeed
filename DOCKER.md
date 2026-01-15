# Docker環境セットアップガイド

このガイドでは、Docker環境でrustfeedを開発し、Claude Codeを使用する方法を説明します。

## 前提条件

- Docker
- Docker Compose
- Anthropic API Key

## クイックスタート（自動セットアップ）

最も簡単な方法は、テストスクリプトを使用することです。

```bash
# 設定ファイルの検証
python3 scripts/validate-docker.py

# Docker環境のビルドとテスト（自動）
./scripts/test-docker-env.sh
```

テストスクリプトが以下を自動で実行します：
- 前提条件のチェック
- .envファイルの作成（存在しない場合）
- Dockerイメージのビルド
- コンテナの起動
- 環境の動作確認

詳細は [scripts/README.md](./scripts/README.md) を参照してください。

## 手動セットアップ

### 1. API Keyの設定

`.env`ファイルを作成し、Anthropic API Keyを設定します。

```bash
cp .env.example .env
```

`.env`ファイルを編集して、API Keyを設定してください。

```env
ANTHROPIC_API_KEY=your_actual_api_key_here
```

**重要**: `.env`ファイルは`.gitignore`に含まれているため、コミットされません。

### 2. Dockerイメージのビルド

```bash
docker-compose build
```

初回ビルドは数分かかります。Rust環境とClaude Code CLIがインストールされます。

### 3. コンテナの起動

```bash
docker-compose up -d
```

コンテナがバックグラウンドで起動します。

### 4. コンテナに接続

```bash
docker-compose exec rustfeed-dev bash
```

これで、Docker環境内のシェルに接続できます。

## 使い方

### Rustプロジェクトのビルド

コンテナ内で以下のコマンドを実行します。

```bash
# ビルド
cargo build

# リリースビルド
cargo build --release

# テスト実行
cargo test

# CLI実行
cargo run --bin rustfeed-cli -- --help

# TUI実行
cargo run --bin rustfeed-tui
```

### Claude Codeの使用

コンテナ内でClaude Codeを起動します。

```bash
# Claude Code CLIの起動
claude-code

# 特定のプロンプトで起動
claude-code "rustfeedのコードをレビューしてください"
```

### データの永続化

以下のデータは、Dockerボリュームとして永続化されます。

- `cargo-cache`: Cargoのレジストリキャッシュ
- `cargo-git-cache`: Cargoのgitキャッシュ
- `target-cache`: ビルド成果物
- `rustfeed-data`: アプリケーションデータ（`~/.rustfeed/`）
- `claude-config`: Claude Code設定

コンテナを削除してもこれらのデータは保持されます。

## よくあるコマンド

### コンテナの状態確認

```bash
# 実行中のコンテナを確認
docker-compose ps

# ログを確認
docker-compose logs -f
```

### コンテナの停止・再起動

```bash
# コンテナの停止
docker-compose stop

# コンテナの再起動
docker-compose restart

# コンテナの停止と削除
docker-compose down
```

### コンテナのクリーンアップ

```bash
# コンテナとボリュームを削除（データも削除される）
docker-compose down -v

# イメージを再ビルド（キャッシュなし）
docker-compose build --no-cache
```

### ホストとコンテナ間でファイルをコピー

プロジェクトディレクトリは自動的にマウントされているため、通常は不要です。
ただし、必要に応じて以下のコマンドでコピーできます。

```bash
# ホスト → コンテナ
docker cp ./file.txt rustfeed-dev:/workspace/

# コンテナ → ホスト
docker cp rustfeed-dev:/workspace/file.txt ./
```

## トラブルシューティング

### ビルドエラーが発生する

キャッシュをクリアして再ビルドしてください。

```bash
docker-compose down -v
docker-compose build --no-cache
docker-compose up -d
```

### API Keyが認識されない

`.env`ファイルが正しく設定されているか確認してください。
変更後はコンテナを再起動する必要があります。

```bash
docker-compose restart
```

### ディスク容量が不足する

未使用のDockerリソースをクリーンアップしてください。

```bash
# 未使用のコンテナ、ネットワーク、イメージを削除
docker system prune -a

# ボリュームも含めて削除
docker system prune -a --volumes
```

## 開発ワークフロー

### 1. コンテナ起動

```bash
docker-compose up -d
docker-compose exec rustfeed-dev bash
```

### 2. 開発作業

コンテナ内で通常のRust開発を行います。

```bash
# コードを編集（ホスト側でも可能）
# vim, nano などのエディタが利用可能

# ビルド＆テスト
cargo build
cargo test

# Claude Codeでコードレビュー
claude-code "変更をレビューしてください"
```

### 3. コミット

コンテナ内またはホスト側でgit操作を行います。

```bash
git add .
git commit -m "[機能追加] 新機能を実装"
git push
```

### 4. 終了

```bash
# コンテナから抜ける
exit

# コンテナを停止（オプション）
docker-compose stop
```

## セキュリティに関する注意

- `.env`ファイルは絶対にコミットしないでください
- API Keyは安全に管理してください
- 本番環境では適切なセキュリティ設定を行ってください

## 参考情報

- [Docker公式ドキュメント](https://docs.docker.com/)
- [Docker Compose公式ドキュメント](https://docs.docker.com/compose/)
- [Claude Code CLI](https://github.com/anthropics/claude-code)
- [Rust公式ドキュメント](https://doc.rust-lang.org/)
