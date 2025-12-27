# 01. 所有権とボローイング

Rustの最も重要で特徴的な概念が**所有権（Ownership）**と**ボローイング（Borrowing）**です。これらはコンパイル時にメモリ安全性を保証するRustの中核的な仕組みです。

## 目次

1. [所有権とは何か](#1-所有権とは何か)
2. [ムーブセマンティクス](#2-ムーブセマンティクス)
3. [参照とボローイング](#3-参照とボローイング)
4. [可変参照と不変参照](#4-可変参照と不変参照)
5. [rustfeedでの実例](#5-rustfeedでの実例)

---

## 1. 所有権とは何か

### Rustの所有権ルール

Rustでは、すべての値に「所有者（owner）」が存在し、以下の3つのルールに従います：

1. **各値は所有者を1つだけ持つ**
2. **所有者がスコープを抜けると、値は自動的に破棄される**
3. **所有権は移動（move）できる**

### C++との比較

**C++** では、メモリ管理は主に3つの方法で行われます：

```cpp
// C++: 手動メモリ管理（危険）
void example() {
    int* ptr = new int(42);
    // ... 処理 ...
    delete ptr;  // 忘れるとメモリリーク
    // ptr をまた使うとダングリングポインタ
}

// C++: スマートポインタ（C++11以降）
void safe_example() {
    std::unique_ptr<int> ptr = std::make_unique<int>(42);
    // スコープを抜けると自動解放
}

// C++: RAII パターン
class Resource {
public:
    Resource() { /* リソース確保 */ }
    ~Resource() { /* リソース解放 */ }
};
```

**Rust** では、所有権がこれらを言語レベルで統一しています：

```rust
fn example() {
    let value = String::from("hello");  // valueが所有者
    // ... 処理 ...
}  // スコープを抜けると自動的にdropが呼ばれる
```

### 重要なポイント

| 言語 | メモリ管理 | 安全性の保証 |
|------|------------|--------------|
| C++ | 手動 / スマートポインタ（任意） | 実行時（または未定義動作） |
| Rust | 所有権システム（強制） | コンパイル時 |

---

### 理解度確認 1

**問題**: 以下のRustコードはコンパイルできるでしょうか？できない場合、なぜでしょうか？

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1;
    println!("{}", s1);
}
```

<details>
<summary>回答を見る</summary>

**コンパイルエラーになります。**

`let s2 = s1;` の時点で、`s1` の所有権が `s2` に移動（ムーブ）します。その後 `s1` を使おうとすると「value borrowed here after move」というエラーになります。

これはRustの所有権ルール1「各値は所有者を1つだけ持つ」に基づいています。

修正方法：
```rust
let s2 = s1.clone();  // 明示的にコピーを作成
println!("{}", s1);   // これでOK
```

</details>

---

## 2. ムーブセマンティクス

### 所有権の移動

Rustでは、代入や関数への引数渡しで所有権が「移動」します：

```rust
fn take_ownership(s: String) {
    println!("{}", s);
}  // s はここでドロップされる

fn main() {
    let my_string = String::from("hello");
    take_ownership(my_string);  // 所有権が移動
    // println!("{}", my_string);  // エラー！もう使えない
}
```

### C++との比較

**C++** のムーブセマンティクス（C++11以降）は明示的な `std::move` が必要：

```cpp
void take_ownership(std::string s) {
    std::cout << s << std::endl;
}

int main() {
    std::string my_string = "hello";
    take_ownership(std::move(my_string));  // 明示的にムーブ
    // my_string は「有効だが未規定の状態」
    // 使えるが、中身は保証されない（危険！）
}
```

**Rust** の違い：
- ムーブはデフォルトの動作
- ムーブ後の変数はコンパイル時に使用禁止
- 「有効だが未規定」という危険な状態が存在しない

### Copy トレイト

一部の型（整数、浮動小数点数、bool、参照など）は `Copy` トレイトを実装しており、代入時に自動的にコピーされます：

```rust
fn main() {
    let x = 42;     // i32 は Copy
    let y = x;      // コピーが作られる
    println!("{}", x);  // OK! x はまだ有効
}
```

---

### 理解度確認 2

**問題**: 以下のC++とRustのコードを比較してください。それぞれムーブ後の変数の状態はどうなりますか？

```cpp
// C++
std::vector<int> v1 = {1, 2, 3};
std::vector<int> v2 = std::move(v1);
std::cout << v1.size() << std::endl;  // 何が出力される？
```

```rust
// Rust
let v1 = vec![1, 2, 3];
let v2 = v1;
println!("{}", v1.len());  // コンパイルできる？
```

<details>
<summary>回答を見る</summary>

**C++**: コンパイルは通りますが、`v1.size()` の結果は**未規定**です。おそらく `0` が出力されますが、標準は何も保証しません。これは潜在的なバグの原因になります。

**Rust**: **コンパイルエラー**になります。`v1` はムーブ後に使用できないことがコンパイル時に検出されます。

この違いがRustの安全性の核心です。C++では実行時に問題が発生しうる箇所を、Rustはコンパイル時に防ぎます。

</details>

---

## 3. 参照とボローイング

所有権を移動せずに値を使いたい場合、**参照（reference）**を使って**借用（borrow）**します。

### 不変参照

```rust
fn calculate_length(s: &String) -> usize {  // &String は参照
    s.len()
}  // s は参照なので、ここで元の値はドロップされない

fn main() {
    let my_string = String::from("hello");
    let length = calculate_length(&my_string);  // 借用
    println!("'{}' has {} chars", my_string, length);  // まだ使える！
}
```

### C++との比較

**C++** の参照：

```cpp
size_t calculate_length(const std::string& s) {  // 参照
    return s.length();
}

int main() {
    std::string my_string = "hello";
    size_t length = calculate_length(my_string);  // 暗黙的に参照
    std::cout << my_string << " has " << length << " chars" << std::endl;
}
```

表面的には似ていますが、重要な違いがあります：

| 特徴 | C++ | Rust |
|------|-----|------|
| 参照の記法 | 暗黙的に渡せる | `&` を明示的に書く |
| ダングリング参照 | 実行時エラー/未定義動作 | コンパイルエラー |
| 参照の有効期間 | プログラマの責任 | コンパイラが検証 |

---

### 理解度確認 3

**問題**: 以下のRustコードは安全でしょうか？安全でない場合、Rustはどうやってそれを防ぎますか？

```rust
fn get_reference() -> &String {
    let s = String::from("hello");
    &s
}
```

<details>
<summary>回答を見る</summary>

**コンパイルエラーになります。**

関数 `get_reference` は `String` のローカル変数 `s` への参照を返そうとしています。しかし、`s` は関数の終わりでドロップされるため、返される参照は無効なメモリを指す「ダングリング参照」になります。

Rustはこれを**ライフタイム検査**で防ぎます。エラーメッセージ：
```
error[E0106]: missing lifetime specifier
--> src/main.rs:1:24
  |
1 | fn get_reference() -> &String {
  |                        ^ expected named lifetime parameter
```

C++では同様のコードがコンパイルでき、実行時に未定義動作を引き起こします：

```cpp
std::string& get_reference() {
    std::string s = "hello";
    return s;  // 警告は出るが、コンパイルは通る
}
```

</details>

---

## 4. 可変参照と不変参照

Rustでは、参照に2種類あります：

- `&T` : 不変参照（読み取り専用）
- `&mut T` : 可変参照（読み書き可能）

### 借用のルール

**同時に存在できる参照の制限**：

1. **不変参照は複数可** - 読み取りは並行して安全
2. **可変参照は1つだけ** - 書き込みは排他的
3. **不変参照と可変参照は同時に存在できない**

```rust
fn main() {
    let mut s = String::from("hello");

    let r1 = &s;     // OK: 不変参照1
    let r2 = &s;     // OK: 不変参照2
    println!("{} {}", r1, r2);

    let r3 = &mut s; // OK: r1, r2 はもう使われない
    r3.push_str(", world");
    println!("{}", r3);
}
```

```rust
fn main() {
    let mut s = String::from("hello");

    let r1 = &s;
    let r2 = &mut s;  // エラー！不変参照がまだ有効
    println!("{}", r1);
}
```

### C++との比較

**C++** には同様の制限がありません：

```cpp
int main() {
    std::string s = "hello";

    const std::string& r1 = s;  // const参照
    std::string& r2 = s;        // 可変参照（同時に存在OK）

    r2 += ", world";            // r1 の指す内容も変わる
    std::cout << r1 << std::endl;  // "hello, world"
}
```

C++ではこれが許可されますが、マルチスレッド環境ではデータ競合の原因になります。Rustはコンパイル時にこれを防ぎます。

---

### 理解度確認 4

**問題**: 以下のRustコードで、コンパイルエラーになる行はどれでしょうか？

```rust
fn main() {
    let mut vec = vec![1, 2, 3];

    let first = &vec[0];        // 行A
    vec.push(4);                // 行B
    println!("{}", first);      // 行C
}
```

<details>
<summary>回答を見る</summary>

**行Bでコンパイルエラーになります。**

`first` は `vec` への不変参照です。`vec.push(4)` は `vec` を可変で借用しようとしますが、不変参照 `first` がまだ有効（行Cで使用される）なため、借用ルール違反になります。

これは実際には深い意味があります：`push` はベクターの再割り当てを引き起こす可能性があり、その場合 `first` が指すメモリは無効になります。Rustはこの潜在的なバグをコンパイル時に防いでいます。

修正方法：
```rust
fn main() {
    let mut vec = vec![1, 2, 3];

    let first = vec[0];         // コピーを取る（i32はCopy）
    vec.push(4);                // OK
    println!("{}", first);      // OK
}
```

または：
```rust
fn main() {
    let mut vec = vec![1, 2, 3];

    let first = &vec[0];
    println!("{}", first);      // 参照を先に使い終わる
    vec.push(4);                // OK：firstはもう使われない
}
```

</details>

---

## 5. rustfeedでの実例

### 例1: 関数が所有権を受け取る

[crates/rustfeed-core/src/models.rs](../../crates/rustfeed-core/src/models.rs) の `Feed::new` 関数：

```rust
pub fn new(url: String, title: String, description: Option<String>) -> Self {
    let now = Utc::now();
    Self {
        id: 0,
        url,       // 所有権を受け取ってフィールドに格納
        title,     // 同上
        description,
        created_at: now,
        updated_at: now,
        custom_name: None,
        category: None,
        priority: 0,
    }
}
```

**ポイント**:
- 引数 `url: String` は所有権を受け取る
- 呼び出し側は `"https://...".to_string()` で `String` を作成して渡す
- `Feed` 構造体がこれらの `String` の新しい所有者になる

### 例2: 関数が参照を借用する

[crates/rustfeed-core/src/db.rs](../../crates/rustfeed-core/src/db.rs) の `Database::add_feed` メソッド：

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
- `&self`: `Database` への不変参照（データベース接続を読み取り）
- `&Feed`: `Feed` への不変参照（フィードの情報を読み取るだけ）
- 呼び出し側は `Feed` の所有権を保持し続ける

### 例3: コマンドでの組み合わせ

[crates/rustfeed-cli/src/commands.rs](../../crates/rustfeed-cli/src/commands.rs) の `add_feed` 関数：

```rust
pub async fn add_feed(db: &Database, url: &str, name: Option<&str>) -> Result<()> {
    let (mut feed_info, _articles) = feed::fetch_feed(url)
        .await?;

    if let Some(custom_name) = name {
        feed_info.title = custom_name.to_string();  // 新しいStringを作成
    }

    let id = db.add_feed(&feed_info)?;  // 参照を渡す
    // ...
}
```

**ポイント**:
- `db: &Database` - データベースへの参照を借用
- `url: &str` - 文字列スライスへの参照（所有権を受け取らない）
- `&feed_info` - ローカル変数への参照を渡す
- `custom_name.to_string()` - `&str` から新しい `String` を作成

---

### 理解度確認 5

**問題**: rustfeedの `Feed::display_name` メソッドを見てください：

```rust
pub fn display_name(&self) -> &str {
    self.custom_name.as_deref().unwrap_or(&self.title)
}
```

1. なぜ `&str` を返すのでしょうか？`String` を返すのとどう違いますか？
2. `as_deref()` は何をしていますか？

<details>
<summary>回答を見る</summary>

**1. `&str` vs `String`**:

- `&str` を返すことで、新しいメモリ割り当てを避けています
- `self.title` や `self.custom_name` 内の既存のデータへの参照を返すだけ
- `String` を返すと、毎回文字列のコピーが発生してしまいます

戻り値の `&str` のライフタイムは `&self` と同じです。つまり、`Feed` 構造体が有効な間だけ、返された参照も有効です。

**2. `as_deref()` の役割**:

`self.custom_name` の型は `Option<String>` です。

- `as_deref()` は `Option<String>` を `Option<&str>` に変換
- `String` の中身（`str`）への参照を取得
- 所有権を移動せずに、中の値を参照として取り出せる

変換の流れ：
```rust
Option<String>  --as_deref()-->  Option<&str>  --unwrap_or()-->  &str
```

C++で同様のことをする場合：
```cpp
const std::string& display_name() const {
    if (custom_name.has_value()) {
        return custom_name.value();
    }
    return title;
}
```

Rustの `Option::as_deref()` は、この「所有型をその参照型に変換」というパターンを簡潔に表現しています。

</details>

---

## まとめ

| 概念 | Rust | C++ |
|------|------|-----|
| 所有権 | 言語組み込み、コンパイル時検証 | `unique_ptr`で模倣可能、強制力なし |
| ムーブ | デフォルト動作、使用禁止を保証 | `std::move`で明示、未規定状態あり |
| 参照 | 借用ルールで安全性保証 | 任意、ダングリング可能 |
| 可変性 | 参照レベルで排他制御 | `const`のみ、同時アクセス可能 |

次は [02-error-handling.md](./02-error-handling.md) でRustのエラー処理について学びます。
