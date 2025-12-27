# 07. メモリ安全性

Rustの最大の特徴は、ガベージコレクションなしで**コンパイル時にメモリ安全性を保証**することです。ライフタイム、借用チェッカー、そしていくつかの安全なパターンにより、C++で発生しうる多くのバグをコンパイル時に防ぎます。

## 目次

1. [ライフタイムの基本](#1-ライフタイムの基本)
2. [ライフタイム注釈](#2-ライフタイム注釈)
3. [一般的なメモリバグとRustの防止策](#3-一般的なメモリバグとrustの防止策)
4. [スマートポインタ](#4-スマートポインタ)
5. [rustfeedでの実例](#5-rustfeedでの実例)

---

## 1. ライフタイムの基本

### ライフタイムとは

ライフタイムは「参照が有効である期間」を表します。Rustコンパイラは、すべての参照がその対象より長く生存しないことを検証します。

```rust
fn main() {
    let r;                      // 参照を宣言（未初期化）

    {
        let x = 5;              // xのライフタイム開始
        r = &x;                 // rはxを参照
    }                           // xのライフタイム終了（ドロップ）

    // println!("{}", r);       // エラー: rはダングリング参照
}
```

### C++との比較

**C++** ではダングリング参照が実行時エラー（または未定義動作）：

```cpp
int* get_local() {
    int x = 5;
    return &x;  // 警告は出るが、コンパイルは通る
}

int main() {
    int* p = get_local();
    std::cout << *p << std::endl;  // 未定義動作！
}
```

**Rust** ではコンパイル時にエラー：

```rust
fn get_local() -> &i32 {
    let x = 5;
    &x  // コンパイルエラー: xは関数終了時にドロップされる
}
```

### ライフタイムの省略規則

多くの場合、Rustはライフタイムを自動推論します：

```rust
// これらは同じ意味（ライフタイム省略）
fn first_word(s: &str) -> &str { ... }
fn first_word<'a>(s: &'a str) -> &'a str { ... }
```

**省略規則**:
1. 各参照パラメータに別々のライフタイムを割り当て
2. 参照パラメータが1つなら、出力に同じライフタイムを適用
3. `&self` または `&mut self` があれば、そのライフタイムを出力に適用

---

### 理解度確認 1

**問題**: 以下のRustコードはコンパイルできるでしょうか？できない場合、なぜでしょうか？

```rust
fn longest(x: &str, y: &str) -> &str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
```

<details>
<summary>回答を見る</summary>

**コンパイルエラーになります。**

エラーメッセージ：
```
error[E0106]: missing lifetime specifier
 --> src/main.rs:1:33
  |
1 | fn longest(x: &str, y: &str) -> &str {
  |               ----     ----     ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but the
          signature does not say whether it is borrowed from `x` or `y`
```

**理由**: 戻り値が `x` か `y` のどちらから借用されているか、コンパイラは判断できません。ライフタイムを明示する必要があります：

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
```

`'a` は「`x` と `y` のうち短い方のライフタイム」を意味します。戻り値はそのライフタイムの間だけ有効です。

</details>

---

## 2. ライフタイム注釈

### 明示的なライフタイム

```rust
// 'a はライフタイムパラメータ
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
```

### 構造体のライフタイム

参照を持つ構造体にはライフタイムが必要：

```rust
struct Excerpt<'a> {
    part: &'a str,  // 'a は参照の有効期間
}

impl<'a> Excerpt<'a> {
    fn level(&self) -> i32 {
        3
    }

    fn announce_and_return(&self, announcement: &str) -> &str {
        println!("Attention: {}", announcement);
        self.part
    }
}
```

### 'static ライフタイム

プログラム全体で有効な特別なライフタイム：

```rust
// 文字列リテラルは 'static
let s: &'static str = "Hello, world!";

// 静的変数
static GREETING: &str = "Hello";
```

### C++との比較

**C++** には明示的なライフタイム注釈がありません：

```cpp
// C++: ライフタイムはプログラマの責任
class Excerpt {
    std::string_view part;  // 参照先がいつまで有効かは不明
public:
    Excerpt(std::string_view p) : part(p) {}
};

void danger() {
    Excerpt e;
    {
        std::string s = "temporary";
        e = Excerpt(s);
    }  // s はここで破棄
    // e.part はダングリング参照（コンパイルエラーにならない）
}
```

**Rust** ではコンパイルエラー：

```rust
fn danger() {
    let e;
    {
        let s = String::from("temporary");
        e = Excerpt { part: &s };  // エラー: sはスコープを抜けると破棄
    }
    // e を使おうとするとライフタイムエラー
}
```

---

### 理解度確認 2

**問題**: 以下の構造体定義で、ライフタイム注釈はなぜ必要でしょうか？また、`'a` を省略するとどうなりますか？

```rust
struct Config<'a> {
    name: &'a str,
    value: &'a str,
}
```

<details>
<summary>回答を見る</summary>

**ライフタイム注釈が必要な理由**:

`Config` は2つの参照を持っています。Rustは、これらの参照が有効な期間を知る必要があります。`'a` は「`Config` インスタンスは、`name` と `value` の参照先が有効な間だけ存在できる」ことを表現しています。

**省略した場合**:

```rust
struct Config {
    name: &str,   // エラー: expected named lifetime parameter
    value: &str,
}
```

コンパイルエラーになります。構造体に参照を持たせる場合、ライフタイムは常に必要です。

**所有権を持つ代替案**:

ライフタイムを避けたい場合は、`String` を使います：

```rust
struct Config {
    name: String,   // 所有権を持つ
    value: String,
}
```

この場合、`Config` は文字列データを所有するので、ライフタイムの問題は発生しません。ただし、メモリ割り当てのコストがあります。

</details>

---

## 3. 一般的なメモリバグとRustの防止策

### ダングリングポインタ

**C++** での問題：

```cpp
int* dangling() {
    int x = 42;
    return &x;  // xはスコープ外で破棄
}

int* ptr = dangling();
*ptr = 10;  // 未定義動作
```

**Rust** での防止：

```rust
fn dangling() -> &i32 {
    let x = 42;
    &x  // コンパイルエラー: xはドロップされる
}
```

### Use After Free

**C++** での問題：

```cpp
std::vector<int>* create() {
    auto vec = new std::vector<int>{1, 2, 3};
    delete vec;
    return vec;  // 解放済みメモリへのポインタ
}
```

**Rust** での防止：

```rust
fn create() -> Vec<i32> {
    let vec = vec![1, 2, 3];
    vec  // 所有権が移動（ムーブ）
}
// 解放済みメモリにアクセスする方法がない
```

### 二重解放

**C++** での問題：

```cpp
int* ptr = new int(42);
delete ptr;
delete ptr;  // 二重解放（クラッシュまたはメモリ破壊）
```

**Rust** での防止：

```rust
let b = Box::new(42);
drop(b);
// drop(b);  // コンパイルエラー: bはすでにムーブされている
```

### データ競合

**C++** での問題：

```cpp
std::vector<int> vec = {1, 2, 3};

// スレッド1
for (auto& x : vec) {
    std::cout << x;
}

// スレッド2（同時に実行）
vec.push_back(4);  // データ競合！
```

**Rust** での防止：

```rust
let mut vec = vec![1, 2, 3];

// コンパイルエラー: 可変借用と不変借用は同時に存在できない
let r1 = &vec[0];
vec.push(4);  // エラー: vecは借用中
println!("{}", r1);
```

スレッド間のデータ競合も同様に防止されます（`Send`/`Sync` トレイトによる）。

---

### 理解度確認 3

**問題**: 以下のC++コードにはどのような問題がありますか？Rustではどう防がれますか？

```cpp
class Observer {
    std::vector<int>& data;
public:
    Observer(std::vector<int>& d) : data(d) {}
    void print() { for (int x : data) std::cout << x << " "; }
};

void example() {
    Observer* obs;
    {
        std::vector<int> vec = {1, 2, 3};
        obs = new Observer(vec);
    }  // vec は破棄される
    obs->print();  // ダングリング参照！
}
```

<details>
<summary>回答を見る</summary>

**問題**: `Observer` は `vec` への参照を保持していますが、`vec` がスコープを抜けて破棄された後も `Observer` が存在し続けます。`print()` を呼ぶとダングリング参照にアクセスし、未定義動作になります。

**Rustでの防止**:

```rust
struct Observer<'a> {
    data: &'a Vec<i32>,  // ライフタイム注釈が必要
}

fn example() {
    let obs;
    {
        let vec = vec![1, 2, 3];
        obs = Observer { data: &vec };
    }  // vec がドロップ
    // obs.print();  // コンパイルエラー: vecはもう存在しない
}
```

ライフタイムシステムにより、`Observer` の参照が `vec` より長く生存することは許可されません。

**代替設計（所有権を持つ）**:

```rust
struct Observer {
    data: Vec<i32>,  // 所有権を持つ
}

fn example() {
    let obs;
    {
        let vec = vec![1, 2, 3];
        obs = Observer { data: vec };  // ムーブ
    }
    obs.print();  // OK: Observerがデータを所有
}
```

</details>

---

## 4. スマートポインタ

### Box<T>

ヒープにデータを割り当てる最もシンプルなスマートポインタ：

```rust
// スタックに収まらない大きなデータ
let b = Box::new([0u8; 1_000_000]);

// 再帰的なデータ構造
enum List {
    Cons(i32, Box<List>),
    Nil,
}
```

### Rc<T> と Arc<T>

参照カウントによる共有所有権：

```rust
use std::rc::Rc;

let a = Rc::new(vec![1, 2, 3]);
let b = Rc::clone(&a);  // 参照カウント +1
let c = Rc::clone(&a);  // 参照カウント +1

println!("Count: {}", Rc::strong_count(&a));  // 3
```

- `Rc<T>`: シングルスレッド用
- `Arc<T>`: マルチスレッド用（Atomic Reference Count）

### RefCell<T>

内部可変性を提供（実行時借用チェック）：

```rust
use std::cell::RefCell;

let data = RefCell::new(5);

// 実行時に借用ルールをチェック
let mut borrowed = data.borrow_mut();
*borrowed += 1;
```

### C++との比較

| Rust | C++ | 用途 |
|------|-----|------|
| `Box<T>` | `std::unique_ptr<T>` | 単独所有権 |
| `Rc<T>` | `std::shared_ptr<T>` | 共有所有権（ST） |
| `Arc<T>` | `std::shared_ptr<T>` + 原子操作 | 共有所有権（MT） |
| `RefCell<T>` | なし | 内部可変性 |

**違い**:
- Rustは`unsafe`なしでは生ポインタを解参照できない
- C++の`shared_ptr`はスレッド安全性が任意、Rustは`Rc`/`Arc`で明確に分離
- Rustの`RefCell`はシングルスレッド保証の下で動的借用チェック

---

### 理解度確認 4

**問題**: 以下の場面でどのスマートポインタを使うべきでしょうか？

1. ツリー構造で、親ノードが子ノードを所有する場合
2. グラフ構造で、複数のノードが同じエッジを参照する場合（シングルスレッド）
3. キャッシュのように、読み取りは頻繁だが更新は稀な共有データ（マルチスレッド）

<details>
<summary>回答を見る</summary>

**1. ツリー構造の親子関係: `Box<T>`**

```rust
struct TreeNode {
    value: i32,
    children: Vec<Box<TreeNode>>,
}
```

親が子を単独で所有する場合、`Box` で十分です。

**2. グラフ構造の共有エッジ: `Rc<T>`**

```rust
use std::rc::Rc;

struct Edge {
    weight: f64,
}

struct Node {
    edges: Vec<Rc<Edge>>,  // 複数ノードが同じエッジを共有
}
```

シングルスレッドで複数の所有者が必要な場合、`Rc` を使います。

**3. マルチスレッドの共有キャッシュ: `Arc<RwLock<T>>`**

```rust
use std::sync::{Arc, RwLock};

let cache: Arc<RwLock<HashMap<String, Data>>> = Arc::new(RwLock::new(HashMap::new()));

// 読み取り（複数スレッドが同時に可能）
let data = cache.read().unwrap().get("key").cloned();

// 書き込み（排他的）
cache.write().unwrap().insert("key".to_string(), new_data);
```

`Arc` で共有し、`RwLock` で読み書きの同期を取ります。読み取りが多い場合、`RwLock` は `Mutex` より効率的です。

</details>

---

## 5. rustfeedでの実例

### 例1: 所有権による安全なリソース管理

[crates/rustfeed-core/src/db.rs](../../crates/rustfeed-core/src/db.rs):

```rust
pub struct Database {
    conn: Connection,  // rusqlite::Connection を所有
}

impl Database {
    pub fn new() -> Result<Self> {
        let conn = Connection::open(Self::get_db_path()?)?;
        Ok(Self { conn })
    }
}
```

**ポイント**:
- `Database` が `Connection` を所有
- `Database` がドロップされると、`Connection` も自動的に閉じられる
- 明示的な `close()` 呼び出しが不要（RAII）

### 例2: 参照による効率的なデータ渡し

```rust
pub fn add_feed(&self, feed: &Feed) -> Result<i64> {
    self.conn.execute(
        "INSERT INTO feeds ...",
        params![
            feed.url,
            feed.title,
            // ...
        ],
    )?;
    Ok(self.conn.last_insert_rowid())
}
```

**ポイント**:
- `&self`: データベースは借用のみ（所有権は移動しない）
- `&Feed`: フィードも借用（呼び出し側がまだ使える）
- コピーやクローンなしで効率的にデータを渡す

### 例3: ライフタイムとdisplay_name

[crates/rustfeed-core/src/models.rs](../../crates/rustfeed-core/src/models.rs):

```rust
impl Feed {
    pub fn display_name(&self) -> &str {
        self.custom_name.as_deref().unwrap_or(&self.title)
    }
}
```

**ポイント**:
- 戻り値 `&str` のライフタイムは `&self` と同じ（省略規則3）
- `Feed` が有効な間だけ戻り値も有効
- 新しい `String` を割り当てずに既存データを参照

### 例4: Vecとイテレータ

[crates/rustfeed-core/src/feed.rs](../../crates/rustfeed-core/src/feed.rs):

```rust
let articles: Vec<Article> = parsed
    .entries
    .into_iter()  // 所有権を取得してイテレート
    .map(|entry| {
        // entry の所有権を持っているので自由に変換
        let title = entry.title.map(|t| t.content).unwrap_or_else(|| "Untitled".to_string());
        // ...
        Article::new(0, title, url, content, published_at)
    })
    .collect();
```

**ポイント**:
- `into_iter()`: `entries` の所有権を取得
- 各 `entry` は処理後に破棄される
- 新しい `Vec<Article>` が作成される

### 例5: TUIのアプリケーション状態

[crates/rustfeed-tui/src/app.rs](../../crates/rustfeed-tui/src/app.rs):

```rust
pub struct App {
    pub db: Database,           // 所有
    pub config: AppConfig,      // 所有
    pub feeds: Vec<Feed>,       // 所有
    pub articles: Vec<Article>, // 所有
    // ...
}
```

**ポイント**:
- `App` がすべてのデータを所有
- 参照を持たないので、ライフタイムの問題がない
- `App` がドロップされると、すべてのリソースが解放される

### 例6: エラー処理とResult

Rustのエラー処理もメモリ安全性に貢献：

```rust
pub fn get_feed(&self, id: i64) -> Result<Option<Feed>> {
    let mut stmt = self.conn.prepare("SELECT ... WHERE id = ?")?;
    // ...
}
```

**ポイント**:
- `?` で早期リターンする場合、ローカル変数は適切にドロップ
- 例外と違い、スタック巻き戻しが予測可能
- リソースリークが発生しない

---

### 理解度確認 5

**問題**: rustfeedの設計を見て、以下の質問に答えてください。

1. なぜ `App` 構造体は `Database` への参照ではなく、`Database` を所有しているのでしょうか？
2. `Database::add_feed(&self, feed: &Feed)` で、なぜ `&self` と `&Feed` の両方が参照なのでしょうか？

<details>
<summary>回答を見る</summary>

**1. App が Database を所有する理由**:

- **シンプルなライフタイム**: `App` が `Database` を所有すると、ライフタイム注釈が不要
- **リソース管理**: `App` がドロップされると、`Database` も自動的に閉じられる
- **単一の所有者**: `Database` は一箇所でのみ管理されるべき（複数箇所から書き込みは危険）
- **長寿命**: `App` はアプリケーション全体で生存するので、`Database` も同様に長寿命

参照にした場合の複雑さ：
```rust
// 参照にすると、ライフタイムが必要
struct App<'a> {
    db: &'a Database,  // ライフタイム注釈
    // ...
}

// 使用側でも管理が必要
fn main() {
    let db = Database::new()?;
    let app = App::new(&db);  // dbはappより長く生存する必要
}
```

**2. &self と &Feed が両方参照な理由**:

- `&self`: `Database` の所有権は移動したくない（他のメソッドでも使う）
- `&Feed`: `Feed` データを読み取るだけで、所有権は不要
- **効率性**: データのコピーを避ける
- **呼び出し側の利便性**: 呼び出し後も `Feed` を使い続けられる

もし所有権を取ると：
```rust
// これだと呼び出し側で feed が使えなくなる
pub fn add_feed(self, feed: Feed) -> Result<i64> { ... }

// 使用側
let feed = Feed::new(...);
db.add_feed(feed);  // feed の所有権が移動
// println!("{}", feed.title);  // エラー: feed はもう使えない
```

</details>

---

## まとめ

| 問題 | C++ | Rust |
|------|-----|------|
| ダングリングポインタ | 実行時エラー/未定義動作 | コンパイルエラー |
| Use After Free | 実行時エラー/未定義動作 | コンパイルエラー |
| 二重解放 | 実行時エラー/メモリ破壊 | コンパイルエラー |
| データ競合 | 実行時/未定義動作 | コンパイルエラー |
| リソースリーク | 可能（RAIIで軽減） | 極めて困難 |
| Null参照 | 可能 | `Option` で型安全に |

### Rustのメモリ安全性を支える3つの柱

1. **所有権システム**: 各値に1つの所有者
2. **借用チェッカー**: 参照の有効期間をコンパイル時に検証
3. **ライフタイム**: 参照の寿命を型システムで表現

これらにより、Rustは「安全かつ高速」を実現しています。

---

## 学習ガイド完了

おめでとうございます！全7章の学習ガイドを完了しました。

### 次のステップ

1. **実践**: rustfeedのコードを実際に読み、修正してみる
2. **The Rust Book**: [公式ドキュメント](https://doc.rust-lang.org/book/)でさらに深く学ぶ
3. **Rustlings**: [練習問題](https://github.com/rust-lang/rustlings)で手を動かす
4. **Rust by Example**: [例題集](https://doc.rust-lang.org/rust-by-example/)で様々なパターンを学ぶ

Happy Rusting!
