# 02. エラー処理

Rustのエラー処理は、例外（exception）を使わず、型システムを活用した明示的なアプローチを取ります。これにより、エラーの可能性が型に表れ、コンパイラが処理漏れを検出できます。

## 目次

1. [Result型とOption型](#1-result型とoption型)
2. [エラー伝播と?演算子](#2-エラー伝播と演算子)
3. [パターンマッチによるエラー処理](#3-パターンマッチによるエラー処理)
4. [anyhowとthiserror](#4-anyhowとthiserror)
5. [rustfeedでの実例](#5-rustfeedでの実例)

---

## 1. Result型とOption型

### Result<T, E>

`Result` は「成功または失敗」を表す型です：

```rust
enum Result<T, E> {
    Ok(T),   // 成功：値Tを含む
    Err(E),  // 失敗：エラーEを含む
}
```

### Option<T>

`Option` は「値があるかもしれない」を表す型です：

```rust
enum Option<T> {
    Some(T),  // 値が存在する
    None,     // 値が存在しない
}
```

### C++との比較

**C++** のエラー処理アプローチ：

```cpp
// 方法1: 例外（従来型）
std::string read_file(const std::string& path) {
    std::ifstream file(path);
    if (!file) {
        throw std::runtime_error("File not found");
    }
    // ...
}

// 方法2: std::optional (C++17)
std::optional<std::string> find_user(int id) {
    if (/* ユーザーが見つからない */) {
        return std::nullopt;
    }
    return "user_name";
}

// 方法3: std::expected (C++23)
std::expected<int, std::string> parse_int(std::string_view s) {
    // ...
}
```

**Rust** では `Result` と `Option` が標準：

```rust
// 失敗する可能性がある操作
fn read_file(path: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(path)
}

// 値がないかもしれない操作
fn find_user(id: i32) -> Option<String> {
    // ...
}
```

### 重要な違い

| 特徴 | C++ 例外 | Rust Result |
|------|----------|-------------|
| エラーの可視性 | 関数シグネチャに現れない | 型として明示される |
| 処理の強制 | 処理しなくてもコンパイル通る | 未処理は警告される |
| パフォーマンス | ゼロコスト（成功時）またはオーバーヘッド | 常に予測可能 |
| スタックアンワインド | あり | なし（panic除く） |

### TypeScriptとの比較

**TypeScript** では、多くの場合 `null/undefined` か例外を使います：

```typescript
// null/undefined パターン
function findUser(id: number): User | undefined {
    const user = users.find(u => u.id === id);
    return user;  // undefined の可能性
}

// 例外パターン
async function fetchData(url: string): Promise<Data> {
    const response = await fetch(url);
    if (!response.ok) {
        throw new Error(`HTTP error: ${response.status}`);
    }
    return response.json();
}
```

**Rust** のアプローチとの違い：

```rust
// Option: null/undefined の代わり
fn find_user(id: i32) -> Option<User> {
    users.iter().find(|u| u.id == id).cloned()
}

// Result: 例外の代わり
async fn fetch_data(url: &str) -> Result<Data, reqwest::Error> {
    let response = reqwest::get(url).await?;
    let data = response.json().await?;
    Ok(data)
}
```

---

### 理解度確認 1

**問題**: 以下のTypeScript関数をRustで書き直してください。

```typescript
function divide(a: number, b: number): number {
    if (b === 0) {
        throw new Error("Division by zero");
    }
    return a / b;
}
```

<details>
<summary>回答を見る</summary>

```rust
fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err(String::from("Division by zero"))
    } else {
        Ok(a / b)
    }
}

// 使用例
fn main() {
    match divide(10.0, 2.0) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Error: {}", e),
    }
}
```

または、`Option` を使う方法もあります（エラー情報が不要な場合）：

```rust
fn divide(a: f64, b: f64) -> Option<f64> {
    if b == 0.0 {
        None
    } else {
        Some(a / b)
    }
}
```

</details>

---

## 2. エラー伝播と?演算子

### ?演算子の基本

`?` 演算子は、`Result` または `Option` の値を展開し、エラーの場合は早期リターンします：

```rust
fn read_username() -> Result<String, std::io::Error> {
    let contents = std::fs::read_to_string("username.txt")?;
    //                                                     ^ エラー時は即return
    Ok(contents.trim().to_string())
}
```

これは以下と同等です：

```rust
fn read_username() -> Result<String, std::io::Error> {
    let contents = match std::fs::read_to_string("username.txt") {
        Ok(c) => c,
        Err(e) => return Err(e),
    };
    Ok(contents.trim().to_string())
}
```

### 連鎖的なエラー伝播

`?` は連鎖させることができます：

```rust
fn get_user_age() -> Result<u32, Box<dyn std::error::Error>> {
    let contents = std::fs::read_to_string("age.txt")?;  // ファイル読み込みエラー
    let age: u32 = contents.trim().parse()?;              // パースエラー
    Ok(age)
}
```

### C++との比較

**C++** では例外が自動的に伝播します：

```cpp
std::string read_username() {
    std::ifstream file("username.txt");
    if (!file) throw std::runtime_error("Cannot open file");

    std::string contents;
    std::getline(file, contents);
    return contents;
}

void caller() {
    try {
        std::string name = read_username();
        // 使用
    } catch (const std::exception& e) {
        // エラー処理
    }
}
```

**Rust** では明示的に伝播させます：

```rust
fn read_username() -> Result<String, std::io::Error> {
    let contents = std::fs::read_to_string("username.txt")?;
    Ok(contents)
}

fn caller() -> Result<(), std::io::Error> {
    let name = read_username()?;  // 明示的に伝播
    // 使用
    Ok(())
}
```

### TypeScript（React）との比較

**React** でのエラー処理：

```typescript
// Reactコンポーネントでの非同期処理
const UserProfile: React.FC<{ userId: number }> = ({ userId }) => {
    const [user, setUser] = useState<User | null>(null);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        fetchUser(userId)
            .then(setUser)
            .catch(e => setError(e.message));
    }, [userId]);

    if (error) return <div>Error: {error}</div>;
    if (!user) return <div>Loading...</div>;
    return <div>{user.name}</div>;
};
```

**Rust** の対応するパターン：

```rust
// Rustでは Result を返す関数として表現
async fn get_user_profile(user_id: i32) -> Result<User, AppError> {
    let user = fetch_user(user_id).await?;  // エラーは?で伝播
    Ok(user)
}

// 呼び出し側でハンドリング
match get_user_profile(42).await {
    Ok(user) => println!("User: {}", user.name),
    Err(e) => eprintln!("Error: {}", e),
}
```

---

### 理解度確認 2

**問題**: 以下のRustコードで `?` が使えない理由は何でしょうか？

```rust
fn main() {
    let contents = std::fs::read_to_string("file.txt")?;
    println!("{}", contents);
}
```

<details>
<summary>回答を見る</summary>

**`main` 関数の戻り値が `()` だからです。**

`?` 演算子はエラーを「返す」ため、関数が `Result` か `Option` を返す必要があります。

修正方法：

```rust
// 方法1: main関数にResultを返させる
fn main() -> Result<(), std::io::Error> {
    let contents = std::fs::read_to_string("file.txt")?;
    println!("{}", contents);
    Ok(())
}

// 方法2: main内でマッチする
fn main() {
    match std::fs::read_to_string("file.txt") {
        Ok(contents) => println!("{}", contents),
        Err(e) => eprintln!("Error: {}", e),
    }
}

// 方法3: unwrapを使う（プロトタイプ向け、本番では非推奨）
fn main() {
    let contents = std::fs::read_to_string("file.txt").unwrap();
    println!("{}", contents);
}
```

</details>

---

## 3. パターンマッチによるエラー処理

### match式でのエラー処理

`Result` と `Option` は `match` で網羅的に処理できます：

```rust
fn process_file(path: &str) -> Result<(), std::io::Error> {
    match std::fs::read_to_string(path) {
        Ok(contents) => {
            println!("File contents: {}", contents);
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to read file: {}", e);
            Err(e)
        }
    }
}
```

### if letパターン

成功ケースだけを処理したい場合：

```rust
if let Ok(contents) = std::fs::read_to_string("file.txt") {
    println!("{}", contents);
}
// エラーは無視される
```

### メソッドチェーン

`Option` と `Result` には便利なメソッドがあります：

```rust
// unwrap_or: デフォルト値を指定
let name = get_username().unwrap_or(String::from("Anonymous"));

// unwrap_or_else: デフォルト値を遅延評価
let name = get_username().unwrap_or_else(|| generate_random_name());

// map: 成功時に変換
let length: Option<usize> = get_username().map(|s| s.len());

// and_then: 成功時に別のOption/Resultを返す操作
let user: Option<User> = get_user_id()
    .and_then(|id| find_user(id));

// ok_or: Option を Result に変換
let user: Result<User, &str> = find_user(42).ok_or("User not found");
```

### C++との比較

**C++ std::optional** のメソッド：

```cpp
std::optional<std::string> get_username();

// value_or: デフォルト値
std::string name = get_username().value_or("Anonymous");

// transform (C++23): map相当
auto length = get_username().transform([](auto& s) { return s.length(); });

// and_then (C++23)
auto user = get_user_id().and_then([](int id) { return find_user(id); });
```

**違い**: RustはC++23より前から同様の機能を持ち、より広く使われています。

---

### 理解度確認 3

**問題**: 以下のコードを `match` を使わずに書き直してください。

```rust
fn get_first_word_length(text: Option<String>) -> usize {
    match text {
        Some(s) => {
            match s.split_whitespace().next() {
                Some(word) => word.len(),
                None => 0,
            }
        }
        None => 0,
    }
}
```

<details>
<summary>回答を見る</summary>

```rust
fn get_first_word_length(text: Option<String>) -> usize {
    text.as_deref()                     // Option<String> -> Option<&str>
        .and_then(|s| s.split_whitespace().next())  // Option<&str>
        .map(|word| word.len())         // Option<usize>
        .unwrap_or(0)                   // usize
}
```

または：

```rust
fn get_first_word_length(text: Option<String>) -> usize {
    text.and_then(|s| {
        s.split_whitespace()
            .next()
            .map(|word| word.len())
    })
    .unwrap_or(0)
}
```

メソッドチェーンを使うことで：
- ネストが浅くなる
- 処理の流れが明確になる
- 各ステップが独立して理解しやすい

</details>

---

## 4. anyhowとthiserror

### anyhow クレート

`anyhow` は、アプリケーションレベルのエラー処理を簡潔にするクレートです：

```rust
use anyhow::{Context, Result};

// anyhow::Result<T> = Result<T, anyhow::Error>
fn read_config() -> Result<Config> {
    let contents = std::fs::read_to_string("config.toml")
        .with_context(|| "Failed to read config file")?;

    let config: Config = toml::from_str(&contents)
        .with_context(|| "Failed to parse config")?;

    Ok(config)
}
```

**特徴**:
- `anyhow::Error` はどんなエラー型も受け入れる
- `.context()` / `.with_context()` でエラーに情報を追加
- バックトレースをサポート

### thiserror クレート

`thiserror` は、カスタムエラー型の定義を簡潔にします：

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("Item not found: {0}")]
    NotFound(String),
}
```

### 使い分け

| クレート | 用途 | 使用場所 |
|----------|------|----------|
| `anyhow` | エラーを簡単に伝播 | アプリケーション（バイナリ） |
| `thiserror` | カスタムエラー型を定義 | ライブラリ |

### TypeScriptとの比較

**TypeScript** でのカスタムエラー：

```typescript
class AppError extends Error {
    constructor(
        message: string,
        public readonly code: string,
        public readonly cause?: Error
    ) {
        super(message);
        this.name = 'AppError';
    }
}

class DatabaseError extends AppError {
    constructor(cause: Error) {
        super('Database operation failed', 'DB_ERROR', cause);
    }
}
```

**Rust** の `thiserror`：

```rust
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database operation failed: {0}")]
    Database(#[source] rusqlite::Error),
}
```

Rustの方が：
- ボイラープレートが少ない
- エラーチェーンが自動的に処理される
- パターンマッチで網羅的に処理できる

---

### 理解度確認 4

**問題**: 以下の `anyhow` を使ったコードで、エラーメッセージはどのように表示されますか？

```rust
use anyhow::{Context, Result};

fn inner() -> Result<()> {
    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "file missing"))
        .context("Failed to read data file")?
}

fn outer() -> Result<()> {
    inner().context("Initialization failed")?
}

fn main() {
    if let Err(e) = outer() {
        eprintln!("Error: {:?}", e);
    }
}
```

<details>
<summary>回答を見る</summary>

エラーメッセージは以下のようにチェーンとして表示されます：

```
Error: Initialization failed

Caused by:
    0: Failed to read data file
    1: file missing
```

`anyhow` は `context()` で追加された情報を「エラーチェーン」として保持します。`:?` フォーマットで出力すると、最外側のコンテキストから最内側の元のエラーまで順番に表示されます。

これは以下のように展開されます：
1. `outer()` が追加した "Initialization failed"
2. `inner()` が追加した "Failed to read data file"
3. 元の `std::io::Error` の "file missing"

</details>

---

## 5. rustfeedでの実例

### 例1: fetch_feed関数のエラー処理

[crates/rustfeed-core/src/feed.rs](../../crates/rustfeed-core/src/feed.rs) の `fetch_feed` 関数：

```rust
pub async fn fetch_feed(url: &str) -> Result<(Feed, Vec<Article>)> {
    // HTTPリクエスト - ネットワークエラーの可能性
    let response = reqwest::get(url)
        .await
        .with_context(|| format!("Failed to fetch feed from {}", url))?;

    // レスポンス読み取り - I/Oエラーの可能性
    let bytes = response
        .bytes()
        .await
        .with_context(|| "Failed to read response body")?;

    // フィードパース - パースエラーの可能性
    let parsed = parser::parse(&bytes[..])
        .with_context(|| "Failed to parse feed")?;

    // ... 成功時の処理 ...

    Ok((feed, articles))
}
```

**ポイント**:
- 各操作で `?` を使って即座にエラーを伝播
- `with_context()` で何が失敗したかの情報を追加
- フォーマット文字列で動的な情報（URL）を含める

### 例2: フォールバックパターン

同じファイルから、Option のフォールバック：

```rust
// 公開日時を取得（published または updated から）
let published_at: Option<DateTime<Utc>> = entry
    .published
    .or(entry.updated)  // published がなければ updated を試す
    .map(|dt| dt.with_timezone(&Utc));
```

**ポイント**:
- `or()` は `Option` のフォールバック
- 最初の `Some` が見つかるまで順番に試す
- `map()` で見つかった値を変換

### 例3: 個別エラーの処理

[crates/rustfeed-cli/src/commands.rs](../../crates/rustfeed-cli/src/commands.rs) の `fetch_feeds` 関数：

```rust
pub async fn fetch_feeds(db: &Database) -> Result<()> {
    let feeds = db.get_feeds(None)?;

    for stored_feed in feeds {
        // 各フィードを個別に処理し、1つの失敗が他に影響しない
        match feed::fetch_feed(&stored_feed.url).await {
            Ok((_feed_info, articles)) => {
                // 成功：記事を保存
                for mut article in articles {
                    article.feed_id = stored_feed.id;
                    if db.add_article(&article)?.is_some() {
                        // 新規記事がある
                    }
                }
                println!("{}", "OK".green());
            }
            Err(e) => {
                // 失敗：エラーを表示して続行
                println!("{} ({})", "ERROR".red(), e);
            }
        }
    }

    Ok(())
}
```

**ポイント**:
- `?` を使わず `match` で処理することで、一部の失敗を許容
- 関数全体は `Result<()>` を返すが、内部の各操作は個別にハンドリング
- ユーザーにエラーを表示しつつ、他のフィードの処理を継続

### 例4: Option と Result の組み合わせ

```rust
// データベースに記事を追加（重複の場合は None を返す）
pub fn add_article(&self, article: &Article) -> Result<Option<i64>> {
    match self.conn.execute(/* INSERT */) {
        Ok(_) => Ok(Some(self.conn.last_insert_rowid())),
        Err(rusqlite::Error::SqliteFailure(err, _))
            if err.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            // 重複は「エラー」ではなく「挿入なし」として扱う
            Ok(None)
        }
        Err(e) => Err(e.into()),  // その他のエラーは伝播
    }
}
```

**ポイント**:
- `Result<Option<T>>` で「操作自体の失敗」と「結果がない」を区別
- 特定のエラー（重複）を正常系として処理
- それ以外のエラーは上位に伝播

---

### 理解度確認 5

**問題**: rustfeedの `fetch_feeds` 関数を見て、以下の質問に答えてください。

1. なぜ `feed::fetch_feed()` の呼び出しで `?` を使わずに `match` を使っているのでしょうか？
2. この設計の利点と欠点は何でしょうか？

<details>
<summary>回答を見る</summary>

**1. `match` を使う理由**:

`?` を使うと、1つのフィードで失敗した時点で関数全体が終了してしまいます。`match` を使うことで：
- 各フィードのエラーを個別に処理
- 1つが失敗しても他のフィードの処理を継続
- ユーザーにエラーを通知しつつ、可能な限り処理を進める

**2. 利点と欠点**:

**利点**:
- **レジリエンス（回復力）**: 一時的なネットワーク障害で1つのフィードが失敗しても、他は正常に処理される
- **ユーザー体験**: 部分的な成功でも価値がある（10個中9個成功など）
- **可視性**: どのフィードが失敗したか明確に表示できる

**欠点**:
- **エラーの集約が難しい**: どれだけ失敗したかのサマリーを返すには追加のロジックが必要
- **コードが複雑になる**: `?` より `match` の方がボイラープレートが多い
- **部分的失敗の意味**: 関数は `Ok(())` を返すが、内部で一部失敗している可能性がある

改善案として、失敗した数や詳細を戻り値に含めることもできます：

```rust
pub async fn fetch_feeds(db: &Database) -> Result<FetchSummary> {
    // ...
    Ok(FetchSummary {
        success_count,
        failure_count,
        new_articles: total_new,
    })
}
```

</details>

---

## まとめ

| 概念 | Rust | C++ | TypeScript |
|------|------|-----|------------|
| エラー表現 | `Result<T, E>` | 例外 / `std::expected` | 例外 / `null` |
| null安全 | `Option<T>` | `std::optional` | `T \| null` |
| エラー伝播 | `?` 演算子 | 自動伝播 / 手動 | `throw` / 手動 |
| 網羅性検査 | コンパイル時 | なし | なし（型ガードで部分的） |
| エラーチェーン | `anyhow` / `thiserror` | 非標準 | `Error.cause` (ES2022) |

次は [03-traits-generics.md](./03-traits-generics.md) でトレイトとジェネリクスについて学びます。
