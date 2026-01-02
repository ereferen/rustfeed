# 05. セキュリティとCapabilities

このセクションでは、Tauri 2.0のセキュリティモデル、特にCapabilitiesシステムについて学びます。

## Tauriのセキュリティ哲学

Tauri は「最小権限の原則」に基づいて設計されています：

1. **デフォルトで安全**: 何も許可しなければ何もできない
2. **明示的な許可**: 必要な権限は明示的に宣言する
3. **細かい粒度**: 権限は細かく制御できる

## Capabilities とは

Capabilities は、アプリケーションが使用できる機能を定義するJSONファイルです。

```
crates/rustfeed-tauri/
├── src/
├── tauri.conf.json
└── capabilities/
    ├── default.json      # デフォルト権限
    └── main-window.json  # メインウィンドウ用
```

### 基本構造

```json
// capabilities/default.json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Default capabilities for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "shell:allow-open"
  ]
}
```

## 権限の種類

### コア権限

```json
{
  "permissions": [
    "core:default",           // 基本的なTauri機能
    "core:window:allow-close", // ウィンドウを閉じる
    "core:window:allow-minimize", // 最小化
    "core:app:allow-version"  // バージョン情報取得
  ]
}
```

### プラグイン権限

各プラグインには独自の権限があります：

```json
{
  "permissions": [
    // Shell プラグイン
    "shell:allow-open",        // URLやファイルを開く
    "shell:allow-execute",     // コマンド実行（危険）

    // HTTP プラグイン
    "http:default",            // HTTP リクエスト

    // File System プラグイン
    "fs:allow-read",           // ファイル読み取り
    "fs:allow-write",          // ファイル書き込み

    // Dialog プラグイン
    "dialog:allow-open",       // ファイル選択ダイアログ
    "dialog:allow-save"        // 保存ダイアログ
  ]
}
```

## rustfeed-gui の Capabilities

rustfeedで必要な権限を定義します：

```json
// crates/rustfeed-tauri/capabilities/default.json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capabilities for rustfeed GUI",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:default",
    "core:app:default",

    "shell:allow-open",

    {
      "identifier": "http:default",
      "allow": [
        { "url": "https://**" },
        { "url": "http://**" }
      ]
    }
  ]
}
```

### 権限の制限

特定のURLやパスのみ許可することもできます：

```json
{
  "permissions": [
    {
      "identifier": "http:default",
      "allow": [
        { "url": "https://example.com/**" },
        { "url": "https://api.example.com/**" }
      ],
      "deny": [
        { "url": "https://evil.com/**" }
      ]
    },
    {
      "identifier": "fs:allow-read",
      "allow": [
        { "path": "$APPDATA/**" },
        { "path": "$HOME/.rustfeed/**" }
      ]
    }
  ]
}
```

## ウィンドウごとの権限

異なるウィンドウに異なる権限を設定できます：

```json
// capabilities/main-window.json
{
  "identifier": "main-window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "shell:allow-open",
    "http:default"
  ]
}

// capabilities/settings-window.json
{
  "identifier": "settings-window",
  "windows": ["settings"],
  "permissions": [
    "core:default",
    "fs:allow-read",
    "fs:allow-write"
  ]
}
```

## tauri.conf.json との関係

`tauri.conf.json` でもセキュリティ設定ができます：

```json
// tauri.conf.json
{
  "app": {
    "security": {
      "csp": "default-src 'self'; img-src 'self' https:; style-src 'self' 'unsafe-inline'",
      "freezePrototype": true,
      "dangerousDisableAssetCspModification": false
    },
    "windows": [
      {
        "label": "main",
        "title": "rustfeed",
        "width": 1200,
        "height": 800
      }
    ]
  }
}
```

### CSP (Content Security Policy)

WebViewで実行できるスクリプトやリソースを制限します：

```json
{
  "security": {
    "csp": {
      "default-src": "'self'",
      "script-src": "'self'",
      "style-src": "'self' 'unsafe-inline'",
      "img-src": "'self' https: data:",
      "connect-src": "'self' https:"
    }
  }
}
```

## Electron との比較

| 観点 | Tauri | Electron |
|------|-------|----------|
| デフォルト | 全て拒否 | 全て許可 |
| 権限制御 | Capabilities (細かい) | 粗い |
| Node.js API | なし | フルアクセス可能 |
| ファイルシステム | 明示的許可必要 | 制限なし |
| ネットワーク | URL制限可能 | 制限困難 |

## よくある権限パターン

### RSSリーダー（rustfeed）

```json
{
  "permissions": [
    "core:default",
    "shell:allow-open",           // ブラウザで記事を開く
    {
      "identifier": "http:default",
      "allow": [{ "url": "https://**" }, { "url": "http://**" }]
    }
  ]
}
```

### ファイルエディタ

```json
{
  "permissions": [
    "core:default",
    "dialog:allow-open",
    "dialog:allow-save",
    "fs:allow-read",
    "fs:allow-write"
  ]
}
```

### 最小限のアプリ

```json
{
  "permissions": [
    "core:default"
  ]
}
```

## セキュリティのベストプラクティス

### 1. 最小権限の原則

```json
// ✗ 悪い例：必要以上の権限
{
  "permissions": [
    "fs:default",        // 全ファイルアクセス
    "shell:default",     // 全シェルコマンド
    "http:default"       // 全HTTPアクセス
  ]
}

// ✓ 良い例：必要最小限
{
  "permissions": [
    {
      "identifier": "fs:allow-read",
      "allow": [{ "path": "$APPDATA/rustfeed/**" }]
    },
    "shell:allow-open",
    {
      "identifier": "http:default",
      "allow": [{ "url": "https://**" }]
    }
  ]
}
```

### 2. ユーザー入力の検証

```rust
#[tauri::command]
async fn add_feed(url: String) -> Result<Feed, String> {
    // URLの検証
    let parsed = url::Url::parse(&url)
        .map_err(|_| "Invalid URL format")?;

    // スキームの確認
    if !["http", "https"].contains(&parsed.scheme()) {
        return Err("Only HTTP/HTTPS URLs are allowed".to_string());
    }

    // プライベートIPの拒否（オプション）
    if is_private_ip(parsed.host_str().unwrap_or("")) {
        return Err("Private IPs are not allowed".to_string());
    }

    // 実際のフィード取得
    fetch_and_save(parsed).await
}
```

### 3. エラーメッセージの注意

```rust
// ✗ 悪い例：詳細すぎるエラー
Err(format!("Database error at {}: {}", db_path, e))

// ✓ 良い例：適切な情報量
Err("Failed to save feed".to_string())
```

## 理解度確認

1. Capabilitiesの目的は何ですか？
2. 「最小権限の原則」をTauriでどのように実践しますか？
3. CSPは何を制御しますか？

## 次のステップ

[06-build-deploy.md](./06-build-deploy.md)で、各プラットフォーム向けのビルドとデプロイについて学びましょう。
