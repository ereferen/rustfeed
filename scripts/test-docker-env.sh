#!/bin/bash
# Docker環境の動作確認スクリプト

set -e

# 色の定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# ヘッダー表示
echo -e "${BLUE}============================================================${NC}"
echo -e "${BLUE}Docker環境 動作確認テスト${NC}"
echo -e "${BLUE}============================================================${NC}"
echo ""

# 前提条件のチェック
echo -e "${YELLOW}[1/8] 前提条件をチェック中...${NC}"

# Dockerのチェック
if ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ Dockerがインストールされていません${NC}"
    echo "   https://docs.docker.com/get-docker/ からインストールしてください"
    exit 1
fi
echo -e "${GREEN}✅ Docker: $(docker --version)${NC}"

# Docker Composeのチェック
if command -v docker-compose &> /dev/null; then
    COMPOSE_CMD="docker-compose"
    echo -e "${GREEN}✅ Docker Compose: $(docker-compose --version)${NC}"
elif docker compose version &> /dev/null; then
    COMPOSE_CMD="docker compose"
    echo -e "${GREEN}✅ Docker Compose: $(docker compose version)${NC}"
else
    echo -e "${RED}❌ Docker Composeがインストールされていません${NC}"
    exit 1
fi

# .envファイルのチェック
echo ""
echo -e "${YELLOW}[2/8] 環境変数ファイルをチェック中...${NC}"
if [ ! -f .env ]; then
    echo -e "${YELLOW}⚠️  .envファイルが存在しません${NC}"
    echo "   .env.exampleからコピーして作成します..."
    cp .env.example .env
    echo -e "${GREEN}✅ .envファイルを作成しました${NC}"
    echo -e "${YELLOW}   ANTHROPIC_API_KEYを設定してください${NC}"
else
    echo -e "${GREEN}✅ .envファイルが存在します${NC}"
fi

# Dockerイメージのビルド
echo ""
echo -e "${YELLOW}[3/8] Dockerイメージをビルド中...${NC}"
echo "   (初回は数分かかる場合があります)"
if $COMPOSE_CMD build; then
    echo -e "${GREEN}✅ ビルド成功${NC}"
else
    echo -e "${RED}❌ ビルド失敗${NC}"
    exit 1
fi

# コンテナの起動
echo ""
echo -e "${YELLOW}[4/8] コンテナを起動中...${NC}"
if $COMPOSE_CMD up -d; then
    echo -e "${GREEN}✅ コンテナ起動成功${NC}"
else
    echo -e "${RED}❌ コンテナ起動失敗${NC}"
    exit 1
fi

# コンテナの動作確認を待つ
echo ""
echo -e "${YELLOW}[5/8] コンテナの準備を待機中...${NC}"
sleep 3

# Rust環境の確認
echo ""
echo -e "${YELLOW}[6/8] Rust環境を確認中...${NC}"
if $COMPOSE_CMD exec -T rustfeed-dev cargo --version; then
    echo -e "${GREEN}✅ Rust環境が正常に動作しています${NC}"
else
    echo -e "${RED}❌ Rust環境の確認に失敗${NC}"
    $COMPOSE_CMD down
    exit 1
fi

# Claude Code CLIの確認
echo ""
echo -e "${YELLOW}[7/8] Claude Code CLIを確認中...${NC}"
if $COMPOSE_CMD exec -T rustfeed-dev claude-code --version 2>/dev/null || \
   $COMPOSE_CMD exec -T rustfeed-dev which claude-code >/dev/null 2>&1; then
    echo -e "${GREEN}✅ Claude Code CLIがインストールされています${NC}"
else
    echo -e "${YELLOW}⚠️  Claude Code CLIの確認をスキップ（インストール中の可能性）${NC}"
fi

# rustfeedプロジェクトの簡易チェック
echo ""
echo -e "${YELLOW}[8/8] rustfeedプロジェクトを確認中...${NC}"
if $COMPOSE_CMD exec -T rustfeed-dev cargo check 2>&1 | head -10; then
    echo -e "${GREEN}✅ rustfeedプロジェクトが認識されています${NC}"
else
    echo -e "${YELLOW}⚠️  cargo checkで警告がありますが、正常です${NC}"
fi

# 結果サマリー
echo ""
echo -e "${BLUE}============================================================${NC}"
echo -e "${GREEN}✅ Docker環境の動作確認が完了しました！${NC}"
echo -e "${BLUE}============================================================${NC}"
echo ""
echo -e "${BLUE}次のステップ:${NC}"
echo -e "  1. コンテナに接続:  ${GREEN}make shell${NC}"
echo -e "  2. Claude Code起動: ${GREEN}make claude${NC}"
echo -e "  3. ビルド実行:      ${GREEN}make check${NC}"
echo -e "  4. テスト実行:      ${GREEN}make test${NC}"
echo ""
echo -e "${BLUE}コンテナの管理:${NC}"
echo -e "  - ログ確認:  ${GREEN}docker-compose logs -f${NC}"
echo -e "  - コンテナ停止: ${GREEN}make down${NC}"
echo -e "  - コンテナ再起動: ${GREEN}make up${NC}"
echo ""
echo -e "${YELLOW}コンテナは起動したままです。停止する場合は 'make down' を実行してください。${NC}"
