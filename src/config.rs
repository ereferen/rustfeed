//! # 設定ファイル管理モジュール
//!
//! このモジュールは、rustfeed の設定ファイル（TOML形式）の読み込みと管理を提供します。
//!
//! ## 設定ファイルの場所
//!
//! XDG Base Directory Specification に準拠し、以下の場所に設定ファイルを配置します：
//! - Linux/macOS: `~/.config/rustfeed/config.toml`
//! - Windows: `%APPDATA%\rustfeed\config.toml`
//!
//! ## 設定項目
//!
//! ```toml
//! [general]
//! default_limit = 20
//! show_unread_only = false
//! disabled_feeds = [2, 3]
//!
//! [display]
//! date_format = "%Y-%m-%d"
//! show_description = true
//!
//! [database]
//! path = "~/.rustfeed/rustfeed.db"
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// アプリケーション全体の設定
///
/// # フィールド
///
/// - `general`: 一般的な設定（デフォルトリミット、未読のみ表示など）
/// - `display`: 表示に関する設定（日付フォーマット、説明表示など）
/// - `database`: データベースに関する設定（パスなど）
///
/// # デフォルト値
///
/// 設定ファイルが存在しない、または一部の設定が欠けている場合、
/// `Default` トレイトで定義されたデフォルト値が使用されます。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct AppConfig {
    #[serde(default)]
    pub general: GeneralConfig,

    #[serde(default)]
    pub display: DisplayConfig,

    #[serde(default)]
    pub database: DatabaseConfig,
}

/// 一般的な設定
///
/// # フィールド
///
/// - `default_limit`: デフォルトの記事表示件数
/// - `show_unread_only`: デフォルトで未読のみ表示するか
/// - `disabled_feeds`: 無効化するフィードのIDリスト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_limit")]
    pub default_limit: usize,

    #[serde(default)]
    pub show_unread_only: bool,

    #[serde(default)]
    pub disabled_feeds: Vec<i64>,
}

/// 表示に関する設定
///
/// # フィールド
///
/// - `date_format`: 日付のフォーマット文字列（chrono形式）
/// - `show_description`: フィードの説明を表示するか
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_date_format")]
    pub date_format: String,

    #[serde(default = "default_true")]
    pub show_description: bool,
}

/// データベースに関する設定
///
/// # フィールド
///
/// - `path`: データベースファイルのパス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    #[serde(default = "default_db_path")]
    pub path: String,
}

// =============================================================================
// デフォルト値関数
// =============================================================================

/// デフォルトのリミット値
fn default_limit() -> usize {
    20
}

/// デフォルトの日付フォーマット
fn default_date_format() -> String {
    "%Y-%m-%d".to_string()
}

/// デフォルトのデータベースパス
fn default_db_path() -> String {
    "~/.rustfeed/rustfeed.db".to_string()
}

/// デフォルトでtrueを返す
fn default_true() -> bool {
    true
}

// =============================================================================
// Default トレイト実装
// =============================================================================

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            default_limit: default_limit(),
            show_unread_only: false,
            disabled_feeds: Vec::new(),
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            date_format: default_date_format(),
            show_description: true,
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: default_db_path(),
        }
    }
}


// =============================================================================
// 設定ファイルの読み込み
// =============================================================================

impl AppConfig {
    /// 設定ファイルを読み込む
    ///
    /// # 読み込み順序
    ///
    /// 1. デフォルト値を使用
    /// 2. 設定ファイルが存在する場合、上書き
    ///
    /// # エラー
    ///
    /// 設定ファイルが存在するが、パースエラーがある場合はエラーを返します。
    /// ファイルが存在しない場合は、デフォルト値を使用します。
    ///
    /// # 例
    ///
    /// ```rust
    /// let config = AppConfig::load()?;
    /// println!("Default limit: {}", config.general.default_limit);
    /// ```
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        // 設定ファイルが存在しない場合はデフォルト値を返す
        if !config_path.exists() {
            return Ok(Self::default());
        }

        // 設定ファイルを読み込む
        let settings = config::Config::builder()
            .add_source(config::File::from(config_path))
            .build()
            .context("Failed to build config")?;

        // 設定をデシリアライズ
        let config: AppConfig = settings
            .try_deserialize()
            .context("Failed to deserialize config")?;

        Ok(config)
    }

    /// 設定ファイルのパスを取得する
    ///
    /// # XDG Base Directory Specification
    ///
    /// - Linux/macOS: `$XDG_CONFIG_HOME/rustfeed/config.toml` または `~/.config/rustfeed/config.toml`
    /// - Windows: `%APPDATA%\rustfeed\config.toml`
    ///
    /// # エラー
    ///
    /// ホームディレクトリが取得できない場合はエラーを返します。
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("rustfeed");

        Ok(config_dir.join("config.toml"))
    }

    /// 設定ファイルのサンプルを文字列として返す
    ///
    /// 新規ユーザー向けに、設定ファイルのサンプルを提供します。
    ///
    /// # 例
    ///
    /// ```rust
    /// println!("{}", AppConfig::sample_config());
    /// ```
    pub fn sample_config() -> &'static str {
        r#"# rustfeed 設定ファイル
# ~/.config/rustfeed/config.toml

[general]
# デフォルトの記事表示件数
default_limit = 20

# デフォルトで未読のみ表示するか
show_unread_only = false

# 無効化するフィードのIDリスト（記事を表示したくないフィード）
# 例: disabled_feeds = [2, 3]
disabled_feeds = []

[display]
# 日付のフォーマット（chrono形式）
# %Y: 年, %m: 月, %d: 日, %H: 時, %M: 分, %S: 秒
date_format = "%Y-%m-%d"

# フィードの説明を表示するか
show_description = true

[database]
# データベースファイルのパス
# ~ はホームディレクトリに展開されます
path = "~/.rustfeed/rustfeed.db"
"#
    }
}
