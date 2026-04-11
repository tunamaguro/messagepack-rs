# messagepack-derive コードレビュー

全テスト通過: **OK** (runtime 18テスト + compile-pass 12ケース + compile-fail 7ケース)

---

## 1. 全体構成

| ファイル | 役割 | 行数(概算) |
|----------|------|------------|
| `src/lib.rs` | proc-macro エントリポイント (`Encode`, `Decode`) | 60 |
| `src/shared.rs` | AST 解析、属性パース、型ユーティリティ | 480 |
| `src/encode.rs` | `Encode` trait の impl 生成 | 170 |
| `src/decode.rs` | `DecodeBorrowed` trait の impl 生成 | 500 |

全体の設計は明快で、`shared.rs` に共通構造を集約し、`encode.rs` / `decode.rs` がそれぞれコード生成に専念する構成になっている。

---

## 2. 良い点

### 2.1 安全なマクロ設計
- 全パス修飾 (`::messagepack_core::...`, `::core::...`) により、ユーザーのスコープ内の名前衝突を回避している。
- 内部変数は `__` プレフィクスでハイジーンを確保。
- `#[automatically_derived]` を付与しており、lint の扱いが正しい。

### 2.2 Cross-format decode の柔軟性
named struct の decode が map / array 両形式を受け付ける設計は、実運用でのフォーマット互換性を高めている。エンコーダ側が map を出力しても array を出力しても正しくデコードできる。

### 2.3 コンパイルテストの充実
`trybuild` を使った compile-pass / compile-fail テストにより、型境界の正確性やエラーメッセージの安定性が保証されている。特に以下のケースが網羅的:
- generic, lifetime, associated type, 再帰的な型境界 (`nested_bounds.rs`)
- `PhantomData<T>` が `T: Encode` を要求しないこと
- `encode_with` / `decode_with` で `Encode`/`Decode` 未実装型も使えること

### 2.4 型境界の推論
`collect_bound_types` がジェネリクスの型パラメータを再帰的に探索し、`Vec<T>` のような入れ子型でも正しく `T: Encode` / `T: DecodeBorrowed` を追加する仕組みは正確に機能している。

### 2.5 decode バリデーション
- 重複キーの検出
- 必須フィールドの欠損検出
- 不明キーのスキップ（`Any` で消費）
- 長さ不一致の拒否

---

## 3. バグ・問題点

### 3.1 [Critical] `#[msgpack(default)]` フィールドが encode でも省略される

`FieldInfo::is_skipped()` は `self.is_phantom || self.attrs.default` を返すが、encode 側でも同じ `is_skipped()` でフィールドをフィルタしている。

```rust
// shared.rs
pub fn is_skipped(&self) -> bool {
    self.is_phantom || self.attrs.default
}
```

```rust
// encode.rs - encode_named, encode_tuple ともに
let active = fields.iter().filter(|field| !field.is_skipped()).collect::<Vec<_>>();
```

**結果**: `#[msgpack(default)] bar: u8` のフィールドに値 `42` を設定しても、encode 時にバイト列に含まれない。decode 側ではこのフィールドが見つからないので `Default::default()` (= `0`) に戻ってしまい、データが静かに失われる。

**修正案**: encode 側では `is_phantom` のみでフィルタし、`default` フィールドは通常通り encode する。もしくは encode 専用の `is_skipped_for_encode()` を用意する。

```rust
// encode.rs 用
fn is_skipped_for_encode(&self) -> bool {
    self.is_phantom
}
```

**テストの問題**: `decode_option_missing.rs` は decode のみテストしており、`S2` / `S3` / `S4` の encode → decode ラウンドトリップを検証していない。この不足がバグを隠している。

### 3.2 [Medium] `Option<T>` + `bytes` 属性の decode で format が二重読みされる可能性

`decode_field_expr` で `option_inner` にマッチした場合、内部の `inner_field` は `bytes: false` にリセットされている:

```rust
// decode.rs - decode_field_expr
let inner_field = FieldInfo {
    // ...
    attrs: crate::shared::FieldAttrs {
        bytes: false,  // ← 元のフィールドの bytes 属性をリセット
        // ...
    },
    // ...
};
```

つまり `Option<&[u8]>` に `#[msgpack(bytes)]` を付けた場合、内部の decode は `bytes` ではなく通常の `DecodeBorrowed` 経由になる。`Option<Vec<u8>>` + `bytes` も同様。
これが意図的であれば明示的にコメントすべきだが、おそらくバグ。

**修正案**: `inner_field` の `bytes` に元の `field.attrs.bytes` を引き継ぐ。

### 3.3 [Medium] `decode_non_option_with_format_expr` が `bytes` 時に format を無視

```rust
fn decode_non_option_with_format_expr(...) -> syn::Result<TokenStream> {
    if field.attrs.bytes {
        // format パラメータを使わず decode_bytes を呼ぶ
        // → format は既に読まれているので、reader からもう一度 format を読むことになる
        return Ok(quote! {{
            let __value: #decode_ty = <#decode_ty as ::messagepack_core::decode::DecodeBytes<#de_lifetime>>::decode_bytes(__reader)?;
            __value
        }});
    }
    // ...
}
```

`Option<T>` の内部で既に format を読んだ後にこの関数が呼ばれると、format が二重消費される。現在は 3.2 の問題で `bytes: false` にリセットされるためこのパスに到達しないが、3.2 を修正するとこの問題が顕在化する。

### 3.4 [Low] `::std::boxed::Box` の使用

ブランチ名が `alloc-support` であることを考慮すると、`decode.rs` の `Box` 生成で `::std::boxed::Box` を使っているのは no_std 環境では動作しない。

```rust
// decode.rs
return Ok(quote! {{
    let __value: #target_ty = ::std::boxed::Box::new(#inner_expr);
    __value
}});
```

**修正案**: `::alloc::boxed::Box` に変更するか、`extern crate alloc` をマクロ展開時のプリアンブルに含める。

### 3.5 [Low] `parse_field_attrs` の `Meta::NameValue` ブランチ

```rust
Meta::NameValue(MetaNameValue {
    value: Expr::Lit(ExprLit { lit: Lit::Str(lit), .. }),
    path,
    ..
}) => {
    if path.is_ident("encode_with") {
        out.encode_with = Some(lit.parse()?);
    } else if path.is_ident("decode_with") {
        out.decode_with = Some(lit.parse()?);
    }
}
```

このブランチは `#[msgpack = "..."]` 形式を処理するが、この形式は `key`, `bytes`, `default` には対応しておらず、ドキュメントにも記載がない。デッドコードか未完成の実装と思われる。不要であれば削除してエラーにすべき。

---

## 4. コード品質の改善提案

### 4.1 `sorted_array_fields` と `validate_*_fields` の重複

`encode.rs` と `decode.rs` がそれぞれ独立に `sorted_array_fields` と `validate_skipped_fields` / `validate_decode_fields` を持っている。両者はロジックが完全に同一なので `shared.rs` に統合できる。

### 4.2 map decode の key 照合が `as_bytes()` ベース

```rust
match __key.as_bytes() {
    b"x" => { ... }
    b"y" => { ... }
    _ => { ... }
}
```

`ReferenceStrBinDecoder` は str と bin の両方を受け付けるため、バイナリキーでも偶然フィールド名と一致すればマッチする。MessagePack の仕様上これは合理的だが、仕様外のバイナリキーが意図せずマッチするリスクがゼロではない。ドキュメントで str キーのみを想定していることを明記すべき。

### 4.3 `encode_field_expr` の一貫性

encode 側では `&self.#member` と参照を取っているが、戻り値の型に `?` が付いている:

```rust
Ok(quote! { ::messagepack_core::encode::Encode::encode(&self.#member, writer)? })
```

一方、式としてはインラインに `__size += #writes;` と組み合わせて使われる。動作上は正しいが、`encode_with` 版のみ `#path(&self.#member, writer)?` と直接 self 参照しており、Deref 越しの動作は `Encode` クレート側の実装に依存する。

### 4.4 エラーメッセージに span を活用

一部のエラーが `Span::call_site()` を使っている（例: tuple struct に `map` を付けた場合）。可能であれば `mode` 属性の span を渡すとユーザーに親切なエラー表示になる。

---

## 5. テストカバレッジの分析

### 5.1 カバーされているシナリオ

| シナリオ | Runtime テスト | Compile テスト |
|----------|:---:|:---:|
| named struct (map) | `basic_struct.rs`, `map_mode.rs` | `basic_struct.rs`, `map_mode.rs` |
| named struct (array) | `array_mode.rs` | — |
| tuple struct | `tuple_struct.rs` | `tuple_struct.rs` |
| unit struct | `unit_struct.rs` | `unit_struct.rs` |
| `#[msgpack(bytes)]` | `bytes_attr.rs` | `bytes_attr.rs` |
| `encode_with` / `decode_with` | `custom_encode_decode.rs` | `custom_encode_decode.rs` |
| `PhantomData` | `phantom_data.rs` | `phantom_data.rs` |
| Cross-format decode | `cross_format_decode.rs` | — |
| `Option<T>` 欠損 | `decode_option_missing.rs` | — |
| `#[msgpack(default)]` | `decode_option_missing.rs` | `default.rs` |
| decode バリデーション | `decode_validation.rs` | — |
| generics | — | `generics.rs` |
| lifetime | — | `lifetime_generics.rs` |
| associated type | — | `associated_type.rs` |
| 再帰的型境界 | — | `nested_bounds.rs` |
| enum 拒否 | — | `enum.rs` |
| `map` + `array` 同時指定 | — | `map_and_array.rs` |
| array key 欠損 | — | `array_missing_key.rs` |
| tuple + map 拒否 | — | `tuple_as_map.rs` |
| `bytes` + 非byte型 | — | `no_bytes.rs` |
| PhantomData + key 拒否 | — | `phantomdata_with_key.rs` |
| default + Default 未実装 | — | `default_missing.rs` |

### 5.2 不足しているテスト

| 不足シナリオ | 優先度 | 備考 |
|------------|:---:|------|
| `#[msgpack(default)]` フィールドの encode roundtrip | **高** | 3.1 のバグを検出できる |
| `Option<T>` + `#[msgpack(bytes)]` | **高** | 3.2, 3.3 のバグを検出できる |
| `Vec<T>` フィールドの encode/decode | 中 | 基本的な collection 型 |
| ネストした derive 構造体 | 中 | `Wrapper<Point>` のような組み合わせ |
| `Box<T>` フィールドの decode | 中 | `box_inner` パスの検証 |
| `Option<Box<T>>` / `Box<Option<T>>` | 中 | 入れ子の特殊処理パスの検証 |
| 16 フィールド以上の struct (Map16/Array16) | 低 | fixmap/fixarray を超えるケース |
| `tuple_struct.rs` / `unit_struct.rs` に `#[test]` 追加 | 低 | 現在は `fn main()` のみで `running 0 tests` と表示される |

---

## 6. まとめ

マクロの基本設計は堅実で、ジェネリクス・ライフタイム・associated type を含む複雑なケースにも対応できている。テストも compile-pass / compile-fail / runtime の3層で整備されている。

最優先で対応すべきは **3.1 (`#[msgpack(default)]` が encode を抑制する問題)** で、これはデータロスにつながるサイレントなバグ。3.2, 3.3 は `Option<T>` + `bytes` の組み合わせで顕在化する問題で、該当パターンのテスト追加とともに修正が望ましい。

3.4 (`::std::boxed::Box`) はブランチの目的（alloc-support）を踏まえると対応が必要。
