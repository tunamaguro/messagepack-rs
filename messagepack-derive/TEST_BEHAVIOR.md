# messagepack-derive Test Behavior

`messagepack-derive` のテストケースから読み取れる想定挙動をまとめたメモ。

対象:

- 実行時の encode/decode テスト: `messagepack-derive/tests/*.rs`
- derive 成功/失敗のコンパイルテスト: `messagepack-derive/tests/success/*.rs`, `messagepack-derive/tests/fail/*.rs`

## 概要

- named struct は既定で MessagePack map として encode される
- named struct の decode は map を受け付ける
- named struct の decode は array 形式も受け付ける
- tuple struct は MessagePack array として encode/decode される
- unit struct は MessagePack `nil` として encode/decode される
- `#[msgpack(array)]` により named struct を array 形式で encode/decode できる
- `#[msgpack(bytes)]` は対象フィールドを MessagePack binary として扱う
- `#[msgpack(encode_with = "...", decode_with = "...")]` でカスタム変換を差し込める
- `PhantomData` フィールドは encode/decode 対象から除外される
- `Option<T>` の欠損フィールドは decode 時に `None` になる
- `#[msgpack(default)]` の付いた欠損フィールドは `Default::default()` で補完される
- generics, lifetime, associated type, 再帰的な型境界を含む struct でも derive できる

## 正常系

### named struct の既定挙動

`#[derive(Encode, Decode)]` を付けた named struct は、既定で MessagePack map として encode される。

例:

```rust
#[derive(Encode, Decode)]
struct Point {
    x: u32,
    y: u32,
}
```

`Point { x: 10, y: 20 }` は次のような map 表現になる想定。

```text
0x82                // fixmap 2
0xa1 'x' 0x0a       // "x" => 10
0xa1 'y' 0x14       // "y" => 20
```

対応テスト:

- `messagepack-derive/tests/basic_struct.rs`

### named struct は map / array の両形式で decode できる

named struct の decode は、既定の map 形式に加えて array 形式も受け付ける想定になっている。

例:

```rust
#[derive(Encode, Decode)]
struct Point {
    x: u8,
    y: u8,
}
```

次の array 表現からも decode できる。

```text
0x92 0x0a 0x14      // [10, 20]
```

対応テスト:

- `messagepack-derive/tests/cross_format_decode.rs`

### tuple struct

tuple struct は MessagePack array として encode/decode される。

例:

```rust
#[derive(Encode, Decode)]
struct Pair(u8, u16);
```

`Pair(42, 65535)` は次のような array 表現になる想定。

```text
0x92 0x2a 0xff 0xff
```

対応テスト:

- `messagepack-derive/tests/tuple_struct.rs`

### unit struct

unit struct は MessagePack `nil` として encode/decode される。

例:

```rust
#[derive(Encode, Decode)]
struct Unit;
```

対応テスト:

- `messagepack-derive/tests/unit_struct.rs`

### `#[msgpack(array)]`

named struct でも `#[msgpack(array)]` を付けると array 形式で encode/decode される。
このとき、実データを持つ各フィールドには `#[msgpack(key = N)]` が必要。

例:

```rust
#[derive(Encode, Decode)]
#[msgpack(array)]
struct Record {
    #[msgpack(key = 0)]
    name: String,
    #[msgpack(key = 1)]
    age: u8,
}
```

`Record { name: "Alice".into(), age: 30 }` は次のようになる想定。

```text
0x92
0xa5 'A' 'l' 'i' 'c' 'e'
0x1e
```

対応テスト:

- `messagepack-derive/tests/array_mode.rs`

### `#[msgpack(bytes)]`

`#[msgpack(bytes)]` が付いたフィールドは、通常の配列やシーケンスではなく MessagePack binary として扱われる。

例:

```rust
#[derive(Encode, Decode)]
struct Record<'a> {
    #[msgpack(bytes)]
    bytes: &'a [u8],
}
```

`&[1, 2, 3, 4, 5]` は `bin8` 形式で出力される想定。

```text
0xc4 0x05 0x01 0x02 0x03 0x04 0x05
```

また、compile-pass テストから次の型でも `bytes` 属性が許可される想定が分かる。

- `Vec<u8>`
- `[u8; N]`
- `Box<[u8]>`
- `&[u8]`

対応テスト:

- `messagepack-derive/tests/bytes_attr.rs`
- `messagepack-derive/tests/success/bytes_attr.rs`

### `encode_with` / `decode_with`

フィールド単位で custom encode/decode 関数を指定できる。
この指定があるフィールドは、元の型が `Encode` / `Decode` を実装していなくてもよい想定。

例:

```rust
#[derive(Encode, Decode)]
struct Custom {
    #[msgpack(encode_with = "encode_doubled", decode_with = "decode_halved")]
    value: u32,
}
```

`value = 21` は encode 時に `42` として書かれ、decode 時に `21` に戻る。

さらに compile-pass テストでは、`NoEncodeDecode` のような `Encode` / `Decode` 未実装型にも custom 関数経由で derive できることを確認している。

対応テスト:

- `messagepack-derive/tests/custom_encode_decode.rs`
- `messagepack-derive/tests/success/custom_encode_decode.rs`

### `PhantomData`

`PhantomData<T>` フィールドは encode/decode の実データ対象から除外され、map や array の要素数にも含まれない想定。

例:

```rust
#[derive(Encode, Decode)]
struct Foo {
    data: PhantomData<u8>,
}
```

この場合、出力は空 map になる。

```text
0x80                // fixmap 0
```

また、`#[msgpack(array)]` を付けた struct に `PhantomData<T>` があっても、そのフィールドは配列長に寄与しない。

compile-pass テストからは、`PhantomData<T>` を含むことで `T: Encode + Decode` を要求しないことも意図されている。

対応テスト:

- `messagepack-derive/tests/phantom_data.rs`
- `messagepack-derive/tests/success/phantom_data.rs`

### 欠損フィールドの decode

#### `Option<T>`

`Option<T>` フィールドが入力に存在しない場合、decode 結果は `None` になる。

例:

```rust
#[derive(Encode, Decode)]
struct S1 {
    foo: u8,
    bar: Option<u8>,
}
```

`{"foo": 12}` だけが与えられた場合、`bar` は `None` になる想定。

#### `#[msgpack(default)]`

`#[msgpack(default)]` が付いたフィールドが入力に存在しない場合、`Default::default()` で補完される。

例:

```rust
#[derive(Encode, Decode)]
struct S2 {
    foo: u8,
    #[msgpack(default)]
    bar: u8,
}
```

`{"foo": 12}` だけが与えられた場合、`bar` は `0` になる想定。

generic フィールドでも同様で、`T: Default` があれば `#[msgpack(default)] bar: T` は成立する。

また compile-pass テストから、`#[msgpack(default)] foo: Option<T>` は `T: Default` でなくても通る想定が読み取れる。これは `Option<T>` 自体が `Default` を実装しているため。

対応テスト:

- `messagepack-derive/tests/decode_option_missing.rs`
- `messagepack-derive/tests/success/default.rs`

### generics / lifetime / associated type / 再帰的境界

compile-pass テストでは、derive が次のケースを処理できる想定になっている。

- generic struct
- lifetime parameter を持つ struct
- 関連型 `T::Item` をフィールドに持つ struct
- 相互再帰的な入れ子構造を持つ generic struct

対応テスト:

- `messagepack-derive/tests/success/generics.rs`
- `messagepack-derive/tests/success/lifetime_generics.rs`
- `messagepack-derive/tests/success/associated_type.rs`
- `messagepack-derive/tests/success/nested_bounds.rs`

## 異常系

### enum は未対応

`Encode` / `Decode` derive は enum に対して未対応で、コンパイルエラーになる想定。

対応テスト:

- `messagepack-derive/tests/fail/enum.rs`

期待エラー:

- `Encode derive is not yet supported for enums`
- `Decode derive is not yet supported for enums`

### `map` と `array` は同時指定不可

`#[msgpack(map, array)]` のように両方を付けるとコンパイルエラーになる想定。

対応テスト:

- `messagepack-derive/tests/fail/map_and_array.rs`

期待エラー:

- `` `map` and `array` are mutually exclusive ``

### array モードでは各フィールドに key が必要

`#[msgpack(array)]` を付けた named struct では、実データを持つ各フィールドに `#[msgpack(key = N)]` が必要。

対応テスト:

- `messagepack-derive/tests/fail/array_missing_key.rs`

期待エラー:

- `all fields must have #[msgpack(key = N)] when using #[msgpack(array)]`

### tuple struct を map として扱うことはできない

tuple struct に `#[msgpack(map)]` を付ける構成は不正として扱う想定。

対応テスト:

- `messagepack-derive/tests/fail/tuple_as_map.rs`

### `#[msgpack(bytes)]` は byte 系の型にしか使えない

`#[msgpack(bytes)]` は `Vec<u8>` や `&[u8]` のような byte 系フィールド向けであり、`Vec<Foo>` や `&[Foo]` のような型には使えない想定。

対応テスト:

- `messagepack-derive/tests/fail/no_bytes.rs`

### `PhantomData` に key 指定はできない

`PhantomData<T>` は shape に含まれないため、`#[msgpack(key = N)]` を付けるのは不正な想定。

対応テスト:

- `messagepack-derive/tests/fail/phantomdata_with_key.rs`

### `#[msgpack(default)]` には `Default` が必要

`#[msgpack(default)]` を付けたフィールド型が `Default` を実装していない場合、derive は成立しない想定。

対応テスト:

- `messagepack-derive/tests/fail/default_missing.rs`

## 補足

このファイルは「現行テストが表現している期待仕様」の整理であり、現在の実装がすべてのケースを満たしていることまでは保証しない。

実際、作業時点では `cargo test -p messagepack-derive` は通っておらず、`DecodeBorrowed` 実装周辺と integration test 側にビルド不整合がある。
