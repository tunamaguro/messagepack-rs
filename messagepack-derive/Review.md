# messagepack-derive コードレビュー (第2回)

前回のレビューで指摘した問題の大部分が修正されている。
全既存テスト通過: **OK** (runtime 22テスト + compile-pass 12ケース + compile-fail 8ケース)

---

## 1. 前回指摘事項の対応状況

| # | 指摘 | 重要度 | 状態 | 対応内容 |
|---|------|:---:|:---:|------|
| 3.1 | `#[msgpack(default)]` が encode でも省略される | Critical | **修正済** | `is_skipped()` を `is_skipped_for_encode()` / `is_skipped_for_decode()` に分離。encode 側は `is_phantom` のみでフィルタ。roundtrip テストも追加 |
| 3.2 | `Option<T>` + `bytes` で内部の bytes 属性がリセットされる | Medium | **修正済** | `decode_field_expr` で `bytes` チェックを `option_inner` より先に配置。`DecodeBytes` の `Option<T>` blanket impl に委譲する設計に変更 |
| 3.3 | `decode_non_option_with_format_expr` が bytes 時に format を無視 | Medium | **修正済** | `decode_bytes_with_format(#format, __reader)` を呼ぶように修正 |
| 3.4 | `::std::boxed::Box` の使用 (no_std 非対応) | Low | **修正済** | `extern crate alloc as __msgpack_alloc;` + `__msgpack_alloc::boxed::Box` に変更 |
| 3.5 | `Meta::NameValue` ブランチがデッドコード | Low | **修正済** | 該当ブランチを削除しエラーにする処理に変更。`fail/name_value_attr.rs` テストを追加 |

---

## 2. 今回発見した問題

### 2.1 [Medium] named struct の array decode で `Option<T>` が暗黙的にオプショナルにならない

named struct の map decode では、`Option<T>` フィールドが入力に存在しなければ `None` に補完される。しかし array decode では `minimum_array_len` が `field.attrs.default` のみを考慮するため、`Option<T>` フィールド（`#[msgpack(default)]` なし）は必須扱いになる。

**再現手順**: `tests/review_issues.rs` の `option_missing_from_array_decode` テストで確認可能。

```rust
#[derive(Encode, Decode)]
struct WithOption {
    foo: u8,
    bar: Option<u8>,
}
```

- map decode: `{"foo": 12}` → `WithOption { foo: 12, bar: None }` ✓
- array decode: `[12]` → **`Error::InvalidData`** ✗ (foo のみの 1 要素配列は拒否される)

**原因**: `minimum_array_len` は `Option<T>` を特別扱いせず、`#[msgpack(default)]` 属性が付いているかどうかのみで判定している。

```rust
fn minimum_array_len(fields: &[&FieldInfo]) -> usize {
    fields
        .iter()
        .rposition(|field| !field.attrs.default)  // Option<T> はここで必須と判定される
        .map(|index| index + 1)
        .unwrap_or(0)
}
```

**影響**: cross-format decode（map と array の両方を受け付ける設計）において、map では受理される入力が array では拒否されるケースがある。

**修正案A**: `minimum_array_len` で `Option<T>` フィールドも `default` と同様にオプショナルとみなす。

```rust
fn minimum_array_len(fields: &[&FieldInfo]) -> usize {
    fields
        .iter()
        .rposition(|field| !field.attrs.default && option_inner(&field.ty).is_none())
        .map(|index| index + 1)
        .unwrap_or(0)
}
```

**修正案B**: 意図的な仕様差とするなら、ドキュメントで明記する。array decode では全フィールドが必須であること、`#[msgpack(default)]` を付けて明示的にオプショナルにする必要があることを説明する。

---

## 3. コード品質の改善提案

### 3.1 [Low] `sorted_array_fields` と `validate_*_fields` の重複

`encode.rs` と `decode.rs` がそれぞれ独立に `sorted_array_fields` と `validate_skipped_fields` / `validate_decode_fields` を持っている。ロジックはほぼ同一なので `shared.rs` に統合できる。

### 3.2 [Low] `tuple_struct.rs` / `unit_struct.rs` に `#[test]` 関数がない

これらのファイルは `fn main()` で書かれているため、`cargo test` 実行時に `running 0 tests` と表示される。`#[test]` 関数に書き直すか、既に `tests/success/` 側に compile-pass テストがあるので runtime テストとしてアサーション付き `#[test]` に変換すべき。

### 3.3 [Info] `add_decode_bounds` における `default` フィールドの二重バウンド

`#[msgpack(default)]` フィールドのジェネリック型 `T` に対して `T: Default` と `T: DecodeBorrowed` の両方が要求される。フィールドがデータに存在する場合はデコードが必要なのでこれ自体は正しいが、ユーザーが「default はデコード不要なフィールド」と誤解する可能性がある。doc コメントでの説明を推奨。

---

## 4. 検証テストの結果

`tests/review_issues.rs` に以下のテストを追加して検証した:

| テスト名 | 結果 | 検証内容 |
|----------|:---:|------|
| `option_missing_from_map_decode` | ✅ PASS | map decode での Option 欠損 (既知の正常動作) |
| `option_missing_from_array_decode` | ❌ FAIL | **2.1 の問題を確認** — array decode で Option 欠損が拒否される |
| `option_present_roundtrip` | ✅ PASS | Option に値がある場合の roundtrip |
| `option_none_roundtrip` | ✅ PASS | Option が None の場合の roundtrip |
| `option_bytes_present_roundtrip` | ✅ PASS | `#[msgpack(bytes)]` + `Option<&[u8]>` の Some roundtrip |
| `option_bytes_none_roundtrip` | ✅ PASS | `#[msgpack(bytes)]` + `Option<&[u8]>` の None roundtrip |
| `box_field_roundtrip` | ✅ PASS | `Box<T>` フィールドの roundtrip |
| `option_box_some_roundtrip` | ✅ PASS | `Option<Box<T>>` の Some roundtrip |
| `option_box_none_roundtrip` | ✅ PASS | `Option<Box<T>>` の None roundtrip |

---

## 5. まとめ

前回の指摘事項（5件）は全て適切に対応されている。特に Critical だった `#[msgpack(default)]` の encode 省略問題は、`is_skipped_for_encode` / `is_skipped_for_decode` の分離で正しく解決されており、roundtrip テストも追加されている。

新たに発見した問題は 2.1 の「`Option<T>` の array decode における暗黙的オプショナル扱いの不一致」のみ。これは設計判断に依存する部分であり、明示的に `#[msgpack(default)]` を付ければ回避可能。ただし cross-format decode の一貫性の観点では修正が望ましい。
