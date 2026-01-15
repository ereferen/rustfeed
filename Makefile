.PHONY: help setup build up down shell claude clean rebuild test fmt check validate test-env

# デフォルトターゲット
help:
	@echo "rustfeed Docker環境 - 利用可能なコマンド:"
	@echo ""
	@echo "セットアップ＆検証:"
	@echo "  make validate   - Docker設定ファイルを検証"
	@echo "  make test-env   - Docker環境の動作確認（自動テスト）"
	@echo "  make setup      - 初回セットアップ（.env作成 + ビルド）"
	@echo ""
	@echo "コンテナ管理:"
	@echo "  make build      - Dockerイメージをビルド"
	@echo "  make up         - コンテナを起動"
	@echo "  make down       - コンテナを停止"
	@echo "  make shell      - コンテナにシェルで接続"
	@echo "  make clean      - コンテナとボリュームを削除"
	@echo "  make rebuild    - クリーン後に再ビルド"
	@echo ""
	@echo "開発ツール:"
	@echo "  make claude     - Claude Codeを起動"
	@echo "  make test       - テスト実行（コンテナ内）"
	@echo "  make fmt        - コードフォーマット（コンテナ内）"
	@echo "  make check      - コンパイルチェック（コンテナ内）"
	@echo ""

# 初回セットアップ
setup:
	@if [ ! -f .env ]; then \
		echo ".envファイルを作成しています..."; \
		cp .env.example .env; \
		echo ".envファイルを作成しました。ANTHROPIC_API_KEYを設定してください。"; \
	else \
		echo ".envファイルは既に存在します。"; \
	fi
	@echo "Dockerイメージをビルドしています..."
	docker-compose build

# Dockerイメージのビルド
build:
	docker-compose build

# コンテナの起動
up:
	docker-compose up -d
	@echo "コンテナが起動しました。'make shell'で接続できます。"

# コンテナの停止
down:
	docker-compose down

# コンテナにシェルで接続
shell:
	docker-compose exec rustfeed-dev bash

# Claude Codeを起動
claude:
	docker-compose exec rustfeed-dev claude-code

# テスト実行
test:
	docker-compose exec rustfeed-dev cargo test

# コードフォーマット
fmt:
	docker-compose exec rustfeed-dev cargo fmt

# コンパイルチェック
check:
	docker-compose exec rustfeed-dev cargo check

# クリーンアップ
clean:
	docker-compose down -v
	@echo "コンテナとボリュームを削除しました。"

# 再ビルド
rebuild: clean
	docker-compose build --no-cache
	@echo "再ビルドが完了しました。"

# Docker設定ファイルの検証
validate:
	@echo "Docker設定ファイルを検証しています..."
	python3 scripts/validate-docker.py

# Docker環境の動作確認
test-env:
	@echo "Docker環境の動作確認を開始します..."
	bash scripts/test-docker-env.sh
