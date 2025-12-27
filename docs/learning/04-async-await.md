# 04. 非同期プログラミング

Rustの非同期プログラミングは、`async`/`await` 構文とランタイム（主にtokio）を組み合わせて実現します。TypeScriptのPromiseベースの非同期処理と似ていますが、ゼロコスト抽象化を目指した独自の設計があります。

## 目次

1. [async/awaitの基本](#1-asyncawaitの基本)
2. [Futureトレイト](#2-futureトレイト)
3. [tokioランタイム](#3-tokioランタイム)
4. [エラー処理と非同期](#4-エラー処理と非同期)
5. [rustfeedでの実例](#5-rustfeedでの実例)

---

## 1. async/awaitの基本

### 非同期関数の定義

```rust
// async fn は Future を返す関数
async fn fetch_data(url: &str) -> String {
    // ネットワーク処理（実際にはreqwestなどを使用）
    "data".to_string()
}

// 呼び出し側では .await で待機
async fn process() {
    let data = fetch_data("https://example.com").await;
    println!("{}", data);
}
```

### TypeScriptとの比較

**TypeScript** の非同期処理：

```typescript
// async関数はPromiseを返す
async function fetchData(url: string): Promise<string> {
    const response = await fetch(url);
    return response.text();
}

// 呼び出し
async function process(): Promise<void> {
    const data = await fetchData("https://example.com");
    console.log(data);
}
```

**Rust**：

```rust
// async関数はFutureを返す
async fn fetch_data(url: &str) -> Result<String, reqwest::Error> {
    let response = reqwest::get(url).await?;
    response.text().await
}

// 呼び出し
async fn process() -> Result<(), reqwest::Error> {
    let data = fetch_data("https://example.com").await?;
    println!("{}", data);
    Ok(())
}
```

**類似点**:
- `async`/`await` キーワード
- 非同期関数は特別な型を返す（Promise / Future）

**相違点**:

| 特徴 | TypeScript | Rust |
|------|------------|------|
| 返り値の型 | `Promise<T>` | `impl Future<Output = T>` |
| ランタイム | V8/Node.js に組み込み | 外部クレート（tokio等） |
| 実行モデル | イベントループ | ポーリングベース |
| ゼロコスト | N/A（JIT最適化） | コンパイル時に最適化 |

---

### 理解度確認 1

**問題**: 以下のTypeScriptコードをRustに変換してください。

```typescript
async function delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
}

async function main(): Promise<void> {
    console.log("Start");
    await delay(1000);
    console.log("End");
}
```

<details>
<summary>回答を見る</summary>

```rust
use std::time::Duration;
use tokio::time::sleep;

async fn delay(duration: Duration) {
    sleep(duration).await;
}

#[tokio::main]
async fn main() {
    println!("Start");
    delay(Duration::from_millis(1000)).await;
    println!("End");
}
```

または、直接 `tokio::time::sleep` を使う場合：

```rust
use std::time::Duration;

#[tokio::main]
async fn main() {
    println!("Start");
    tokio::time::sleep(Duration::from_millis(1000)).await;
    println!("End");
}
```

**注意点**:
- Rustには組み込みのasyncランタイムがないため、`#[tokio::main]` マクロでtokioランタイムを起動
- 時間は `Duration` 型で表現（TypeScriptのミリ秒数値とは異なる）

</details>

---

## 2. Futureトレイト

### Futureとは

`Future` は非同期計算を表すトレイトです：

```rust
pub trait Future {
    type Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

pub enum Poll<T> {
    Ready(T),   // 計算完了
    Pending,    // まだ完了していない
}
```

### async/awaitの内部動作

`async fn` はコンパイル時に状態機械に変換されます：

```rust
// このコード
async fn example() -> i32 {
    let a = async_op_1().await;
    let b = async_op_2(a).await;
    a + b
}

// 概念的にはこのような状態機械に変換される
enum ExampleFuture {
    State0,                        // 初期状態
    State1 { a: i32 },             // async_op_1完了後
    State2 { a: i32, b: i32 },     // async_op_2完了後
}
```

### TypeScriptのPromiseとの違い

**TypeScript** のPromise：
- 作成時点で実行が開始される（eager）
- V8エンジンが管理

```typescript
// Promiseは作成した瞬間に実行開始
const promise = fetchData();  // すでに実行開始
// ...他の処理...
await promise;  // 結果を待つ
```

**Rust** のFuture：
- `await` されるまで何も実行されない（lazy）
- ランタイムが明示的にポーリング

```rust
// Futureを作成しただけでは何も起きない
let future = fetch_data();  // まだ実行されていない
// ...他の処理...
future.await;  // ここで初めて実行開始
```

---

### 理解度確認 2

**問題**: 以下のRustコードで、`"Fetching..."` は何回出力されるでしょうか？

```rust
async fn fetch() -> String {
    println!("Fetching...");
    "data".to_string()
}

#[tokio::main]
async fn main() {
    let future = fetch();  // (1)
    println!("Created future");
    let result = future.await;  // (2)
}
```

<details>
<summary>回答を見る</summary>

**1回だけ出力されます。**

出力順序：
```
Created future
Fetching...
```

**解説**:
- `(1)` の時点では `Future` が作成されるだけで、中身は実行されない
- `(2)` で `.await` した時点で初めて `fetch()` の本体が実行される

これがRustのFutureが「lazy（遅延評価）」である理由です。

対照的に、TypeScriptでは：
```typescript
const promise = fetch();  // この時点で実行開始
console.log("Created promise");  // Fetchingの後に出力される可能性あり
await promise;
```

</details>

---

## 3. tokioランタイム

### ランタイムとは

Rustには組み込みの非同期ランタイムがありません。`tokio` は最も広く使われるランタイムです：

```rust
// Cargo.toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

### main関数でasyncを使う

```rust
// 方法1: #[tokio::main] マクロ
#[tokio::main]
async fn main() {
    // 非同期コードを直接書ける
    let result = async_operation().await;
}

// 方法2: 手動でランタイムを作成
fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let result = async_operation().await;
    });
}
```

### 並行実行

```rust
use tokio::join;

async fn fetch_multiple() -> (String, String) {
    // 2つの非同期操作を並行実行
    let (a, b) = join!(
        fetch_data("url1"),
        fetch_data("url2"),
    );
    (a, b)
}
```

### TypeScriptとの比較

**TypeScript** の `Promise.all`：

```typescript
async function fetchMultiple(): Promise<[string, string]> {
    const [a, b] = await Promise.all([
        fetchData("url1"),
        fetchData("url2"),
    ]);
    return [a, b];
}
```

**Rust** の `join!`：

```rust
async fn fetch_multiple() -> (String, String) {
    let (a, b) = tokio::join!(
        fetch_data("url1"),
        fetch_data("url2"),
    );
    (a, b)
}
```

### 他のtokio機能

```rust
// タイムアウト
use tokio::time::{timeout, Duration};

async fn with_timeout() -> Result<String, tokio::time::error::Elapsed> {
    timeout(
        Duration::from_secs(5),
        slow_operation()
    ).await
}

// 選択（最初に完了したものを取得）
use tokio::select;

async fn race() -> String {
    select! {
        result = fast_operation() => result,
        result = slow_operation() => result,
    }
}
```

---

### 理解度確認 3

**問題**: 以下のTypeScriptコードをRustに変換してください。

```typescript
async function fetchAll(urls: string[]): Promise<string[]> {
    return Promise.all(urls.map(url => fetchData(url)));
}
```

<details>
<summary>回答を見る</summary>

```rust
use futures::future::join_all;

async fn fetch_all(urls: Vec<&str>) -> Vec<String> {
    let futures: Vec<_> = urls.iter()
        .map(|url| fetch_data(url))
        .collect();

    join_all(futures).await
}
```

または、`tokio::task::JoinSet` を使う方法：

```rust
use tokio::task::JoinSet;

async fn fetch_all(urls: Vec<String>) -> Vec<String> {
    let mut set = JoinSet::new();

    for url in urls {
        set.spawn(async move {
            fetch_data(&url).await
        });
    }

    let mut results = Vec::new();
    while let Some(result) = set.join_next().await {
        if let Ok(data) = result {
            results.push(data);
        }
    }

    results
}
```

**注意点**:
- `join_all` は `futures` クレートから提供
- `tokio::join!` はマクロなので、動的な数のFutureには使えない
- エラー処理を考慮すると `Result<Vec<String>, Error>` を返す方が良い

</details>

---

## 4. エラー処理と非同期

### ?演算子との組み合わせ

非同期関数でも `?` は通常通り使えます：

```rust
async fn fetch_and_parse(url: &str) -> Result<Data, Box<dyn std::error::Error>> {
    let response = reqwest::get(url).await?;  // ネットワークエラー
    let text = response.text().await?;         // 読み取りエラー
    let data: Data = serde_json::from_str(&text)?;  // パースエラー
    Ok(data)
}
```

### TypeScriptのtry/catchとの比較

**TypeScript**：

```typescript
async function fetchAndParse(url: string): Promise<Data> {
    try {
        const response = await fetch(url);
        const text = await response.text();
        return JSON.parse(text) as Data;
    } catch (error) {
        throw new Error(`Failed to fetch: ${error}`);
    }
}
```

**Rust**：

```rust
async fn fetch_and_parse(url: &str) -> Result<Data, anyhow::Error> {
    let response = reqwest::get(url)
        .await
        .context("Failed to fetch")?;
    let text = response.text()
        .await
        .context("Failed to read response")?;
    let data: Data = serde_json::from_str(&text)
        .context("Failed to parse JSON")?;
    Ok(data)
}
```

Rustの方がエラーチェーンが明確で、各ステップのエラーを追跡しやすいです。

---

### 理解度確認 4

**問題**: 以下のコードで、3つのURLのうち1つが失敗した場合、他の2つの結果はどうなりますか？

```rust
async fn fetch_multiple() -> Result<Vec<String>, reqwest::Error> {
    let urls = vec!["url1", "url2", "url3"];
    let mut results = Vec::new();

    for url in urls {
        let data = reqwest::get(url).await?.text().await?;
        results.push(data);
    }

    Ok(results)
}
```

<details>
<summary>回答を見る</summary>

**最初のエラーで関数全体が終了し、残りのURLは処理されません。**

これは `?` が `Err` を見つけると即座に `return` するためです。

各URLを独立して処理したい場合：

```rust
async fn fetch_multiple() -> Vec<Result<String, reqwest::Error>> {
    let urls = vec!["url1", "url2", "url3"];
    let mut results = Vec::new();

    for url in urls {
        let result = async {
            let data = reqwest::get(url).await?.text().await?;
            Ok::<_, reqwest::Error>(data)
        }.await;
        results.push(result);
    }

    results
}
```

または、成功したものだけを集める：

```rust
async fn fetch_multiple_resilient() -> Vec<String> {
    let urls = vec!["url1", "url2", "url3"];
    let futures: Vec<_> = urls.iter().map(|url| fetch_data(url)).collect();
    let results = join_all(futures).await;

    results.into_iter()
        .filter_map(|r| r.ok())
        .collect()
}
```

</details>

---

## 5. rustfeedでの実例

### 例1: fetch_feed関数

[crates/rustfeed-core/src/feed.rs](../../crates/rustfeed-core/src/feed.rs):

```rust
pub async fn fetch_feed(url: &str) -> Result<(Feed, Vec<Article>)> {
    // HTTPリクエスト（非同期I/O）
    let response = reqwest::get(url)
        .await
        .with_context(|| format!("Failed to fetch feed from {}", url))?;

    // レスポンスボディの読み取り（非同期I/O）
    let bytes = response
        .bytes()
        .await
        .with_context(|| "Failed to read response body")?;

    // フィードのパース（同期処理だがCPUバウンド）
    let parsed = parser::parse(&bytes[..])
        .with_context(|| "Failed to parse feed")?;

    // ... 残りの処理 ...

    Ok((feed, articles))
}
```

**ポイント**:
- `reqwest::get()` は非同期HTTPクライアント
- 各 `.await` ポイントで他のタスクに実行権を譲る
- パース処理は同期的だが、I/O待ち中に他のタスクが実行可能

### 例2: CLIのmain関数

[crates/rustfeed-cli/src/main.rs](../../crates/rustfeed-cli/src/main.rs):

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let db = Database::new()?;
    db.init()?;

    match cli.command {
        Commands::Add { url, name } => {
            commands::add_feed(&db, &url, name.as_deref()).await?;
        }
        Commands::Fetch => {
            commands::fetch_feeds(&db).await?;
        }
        // ...同期的なコマンドも混在...
        Commands::List => {
            commands::list_feeds(&db)?;  // 非同期ではない
        }
    }

    Ok(())
}
```

**ポイント**:
- `#[tokio::main]` でasync mainを有効化
- 非同期コマンド（`fetch`）と同期コマンド（`list`）が混在
- データベース操作は同期（rusqliteは同期API）

### 例3: 複数フィードの逐次取得

[crates/rustfeed-cli/src/commands.rs](../../crates/rustfeed-cli/src/commands.rs):

```rust
pub async fn fetch_feeds(db: &Database) -> Result<()> {
    let feeds = db.get_feeds(None)?;

    for stored_feed in feeds {
        // 各フィードを順番に取得（逐次処理）
        match feed::fetch_feed(&stored_feed.url).await {
            Ok((_feed_info, articles)) => {
                // 記事を保存
            }
            Err(e) => {
                println!("{} ({})", "ERROR".red(), e);
            }
        }
    }

    Ok(())
}
```

**現状の設計**:
- フィードを1つずつ順番に取得（逐次処理）
- 1つのエラーが他に影響しない

**改善の可能性**（並行取得）:

```rust
use futures::future::join_all;

pub async fn fetch_feeds_concurrent(db: &Database) -> Result<()> {
    let feeds = db.get_feeds(None)?;

    // すべてのフィードを並行して取得
    let futures: Vec<_> = feeds.iter()
        .map(|feed| async move {
            let result = feed::fetch_feed(&feed.url).await;
            (feed.id, result)
        })
        .collect();

    let results = join_all(futures).await;

    for (feed_id, result) in results {
        match result {
            Ok((_, articles)) => {
                // 記事を保存（データベース操作は順次）
                for mut article in articles {
                    article.feed_id = feed_id;
                    db.add_article(&article)?;
                }
            }
            Err(e) => println!("Error: {}", e),
        }
    }

    Ok(())
}
```

### 例4: TUIのイベントループ

[crates/rustfeed-tui/src/app.rs](../../crates/rustfeed-tui/src/app.rs):

```rust
pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
    loop {
        // 画面を描画（同期）
        terminal.draw(|frame| ui::render(frame, self))?;

        // イベントをポーリング（タイムアウト付き）
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // キーイベントを処理
            }
        }

        if self.should_quit {
            break;
        }
    }
    Ok(())
}
```

**ポイント**:
- TUIはイベントループパターン
- `event::poll()` でタイムアウト付きで待機
- 非同期だが、主にUI更新とイベント処理

---

### 理解度確認 5

**問題**: rustfeedの `fetch_feeds` 関数を見て、以下の質問に答えてください。

1. なぜ現在の実装は並行ではなく逐次処理なのでしょうか？考えられる理由は？
2. 並行処理にした場合のメリット・デメリットは？

<details>
<summary>回答を見る</summary>

**1. 逐次処理の理由**:

- **シンプルさ**: 実装が単純で理解しやすい
- **リソース制御**: 同時接続数を制限しなくて済む
- **デバッグ容易性**: 問題発生時にどのフィードか特定しやすい
- **サーバー負荷**: 同じサーバーへの同時リクエストを避ける
- **進捗表示**: 現在どのフィードを処理中か表示できる

**2. 並行処理のメリット・デメリット**:

**メリット**:
- 全体の処理時間が短縮される（特にフィード数が多い場合）
- ネットワークレイテンシを隠蔽できる
- モダンなUXを提供できる

**デメリット**:
- 同時接続数の管理が必要（Semaphoreなど）
- エラー処理が複雑になる
- 進捗表示が難しくなる
- 同じサーバーへの過度なリクエストのリスク
- データベースへの書き込みは結局順次になる

**中間的なアプローチ**:
```rust
// 並行度を制限した並行処理
use futures::stream::{self, StreamExt};

let results = stream::iter(feeds)
    .map(|feed| fetch_feed(&feed.url))
    .buffer_unordered(3)  // 最大3並行
    .collect::<Vec<_>>()
    .await;
```

</details>

---

## まとめ

| 概念 | Rust | TypeScript |
|------|------|------------|
| 非同期の抽象化 | `Future` トレイト | `Promise` |
| 構文 | `async`/`await` | `async`/`await` |
| 実行モデル | lazy（遅延） | eager（即時） |
| ランタイム | 外部（tokio等） | 組み込み（V8） |
| 並行実行 | `join!`, `join_all` | `Promise.all` |
| 競争 | `select!` | `Promise.race` |
| エラー処理 | `Result` + `?` | `try`/`catch` |

次は [05-modules-crates.md](./05-modules-crates.md) でモジュールとクレートについて学びます。
