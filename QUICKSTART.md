# クイックスタートガイド

このガイドでは、Docker環境でrustfeedを開発し、Claude Codeを使用する最も簡単な方法を説明します。

**所要時間**: 5-10分

**前提条件**: Docker、Docker Composeがインストールされていること

## ステップ1: リポジトリのクローン

```bash
git clone https://github.com/ereferen/rustfeed.git
cd rustfeed
```

## ステップ2: 自動セットアップとテスト（推奨）

```bash
make test-env
```

このコマンドが以下を自動で実行します：
- Dockerのチェック
- Dockerイメージのビルド
- コンテナの起動
- Rust環境の確認
- Claude Code CLIの確認

**注意**: 初回ビルドは数分かかります。

## ステップ3: Claude Codeを使用する

### コンテナに接続

```bash
make shell
```

### Claude Codeを起動

```bash
claude-code
```

### 初回起動時のログイン

初めてClaude Codeを起動すると、以下のようなメッセージが表示されます。

```
Please visit this URL to authenticate:
https://console.anthropic.com/auth/...
```

このURLを**コピー**して、**ホスト側（あなたのPC）のブラウザ**で開いてください。

### Anthropicアカウントでログイン

1. ブラウザでURLを開く
2. Anthropicアカウントでログイン（アカウントがない場合は作成）
3. 認証を許可

### 認証完了

ブラウザでログインが完了すると、コンテナ内のClaude Codeが自動的に認証されます。

```
✅ Authentication successful!
Welcome to Claude Code!
```

これで、Claude Codeを使用できるようになりました。

## ステップ4: rustfeedを開発する

### プロジェクトのビルド

```bash
cargo build
```

### テストの実行

```bash
cargo test
```

### Claude Codeで開発

```bash
# 対話モード
claude-code

# プロンプトを指定
claude-code "rustfeedのコードをレビューしてください"
```

## よくある質問

### Q: APIキーは必要ですか？

**A**: いいえ、APIキーは不要です。ブラウザでログインするだけで使用できます。

### Q: 毎回ログインする必要がありますか？

**A**: いいえ、ログイン情報は永続化されます。次回以降はログイン不要です。

### Q: コンテナを削除したらログイン情報は消えますか？

**A**: `make down`でコンテナを停止しても、ログイン情報は保持されます。`make clean`を実行すると削除されます。

### Q: APIキーを使いたい場合は？

**A**: `.env`ファイルに`ANTHROPIC_API_KEY`を設定してください。詳細は[DOCKER.md](./DOCKER.md)を参照してください。

## トラブルシューティング

### Dockerがインストールされていない

Docker Desktopをインストールしてください。
- [macOS](https://docs.docker.com/desktop/install/mac-install/)
- [Windows](https://docs.docker.com/desktop/install/windows-install/)
- [Linux](https://docs.docker.com/engine/install/)

### ビルドエラーが発生する

```bash
make clean
make rebuild
```

### Claude Codeが起動しない

```bash
# コンテナを再起動
make down
make up
make shell
claude-code
```

## 次のステップ

- [DOCKER.md](./DOCKER.md): Docker環境の詳細な使い方
- [CLAUDE.md](./CLAUDE.md): rustfeedのプロジェクト構造
- [README.md](./README.md): rustfeedの機能と使い方

## Makefileコマンド一覧

```bash
# セットアップ
make validate   # Docker設定ファイルを検証
make test-env   # Docker環境の自動テスト
make setup      # 初回セットアップ

# コンテナ管理
make up         # コンテナを起動
make down       # コンテナを停止
make shell      # コンテナに接続
make clean      # コンテナとボリュームを削除

# 開発
make claude     # Claude Codeを起動
make check      # コンパイルチェック
make test       # テスト実行
make fmt        # コードフォーマット
```

## サポート

問題が発生した場合は、[Issues](https://github.com/ereferen/rustfeed/issues)で報告してください。
