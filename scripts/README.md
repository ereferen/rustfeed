# テスト・検証スクリプト

このディレクトリには、Docker環境のテストと検証を行うスクリプトが含まれています。

## スクリプト一覧

### 1. validate-docker.py

Docker設定ファイルの構文チェックを行います。

**用途**: Docker環境構築前の事前チェック

**実行方法**:
```bash
python3 scripts/validate-docker.py
```

**チェック項目**:
- Dockerfileの構文
- docker-compose.ymlの構文
- .env.exampleの存在
- .gitignoreに.envが含まれているか

**出力例**:
```
============================================================
Docker環境設定ファイルの検証
============================================================
📋 Dockerfile を検証中...
✅ Dockerfileの検証完了

📋 docker-compose.yml を検証中...
✅ docker-compose.ymlの検証完了

📋 .env.example を検証中...
✅ .env.exampleの検証完了

📋 .gitignore を検証中...
✅ .gitignoreの検証完了

============================================================
✅ すべての検証に成功しました！
```

### 2. test-docker-env.sh

Docker環境の完全な動作確認を行います。

**用途**: Docker環境が正しく動作するかの総合テスト

**実行方法**:
```bash
./scripts/test-docker-env.sh
```

または

```bash
bash scripts/test-docker-env.sh
```

**テスト項目**:
1. Dockerのインストール確認
2. Docker Composeのインストール確認
3. .envファイルの確認（なければ作成）
4. Dockerイメージのビルド
5. コンテナの起動
6. Rust環境の動作確認
7. Claude Code CLIのインストール確認
8. rustfeedプロジェクトの認識確認

**出力例**:
```
============================================================
Docker環境 動作確認テスト
============================================================

[1/8] 前提条件をチェック中...
✅ Docker: Docker version 24.0.0
✅ Docker Compose: Docker Compose version v2.20.0

[2/8] 環境変数ファイルをチェック中...
✅ .envファイルが存在します

[3/8] Dockerイメージをビルド中...
✅ ビルド成功

[4/8] コンテナを起動中...
✅ コンテナ起動成功

[5/8] コンテナの準備を待機中...

[6/8] Rust環境を確認中...
✅ Rust環境が正常に動作しています

[7/8] Claude Code CLIを確認中...
✅ Claude Code CLIがインストールされています

[8/8] rustfeedプロジェクトを確認中...
✅ rustfeedプロジェクトが認識されています

============================================================
✅ Docker環境の動作確認が完了しました！
============================================================
```

## 推奨される使い方

### 初回セットアップ時

1. **設定ファイルの検証**:
   ```bash
   python3 scripts/validate-docker.py
   ```

2. **Docker環境のテスト**:
   ```bash
   ./scripts/test-docker-env.sh
   ```

3. **手動での確認**:
   ```bash
   # コンテナに接続
   make shell

   # Rustバージョン確認
   cargo --version

   # Claude Code確認
   claude-code --version

   # プロジェクトビルド
   cargo check
   ```

### トラブルシューティング時

問題が発生した場合は、以下の順序で確認してください。

1. **設定ファイルの再検証**:
   ```bash
   python3 scripts/validate-docker.py
   ```

2. **Docker環境のクリーンアップ**:
   ```bash
   make clean
   ```

3. **再ビルドとテスト**:
   ```bash
   ./scripts/test-docker-env.sh
   ```

4. **ログの確認**:
   ```bash
   docker-compose logs
   ```

## 前提条件

### 必要なソフトウェア

- **Docker**: 20.10以上
- **Docker Compose**: V2推奨（V1でも動作可能）
- **Python3**: 検証スクリプト用（3.6以上）
- **Bash**: テストスクリプト用

### インストール方法

#### Docker & Docker Compose

- **macOS**: [Docker Desktop for Mac](https://docs.docker.com/desktop/install/mac-install/)
- **Windows**: [Docker Desktop for Windows](https://docs.docker.com/desktop/install/windows-install/)
- **Linux**: [Docker Engine](https://docs.docker.com/engine/install/)

## よくある問題と解決方法

### 1. "docker: command not found"

**原因**: Dockerがインストールされていない

**解決方法**: Docker DesktopまたはDocker Engineをインストール

### 2. "permission denied while trying to connect to the Docker daemon"

**原因**: Dockerデーモンへのアクセス権限がない

**解決方法**:
```bash
# Linuxの場合
sudo usermod -aG docker $USER
# ログアウト後、再ログイン
```

### 3. "ANTHROPIC_API_KEY not set"

**原因**: .envファイルにAPI Keyが設定されていない

**解決方法**:
1. `.env`ファイルを編集
2. `ANTHROPIC_API_KEY=your_actual_api_key`を設定
3. コンテナを再起動: `make down && make up`

### 4. "Port is already in use"

**原因**: 他のコンテナまたはプロセスがポートを使用中

**解決方法**:
```bash
# 使用中のコンテナを確認
docker ps

# 不要なコンテナを停止
docker stop <container_id>
```

## サポート

問題が解決しない場合は、以下の情報を含めてIssueを作成してください。

- OS とバージョン
- Dockerバージョン (`docker --version`)
- Docker Composeバージョン
- エラーメッセージ全文
- `docker-compose logs`の出力

## 参考リンク

- [Docker公式ドキュメント](https://docs.docker.com/)
- [Docker Compose公式ドキュメント](https://docs.docker.com/compose/)
- [rustfeed DOCKER.md](../DOCKER.md)
