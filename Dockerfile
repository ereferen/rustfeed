# Rust開発環境 + Claude Code CLI
FROM rust:1.83-slim

# 必要なパッケージのインストール
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libsqlite3-dev \
    curl \
    git \
    ca-certificates \
    nodejs \
    npm \
    && rm -rf /var/lib/apt/lists/*

# Claude Code CLIのインストール
RUN npm install -g @anthropic-ai/claude-code-cli

# 作業ディレクトリの設定
WORKDIR /workspace

# Cargoのキャッシュディレクトリを作成
RUN mkdir -p /usr/local/cargo/registry

# デフォルトユーザーの作成（セキュリティのため）
RUN useradd -m -s /bin/bash rustfeed && \
    chown -R rustfeed:rustfeed /workspace

# ユーザーの切り替え
USER rustfeed

# 環境変数の設定
ENV CARGO_HOME=/usr/local/cargo
ENV PATH=/usr/local/cargo/bin:$PATH

# ヘルスチェック用のスクリプト
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD cargo --version && claude-code --version || exit 1

# デフォルトコマンド
CMD ["/bin/bash"]
