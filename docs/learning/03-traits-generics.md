# 03. トレイトとジェネリクス

トレイト（Trait）はRustにおける抽象化の中心的な仕組みです。C++のテンプレートや仮想関数、TypeScriptのインターフェースと似た機能を提供しますが、独自の設計思想を持っています。

## 目次

1. [トレイトの基本](#1-トレイトの基本)
2. [ジェネリクス](#2-ジェネリクス)
3. [トレイト境界](#3-トレイト境界)
4. [deriveマクロ](#4-deriveマクロ)
5. [rustfeedでの実例](#5-rustfeedでの実例)

---

## 1. トレイトの基本

### トレイトとは

トレイトは「共通の振る舞い」を定義する仕組みです：

```rust
// トレイトの定義
trait Printable {
    fn print(&self);

    // デフォルト実装も可能
    fn print_twice(&self) {
        self.print();
        self.print();
    }
}

// トレイトの実装
struct Article {
    title: String,
}

impl Printable for Article {
    fn print(&self) {
        println!("Article: {}", self.title);
    }
    // print_twice はデフォルト実装を使用
}
```

### C++との比較

**C++** の抽象クラス/インターフェース：

```cpp
// 純粋仮想関数を持つ抽象クラス
class Printable {
public:
    virtual void print() const = 0;

    // デフォルト実装
    virtual void print_twice() const {
        print();
        print();
    }

    virtual ~Printable() = default;
};

class Article : public Printable {
public:
    std::string title;

    void print() const override {
        std::cout << "Article: " << title << std::endl;
    }
};
```

**違い**:

| 特徴 | Rust トレイト | C++ 抽象クラス |
|------|--------------|---------------|
| 継承 | 構造体は継承なし | 継承ベース |
| vtable | 静的ディスパッチがデフォルト | 動的ディスパッチがデフォルト |
| 既存型への追加 | 可能（orphan ruleに従う） | 不可能 |
| 複数実装 | 複数トレイトを実装可 | 多重継承（複雑） |

### TypeScriptとの比較

**TypeScript** のインターフェース：

```typescript
interface Printable {
    print(): void;
}

// デフォルト実装は直接定義できない
// Mixinパターンなどで回避

class Article implements Printable {
    constructor(public title: string) {}

    print(): void {
        console.log(`Article: ${this.title}`);
    }
}
```

**Rust** では：

```rust
trait Printable {
    fn print(&self);

    // デフォルト実装が直接書ける
    fn print_with_prefix(&self, prefix: &str) {
        print!("{}: ", prefix);
        self.print();
    }
}
```

---

### 理解度確認 1

**問題**: 以下のTypeScriptインターフェースをRustのトレイトに変換してください。

```typescript
interface Displayable {
    getDisplayName(): string;
    getShortName(): string;  // デフォルトはgetDisplayNameの最初の10文字
}
```

<details>
<summary>回答を見る</summary>

```rust
trait Displayable {
    fn get_display_name(&self) -> String;

    // デフォルト実装
    fn get_short_name(&self) -> String {
        let name = self.get_display_name();
        if name.len() <= 10 {
            name
        } else {
            // UTF-8文字境界を考慮
            name.chars().take(10).collect()
        }
    }
}

// 実装例
struct Feed {
    title: String,
}

impl Displayable for Feed {
    fn get_display_name(&self) -> String {
        self.title.clone()
    }
    // get_short_name はデフォルト実装を使用
}
```

注意点：
- Rustでは命名規則が `snake_case`
- UTF-8の文字列は単純なスライスができないため `chars()` を使用
- `String` を返す場合は所有権の移動を考慮

</details>

---

## 2. ジェネリクス

### ジェネリック関数

型パラメータを使って汎用的な関数を作成できます：

```rust
// 任意の型Tを受け取る関数
fn print_vec<T: std::fmt::Debug>(items: &[T]) {
    for item in items {
        println!("{:?}", item);
    }
}

// 使用例
print_vec(&[1, 2, 3]);
print_vec(&["a", "b", "c"]);
```

### ジェネリック構造体

```rust
// ジェネリックな構造体
struct Container<T> {
    value: T,
}

impl<T> Container<T> {
    fn new(value: T) -> Self {
        Container { value }
    }

    fn get(&self) -> &T {
        &self.value
    }
}

// 特定の型に対する追加実装
impl Container<String> {
    fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}
```

### C++との比較

**C++ テンプレート**：

```cpp
template<typename T>
class Container {
    T value;
public:
    Container(T v) : value(std::move(v)) {}

    const T& get() const { return value; }
};

// 特殊化
template<>
class Container<std::string> {
    // 完全に別の実装を定義
};
```

**違い**:

| 特徴 | Rust ジェネリクス | C++ テンプレート |
|------|------------------|------------------|
| コンパイルタイミング | 使用時に単相化 | 使用時にインスタンス化 |
| エラータイミング | 定義時に検証 | 使用時に検証 |
| 制約の表現 | トレイト境界 | コンセプト (C++20) |

### TypeScriptとの比較

**TypeScript** のジェネリクス：

```typescript
class Container<T> {
    constructor(private value: T) {}

    get(): T {
        return this.value;
    }
}

// 使用時に型推論
const numContainer = new Container(42);      // Container<number>
const strContainer = new Container("hello"); // Container<string>
```

**Rust** も同様に型推論が働きます：

```rust
let num_container = Container::new(42);      // Container<i32>
let str_container = Container::new("hello"); // Container<&str>
```

---

### 理解度確認 2

**問題**: 以下のRustコードはコンパイルできるでしょうか？できない場合、なぜでしょうか？

```rust
fn largest<T>(list: &[T]) -> &T {
    let mut largest = &list[0];

    for item in list {
        if item > largest {
            largest = item;
        }
    }

    largest
}
```

<details>
<summary>回答を見る</summary>

**コンパイルエラーになります。**

`T` に対して `>` 演算子が使えるかどうかコンパイラは知りません。トレイト境界を追加する必要があります。

修正版：

```rust
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];

    for item in list {
        if item > largest {
            largest = item;
        }
    }

    largest
}
```

`PartialOrd` トレイトは比較演算子（`<`, `>`, `<=`, `>=`）を提供します。

C++では同様のコードがコンパイルを通り、実際に使った時点でエラーになります（SFINAE/コンセプト以前）：

```cpp
template<typename T>
const T& largest(const std::vector<T>& list) {
    const T* largest = &list[0];
    for (const auto& item : list) {
        if (item > *largest) {  // 使用時にエラー検出
            largest = &item;
        }
    }
    return *largest;
}
```

</details>

---

## 3. トレイト境界

### トレイト境界の書き方

ジェネリック型に制約を付ける方法：

```rust
// 方法1: 山括弧内で指定
fn print_debug<T: std::fmt::Debug>(value: T) {
    println!("{:?}", value);
}

// 方法2: where句で指定（複雑な場合に読みやすい）
fn process<T, U>(a: T, b: U) -> String
where
    T: std::fmt::Display + Clone,
    U: std::fmt::Debug,
{
    format!("{} {:?}", a.clone(), b)
}

// 方法3: impl Trait（引数の場合）
fn print_all(items: impl Iterator<Item = String>) {
    for item in items {
        println!("{}", item);
    }
}
```

### 複数のトレイト境界

```rust
// + で複数のトレイトを要求
fn debug_and_display<T: std::fmt::Debug + std::fmt::Display>(value: T) {
    println!("Debug: {:?}", value);
    println!("Display: {}", value);
}
```

### C++コンセプトとの比較

**C++20 コンセプト**：

```cpp
#include <concepts>

// コンセプトの定義
template<typename T>
concept Printable = requires(T t) {
    { std::cout << t } -> std::same_as<std::ostream&>;
};

// コンセプトの使用
template<Printable T>
void print(const T& value) {
    std::cout << value << std::endl;
}
```

**Rust** のトレイト境界：

```rust
use std::fmt::Display;

fn print<T: Display>(value: T) {
    println!("{}", value);
}
```

Rustの方がシンプルで、言語の最初から統合されています。

---

### 理解度確認 3

**問題**: 以下の関数シグネチャを `where` 句を使って書き直してください。

```rust
fn combine<T: Clone + Debug, U: Display + Into<String>>(a: T, b: U) -> String {
    // ...
}
```

<details>
<summary>回答を見る</summary>

```rust
use std::fmt::{Debug, Display};

fn combine<T, U>(a: T, b: U) -> String
where
    T: Clone + Debug,
    U: Display + Into<String>,
{
    // ...
}
```

`where` 句を使う利点：
1. **読みやすさ**: トレイト境界が複雑な場合に整理される
2. **柔軟性**: より複雑な制約（関連型など）を表現できる
3. **一貫性**: すべての制約が一箇所にまとまる

さらに複雑な例：

```rust
fn process<T, U, V>(a: T, b: U, c: V) -> V::Output
where
    T: Clone + Debug + Send,
    U: Display + Into<String> + 'static,
    V: std::ops::Add<Output = V>,
{
    // ...
}
```

</details>

---

## 4. deriveマクロ

### よく使うderiveマクロ

`#[derive(...)]` で標準トレイトを自動実装できます：

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
struct User {
    id: i64,
    name: String,
    email: String,
}
```

### 標準のderiveマクロ

| マクロ | 用途 | 必要条件 |
|--------|------|----------|
| `Debug` | デバッグ出力 `{:?}` | すべてのフィールドが `Debug` |
| `Clone` | `.clone()` で複製 | すべてのフィールドが `Clone` |
| `Copy` | 暗黙のコピー | すべてのフィールドが `Copy`、`Clone` も必要 |
| `PartialEq` | `==` で比較 | すべてのフィールドが `PartialEq` |
| `Eq` | 完全な等価性 | `PartialEq` + 反射性を保証 |
| `Hash` | ハッシュ計算 | すべてのフィールドが `Hash` |
| `Default` | デフォルト値 | すべてのフィールドが `Default` |
| `PartialOrd` | 部分順序 `<`, `>` | `PartialEq` + すべてのフィールドが `PartialOrd` |
| `Ord` | 全順序 | `PartialOrd` + `Eq` |

### serdeのderive

外部クレートもderiveを提供します：

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Config {
    name: String,
    port: u16,

    #[serde(default)]  // 属性でカスタマイズ
    debug: bool,

    #[serde(rename = "api_key")]
    api_key: String,
}
```

### C++との比較

**C++** では同様の機能を手動で書くか、マクロを使います：

```cpp
struct User {
    int id;
    std::string name;
    std::string email;

    // デフォルトコンストラクタ
    User() = default;

    // コピーコンストラクタ（自動生成される場合もある）
    User(const User&) = default;

    // 比較演算子（C++20の<=>で簡略化）
    auto operator<=>(const User&) const = default;

    // デバッグ出力（手動で書く必要がある）
    friend std::ostream& operator<<(std::ostream& os, const User& u) {
        return os << "User { id: " << u.id << ", name: " << u.name << " }";
    }
};
```

Rustの `#[derive]` はボイラープレートを大幅に削減します。

---

### 理解度確認 4

**問題**: 以下の構造体にどの derive マクロを付けるべきでしょうか？用途を考えて選んでください。

```rust
// 用途: HashMapのキーとして使用、JSONにシリアライズ、デバッグ出力
struct SessionKey {
    user_id: i64,
    token: String,
}
```

<details>
<summary>回答を見る</summary>

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct SessionKey {
    user_id: i64,
    token: String,
}
```

**必要なトレイト**:

1. **`Debug`**: デバッグ出力のため
2. **`Clone`**: 通常必要になる（HashMapから取り出す時など）
3. **`PartialEq` + `Eq`**: `Hash` の前提条件
4. **`Hash`**: HashMapのキーとして使用するため
5. **`Serialize` + `Deserialize`**: JSONシリアライズのため

**不要なトレイト**:
- `Copy`: `String` フィールドがあるため実装不可
- `Default`: セッションキーにデフォルト値は意味がない
- `PartialOrd`/`Ord`: 順序付けは不要

</details>

---

## 5. rustfeedでの実例

### 例1: Feed構造体のderive

[crates/rustfeed-core/src/models.rs](../../crates/rustfeed-core/src/models.rs):

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feed {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub description: Option<String>,
    // ...
}
```

**選択理由**:
- `Debug`: デバッグ出力、エラーメッセージ
- `Clone`: データベースから取得したデータを複製
- `Serialize`/`Deserialize`: JSONエクスポート機能

**選択しなかった理由**:
- `PartialEq`: フィード間の比較は必要ない
- `Hash`: HashMapのキーとしては使わない（IDで管理）

### 例2: 設定ファイルの構造体

[crates/rustfeed-core/src/config.rs](../../crates/rustfeed-core/src/config.rs):

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub general: GeneralConfig,

    #[serde(default)]
    pub display: DisplayConfig,

    #[serde(default)]
    pub database: DatabaseConfig,
}
```

**ポイント**:
- `Default`: 設定ファイルがない場合のデフォルト値
- `#[serde(default)]`: 個別フィールドが欠けていてもデフォルト値を使用

### 例3: 列挙型のderive

[crates/rustfeed-tui/src/app.rs](../../crates/rustfeed-tui/src/app.rs):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Feeds,
    Articles,
}
```

**選択理由**:
- `Copy`: 列挙型は小さいのでコピーが効率的
- `PartialEq`/`Eq`: `if focus == Focus::Feeds` のような比較

**列挙型は Copy を実装しやすい**:
```rust
// 列挙型はデータを持たなければ常にCopy可能
#[derive(Clone, Copy)]
enum Direction {
    Up, Down, Left, Right
}

// データを持つ場合はフィールドによる
#[derive(Clone, Copy)]
enum Value {
    Int(i32),   // i32 は Copy
    Float(f64), // f64 は Copy
}

// String を含む場合は Copy 不可
#[derive(Clone)]  // Copy は実装できない
enum Data {
    Text(String),  // String は Copy ではない
    Number(i32),
}
```

### 例4: ジェネリックトレイト境界

`run` メソッドのシグネチャ：

```rust
pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
    loop {
        terminal.draw(|frame| ui::render(frame, self))?;
        // ...
    }
}
```

**ポイント**:
- `B: Backend` はratatuiのターミナルバックエンドのトレイト
- `CrosstermBackend` や `TestBackend` など、異なるバックエンドを受け入れ可能
- テスト時にモックバックエンドを使用できる

### 例5: serdeのカスタマイズ

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_limit")]
    pub default_limit: usize,

    #[serde(default)]
    pub show_unread_only: bool,

    #[serde(default)]
    pub disabled_feeds: Vec<i64>,
}

fn default_limit() -> usize {
    20
}
```

**ポイント**:
- `#[serde(default)]`: `bool` のデフォルトは `false`、`Vec` のデフォルトは空
- `#[serde(default = "関数名")]`: カスタムデフォルト値を指定
- 設定ファイルで一部のフィールドを省略可能に

---

### 理解度確認 5

**問題**: 以下のrustfeedのコードを見て、質問に答えてください。

```rust
pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()>
```

1. なぜ `B` をジェネリックにしているのでしょうか？
2. もし `B: Backend` ではなく具体的な型 `CrosstermBackend<Stdout>` を使ったらどうなりますか？

<details>
<summary>回答を見る</summary>

**1. ジェネリックにする理由**:

- **テスタビリティ**: `TestBackend` を使ってUIのテストができる
- **柔軟性**: 将来的に別のバックエンド（例：Web用）に対応可能
- **抽象化**: メソッドの実装がバックエンドの詳細に依存しない

例えば、テストでは：
```rust
#[test]
fn test_ui_rendering() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new(/* ... */);

    // テスト可能！
    terminal.draw(|f| ui::render(f, &app)).unwrap();
}
```

**2. 具体的な型を使った場合**:

```rust
// この場合
pub async fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()>
```

- テストでモックバックエンドが使えない
- 他のバックエンド（例：ファイル出力用）に対応できない
- 関数の再利用性が下がる

ただし、アプリケーションが単一のバックエンドしか使わないことが確実なら、具体的な型を使う方がシンプルです。トレードオフを理解した上で選択することが重要です。

</details>

---

## まとめ

| 概念 | Rust | C++ | TypeScript |
|------|------|-----|------------|
| 抽象化 | トレイト | 抽象クラス/コンセプト | インターフェース |
| ジェネリクス | 単相化 + トレイト境界 | テンプレート | 型パラメータ |
| 制約チェック | 定義時 | 使用時（C++20以前） | コンパイル時（型レベル） |
| デフォルト実装 | トレイト内で直接 | 仮想関数で可能 | Mixinパターン |
| 自動導出 | `#[derive]` マクロ | なし（手動/外部ツール） | なし |

次は [04-async-await.md](./04-async-await.md) で非同期プログラミングについて学びます。
