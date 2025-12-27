# 06. パターンマッチング

パターンマッチングはRustの最も強力な機能の一つです。値の構造を分解し、条件分岐を型安全に行うことができます。C++のswitch文やTypeScriptの分割代入と比較して、より表現力豊かで安全です。

## 目次

1. [match式の基本](#1-match式の基本)
2. [パターンの種類](#2-パターンの種類)
3. [if letとwhile let](#3-if-letとwhile-let)
4. [網羅性チェック](#4-網羅性チェック)
5. [rustfeedでの実例](#5-rustfeedでの実例)

---

## 1. match式の基本

### match式の構文

```rust
let number = 13;

let description = match number {
    1 => "one",
    2 => "two",
    3..=12 => "few",      // 範囲パターン
    13 | 14 => "teen",    // 複数パターン
    _ => "many",          // ワイルドカード（その他すべて）
};
```

### C++との比較

**C++** の switch 文：

```cpp
int number = 13;
std::string description;

switch (number) {
    case 1:
        description = "one";
        break;
    case 2:
        description = "two";
        break;
    case 13:
    case 14:
        description = "teen";
        break;
    default:
        if (number >= 3 && number <= 12) {
            description = "few";
        } else {
            description = "many";
        }
        break;
}
```

**違い**:

| 特徴 | Rust match | C++ switch |
|------|------------|------------|
| フォールスルー | なし | `break` 忘れで発生 |
| 範囲パターン | `3..=12` | 不可（if文で代用） |
| 式として使用 | 可能 | 不可 |
| 網羅性チェック | コンパイル時 | なし |
| 型安全 | 完全 | 整数型のみ |

### TypeScriptとの比較

**TypeScript** のswitch文とオブジェクトマッピング：

```typescript
const number = 13;

// switch文
let description: string;
switch (number) {
    case 1:
        description = "one";
        break;
    case 2:
        description = "two";
        break;
    default:
        description = "many";
}

// オブジェクトマッピング（パターンマッチの代替）
const descMap: Record<number, string> = {
    1: "one",
    2: "two",
};
const desc = descMap[number] ?? "many";
```

**Rust** は値を直接返せます：

```rust
let description = match number {
    1 => "one",
    2 => "two",
    _ => "many",
};
```

---

### 理解度確認 1

**問題**: 以下のC++ switch文をRustのmatch式に変換してください。

```cpp
int grade = 85;
char letter;

switch (grade / 10) {
    case 10:
    case 9:
        letter = 'A';
        break;
    case 8:
        letter = 'B';
        break;
    case 7:
        letter = 'C';
        break;
    case 6:
        letter = 'D';
        break;
    default:
        letter = 'F';
        break;
}
```

<details>
<summary>回答を見る</summary>

```rust
let grade = 85;

let letter = match grade / 10 {
    10 | 9 => 'A',
    8 => 'B',
    7 => 'C',
    6 => 'D',
    _ => 'F',
};
```

または、範囲パターンを使う場合：

```rust
let letter = match grade {
    90..=100 => 'A',
    80..=89 => 'B',
    70..=79 => 'C',
    60..=69 => 'D',
    _ => 'F',
};
```

Rustの方が：
- `break` が不要
- 式として値を返せる
- 範囲パターンでより直感的に書ける

</details>

---

## 2. パターンの種類

### 構造体の分解

```rust
struct Point {
    x: i32,
    y: i32,
}

let point = Point { x: 0, y: 7 };

match point {
    Point { x: 0, y } => println!("On y-axis at {}", y),
    Point { x, y: 0 } => println!("On x-axis at {}", x),
    Point { x, y } => println!("At ({}, {})", x, y),
}
```

### 列挙型の分解

```rust
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(i32, i32, i32),
}

let msg = Message::ChangeColor(255, 0, 128);

match msg {
    Message::Quit => println!("Quit"),
    Message::Move { x, y } => println!("Move to ({}, {})", x, y),
    Message::Write(text) => println!("Write: {}", text),
    Message::ChangeColor(r, g, b) => println!("Color: rgb({}, {}, {})", r, g, b),
}
```

### ガード条件

```rust
let num = Some(4);

match num {
    Some(x) if x < 5 => println!("Less than 5: {}", x),
    Some(x) => println!("Greater or equal: {}", x),
    None => println!("No value"),
}
```

### TypeScriptの分割代入との比較

**TypeScript**:

```typescript
interface Point {
    x: number;
    y: number;
}

const point: Point = { x: 0, y: 7 };

// 分割代入
const { x, y } = point;

// 条件分岐は別途必要
if (x === 0) {
    console.log(`On y-axis at ${y}`);
} else if (y === 0) {
    console.log(`On x-axis at ${x}`);
} else {
    console.log(`At (${x}, ${y})`);
}
```

**Rust** のmatchは分解と条件分岐を一体化：

```rust
match point {
    Point { x: 0, y } => println!("On y-axis at {}", y),
    Point { x, y: 0 } => println!("On x-axis at {}", x),
    Point { x, y } => println!("At ({}, {})", x, y),
}
```

---

### 理解度確認 2

**問題**: 以下の `Option<Result<i32, String>>` 型の値をmatchで処理するコードを書いてください。

- `Some(Ok(n))` の場合: 数値を2倍して表示
- `Some(Err(msg))` の場合: エラーメッセージを表示
- `None` の場合: "No value" と表示

```rust
let value: Option<Result<i32, String>> = Some(Ok(21));
// ここにmatch式を書く
```

<details>
<summary>回答を見る</summary>

```rust
let value: Option<Result<i32, String>> = Some(Ok(21));

match value {
    Some(Ok(n)) => println!("Doubled: {}", n * 2),
    Some(Err(msg)) => println!("Error: {}", msg),
    None => println!("No value"),
}
```

ネストしたパターンも自然に書けます。これをTypeScriptで書くと：

```typescript
const value: { ok: number } | { err: string } | null = { ok: 21 };

if (value === null) {
    console.log("No value");
} else if ("ok" in value) {
    console.log(`Doubled: ${value.ok * 2}`);
} else {
    console.log(`Error: ${value.err}`);
}
```

Rustの方がパターンの意図が明確です。

</details>

---

## 3. if letとwhile let

### if let

`match` の1パターンだけ処理したい場合の省略形：

```rust
let some_value = Some(42);

// matchを使う場合
match some_value {
    Some(x) => println!("Got: {}", x),
    _ => {}  // 何もしない
}

// if letを使う場合（より簡潔）
if let Some(x) = some_value {
    println!("Got: {}", x);
}

// else も使える
if let Some(x) = some_value {
    println!("Got: {}", x);
} else {
    println!("No value");
}
```

### while let

条件が満たされる間ループ：

```rust
let mut stack = vec![1, 2, 3];

while let Some(top) = stack.pop() {
    println!("{}", top);
}
// 出力: 3, 2, 1
```

### let else (Rust 1.65+)

パターンにマッチしない場合に早期リターン：

```rust
fn process(value: Option<i32>) -> i32 {
    let Some(x) = value else {
        return 0;  // Noneの場合は早期リターン
    };

    x * 2  // Someの中身を使って処理
}
```

### TypeScriptとの比較

**TypeScript** のOptional chaining と Nullish coalescing：

```typescript
const value: number | null = 42;

// Optional chaining
const doubled = value != null ? value * 2 : undefined;

// Nullish coalescing
const result = value ?? 0;
```

**Rust** の `if let` と `let else`：

```rust
let value: Option<i32> = Some(42);

// if let
if let Some(x) = value {
    println!("Doubled: {}", x * 2);
}

// let else（ガード的な使い方）
fn process(value: Option<i32>) -> i32 {
    let Some(x) = value else { return 0 };
    x * 2
}
```

---

### 理解度確認 3

**問題**: 以下の `match` 式を `if let` を使って書き直してください。

```rust
let config: Option<String> = get_config();

match config {
    Some(value) => {
        println!("Config: {}", value);
        use_config(&value);
    }
    None => {}
}
```

<details>
<summary>回答を見る</summary>

```rust
let config: Option<String> = get_config();

if let Some(value) = config {
    println!("Config: {}", value);
    use_config(&value);
}
```

さらに、エラーを返す関数内では `let else` を使えます：

```rust
fn process_config() -> Result<(), Error> {
    let Some(config) = get_config() else {
        return Err(Error::NoConfig);
    };

    println!("Config: {}", config);
    use_config(&config);
    Ok(())
}
```

`if let` を使うべき場合：
- 1つのパターンだけ処理したい
- `_` アームが空または単純

`match` を使うべき場合：
- 複数のパターンを処理
- 網羅性を明示したい

</details>

---

## 4. 網羅性チェック

### コンパイル時の網羅性検査

Rustのmatchは、すべてのパターンを網羅しないとコンパイルエラー：

```rust
enum Direction {
    North,
    South,
    East,
    West,
}

fn move_player(dir: Direction) {
    match dir {
        Direction::North => println!("Moving north"),
        Direction::South => println!("Moving south"),
        // コンパイルエラー: East と West が未処理
    }
}
```

### 新しいバリアントの追加

列挙型に新しいバリアントを追加すると、関連するmatch式すべてでエラーになります：

```rust
enum Direction {
    North,
    South,
    East,
    West,
    Up,      // 新しく追加
    Down,    // 新しく追加
}

// 既存のmatch式がコンパイルエラーに
// → すべての箇所で対応を強制される
```

### C++/TypeScriptとの比較

**C++** の switch：

```cpp
enum Direction { North, South, East, West };

void move_player(Direction dir) {
    switch (dir) {
        case North: std::cout << "Moving north" << std::endl; break;
        case South: std::cout << "Moving south" << std::endl; break;
        // East, West が未処理でも警告のみ（コンパイルは通る）
    }
}
```

**TypeScript** の網羅性チェック（手動で設定が必要）：

```typescript
type Direction = "North" | "South" | "East" | "West";

function movePlayer(dir: Direction): void {
    switch (dir) {
        case "North":
            console.log("Moving north");
            break;
        case "South":
            console.log("Moving south");
            break;
        // defaultがないと、tsconfigでnoImplicitReturnsなら警告
    }
}

// 網羅性を強制するパターン
function assertNever(x: never): never {
    throw new Error("Unexpected value: " + x);
}

function movePlayerExhaustive(dir: Direction): void {
    switch (dir) {
        case "North":
            console.log("Moving north");
            break;
        case "South":
            console.log("Moving south");
            break;
        case "East":
            console.log("Moving east");
            break;
        case "West":
            console.log("Moving west");
            break;
        default:
            assertNever(dir);  // 未処理があればコンパイルエラー
    }
}
```

**Rust** では網羅性チェックが標準で強制されます。

---

### 理解度確認 4

**問題**: 以下のコードで、コンパイルエラーになる理由と、2つの修正方法を示してください。

```rust
enum Status {
    Active,
    Inactive,
    Pending,
}

fn describe(status: Status) -> &'static str {
    match status {
        Status::Active => "Running",
        Status::Inactive => "Stopped",
    }
}
```

<details>
<summary>回答を見る</summary>

**エラーの理由**: `Status::Pending` が処理されていないため、パターンが網羅的ではありません。

**修正方法1**: 明示的に `Pending` を処理する

```rust
fn describe(status: Status) -> &'static str {
    match status {
        Status::Active => "Running",
        Status::Inactive => "Stopped",
        Status::Pending => "Waiting",
    }
}
```

**修正方法2**: ワイルドカードを使う

```rust
fn describe(status: Status) -> &'static str {
    match status {
        Status::Active => "Running",
        Status::Inactive => "Stopped",
        _ => "Unknown",  // Pendingを含む残りすべて
    }
}
```

**どちらを選ぶべきか？**

- **明示的に列挙**: 新しいバリアント追加時にコンパイルエラーで気づける（推奨）
- **ワイルドカード**: 意図的に「それ以外」を同じ扱いにする場合

`#[non_exhaustive]` 属性が付いた列挙型（外部クレートのものなど）では、ワイルドカードが必須になることがあります。

</details>

---

## 5. rustfeedでの実例

### 例1: コマンドライン引数の処理

[crates/rustfeed-cli/src/main.rs](../../crates/rustfeed-cli/src/main.rs):

```rust
#[derive(Subcommand)]
enum Commands {
    Add { url: String, name: Option<String> },
    Remove { id: i64 },
    List { category: Option<String> },
    Fetch,
    Articles { unread: bool, limit: Option<usize>, /* ... */ },
    // ... その他のコマンド
}

// main関数での処理
match cli.command {
    Commands::Add { url, name } => {
        commands::add_feed(&db, &url, name.as_deref()).await?;
    }
    Commands::Remove { id } => {
        commands::remove_feed(&db, id)?;
    }
    Commands::List { category } => {
        commands::list_feeds(&db, category.as_deref())?;
    }
    Commands::Fetch => {
        commands::fetch_feeds(&db).await?;
    }
    // ... 残りのコマンド
}
```

**ポイント**:
- `clap` の `Subcommand` derive で列挙型を定義
- 各バリアントのフィールドをパターンで分解
- 新しいコマンドを追加すると、matchで対応が必要になる

### 例2: エクスポート形式の処理

[crates/rustfeed-cli/src/commands.rs](../../crates/rustfeed-cli/src/commands.rs):

```rust
match format.to_lowercase().as_str() {
    "json" => export_as_json(&articles)?,
    "markdown" | "md" => export_as_markdown(&articles)?,
    _ => {
        anyhow::bail!(
            "Unsupported format: '{}'. Use 'json' or 'markdown'.",
            format
        );
    }
}
```

**ポイント**:
- 文字列のパターンマッチ
- `|` で複数パターン（`"markdown"` または `"md"`）
- `_` でサポートされていない形式をエラー処理

### 例3: TUIのキー入力処理

[crates/rustfeed-tui/src/app.rs](../../crates/rustfeed-tui/src/app.rs):

```rust
match key.code {
    KeyCode::Char('q') => {
        self.should_quit = true;
    }
    KeyCode::Char('j') | KeyCode::Down => {
        self.move_down();
    }
    KeyCode::Char('k') | KeyCode::Up => {
        self.move_up();
    }
    KeyCode::Tab => {
        self.focus = match self.focus {
            Focus::Feeds => Focus::Articles,
            Focus::Articles => Focus::Feeds,
        };
    }
    KeyCode::Enter => {
        self.open_selected()?;
    }
    _ => {}
}
```

**ポイント**:
- 複数のキーを同じ処理にマップ（`'j'` と `Down`）
- ネストしたmatch（`Tab` キーでフォーカス切り替え）
- `_` で未処理のキーを無視

### 例4: Resultのパターンマッチ

[crates/rustfeed-cli/src/commands.rs](../../crates/rustfeed-cli/src/commands.rs):

```rust
match feed::fetch_feed(&stored_feed.url).await {
    Ok((_feed_info, articles)) => {
        let mut new_count = 0;

        for mut article in articles {
            article.feed_id = stored_feed.id;
            if db.add_article(&article)?.is_some() {
                new_count += 1;
            }
        }

        println!("{} ({} new)", "OK".green(), new_count.to_string().cyan());
    }
    Err(e) => {
        println!("{} ({})", "ERROR".red(), e);
    }
}
```

**ポイント**:
- `Result` を `Ok` と `Err` でパターンマッチ
- `Ok` 内のタプルをさらに分解（`(_feed_info, articles)`）
- 成功/失敗で異なる処理を実行

### 例5: フォーカス状態の切り替え

```rust
self.focus = match self.focus {
    Focus::Feeds => Focus::Articles,
    Focus::Articles => Focus::Feeds,
};
```

**ポイント**:
- matchを式として使用
- 状態を「反転」させる簡潔なパターン
- 列挙型のバリアントが増えてもコンパイラが検出

---

### 理解度確認 5

**問題**: rustfeedのコードを参考に、以下の処理をパターンマッチで書いてください。

HTTPレスポンスのステータスコードに応じて処理を分岐：
- 200-299: 成功として処理
- 400-499: クライアントエラーとして処理
- 500-599: サーバーエラーとして処理
- その他: 不明なステータスとして処理

```rust
fn handle_response(status: u16) -> Result<(), AppError> {
    // ここにmatch式を書く
}
```

<details>
<summary>回答を見る</summary>

```rust
fn handle_response(status: u16) -> Result<(), AppError> {
    match status {
        200..=299 => {
            println!("Success!");
            Ok(())
        }
        400..=499 => {
            Err(AppError::ClientError(format!("Client error: {}", status)))
        }
        500..=599 => {
            Err(AppError::ServerError(format!("Server error: {}", status)))
        }
        _ => {
            Err(AppError::Unknown(format!("Unknown status: {}", status)))
        }
    }
}
```

または、より詳細なハンドリング：

```rust
fn handle_response(status: u16) -> Result<String, AppError> {
    match status {
        200 => Ok("OK".to_string()),
        201 => Ok("Created".to_string()),
        204 => Ok("No Content".to_string()),
        301 | 302 => Err(AppError::Redirect("Resource moved".to_string())),
        400 => Err(AppError::BadRequest),
        401 | 403 => Err(AppError::Unauthorized),
        404 => Err(AppError::NotFound),
        429 => Err(AppError::RateLimited),
        500..=599 => Err(AppError::ServerError(status)),
        _ => Err(AppError::Unknown(status)),
    }
}
```

範囲パターン（`200..=299`）と複数パターン（`401 | 403`）を組み合わせることで、表現力の高い条件分岐が書けます。

</details>

---

## まとめ

| 概念 | Rust | C++ | TypeScript |
|------|------|-----|------------|
| 条件分岐 | `match` 式 | `switch` 文 | `switch` 文 |
| フォールスルー | なし | デフォルト | デフォルト |
| 式として使用 | 可能 | 不可 | 不可 |
| 構造体分解 | パターン内で可能 | 不可 | 限定的（分割代入） |
| 範囲パターン | `1..=10` | 不可 | 不可 |
| 網羅性チェック | コンパイル時強制 | 警告のみ | 手動で設定 |
| ガード条件 | `if` ガード | 不可 | 不可 |

次は [07-memory-safety.md](./07-memory-safety.md) でメモリ安全性について学びます。
