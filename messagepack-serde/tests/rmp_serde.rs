use messagepack_serde::ser::{Exact, LosslessMinimize};
use rstest::rstest;
use serde::Serialize;
use std::collections::BTreeMap;

fn rmp_to_vec_struct_map<T: Serialize>(value: &T) -> Vec<u8> {
    let mut out = Vec::new();
    let mut ser = rmp_serde::Serializer::new(&mut out).with_struct_map();
    value.serialize(&mut ser).unwrap();
    out
}

fn assert_same_lossless<T: Serialize>(value: &T) {
    let rmp = rmp_to_vec_struct_map(value);
    let mut buf = vec![0u8; rmp.len()];
    let len = messagepack_serde::to_slice_with_config(value, &mut buf, LosslessMinimize).unwrap();
    assert_eq!(rmp, buf[..len]);
}

fn assert_same_exact<T: Serialize>(value: &T) {
    let rmp = rmp_to_vec_struct_map(value);
    let mut buf = vec![0u8; rmp.len()];
    let len = messagepack_serde::to_slice_with_config(value, &mut buf, Exact).unwrap();
    assert_eq!(rmp, buf[..len]);
}

#[test]
fn bool_nil_char() {
    assert_same_lossless(&true);
    assert_same_lossless(&false);
    // Note: rmp-serde encodes unit and unit struct as empty array (fixarray 0),
    // while messagepack_serde encodes as nil. Exclude from parity checks.
    assert_same_lossless(&Option::<u8>::None);
    assert_same_lossless(&'a');
    assert_same_lossless(&'😀');
}

#[test]
fn integers_boundaries() {
    // signed
    assert_same_lossless(&-1i64);
    assert_same_lossless(&-32i64); // neg fixint
    assert_same_lossless(&-33i64); // int8 boundary
    assert_same_lossless(&-128i64);
    assert_same_lossless(&-129i64);
    assert_same_lossless(&-32768i64);
    assert_same_lossless(&-32769i64);
    assert_same_lossless(&-2147483648i64);
    assert_same_lossless(&-2147483649i64);
    assert_same_lossless(&i64::MIN);
    assert_same_lossless(&i64::MAX);

    // unsigned
    assert_same_lossless(&0u64);
    assert_same_lossless(&1u64);
    assert_same_lossless(&127u64); // pos fixint
    assert_same_lossless(&128u64); // uint8
    assert_same_lossless(&255u64);
    assert_same_lossless(&256u64); // uint16
    assert_same_lossless(&65535u64);
    assert_same_lossless(&65536u64); // uint32
    assert_same_lossless(&4294967295u64);
    assert_same_lossless(&4294967296u64); // u64 boundary
    assert_same_lossless(&u64::MAX);
}

#[test]
fn floats_f32() {
    assert_same_lossless(&0.0f32);
    assert_same_lossless(&-0.0f32);
    assert_same_lossless(&1.5f32);
    assert_same_lossless(&f32::INFINITY);
    assert_same_lossless(&f32::NEG_INFINITY);
    assert_same_lossless(&f32::from_bits(0x7FC0_0000)); // canonical quiet NaN
}

#[test]
fn floats_f64_exact() {
    // rmp-serde encodes f64 as f64; use Exact to match
    assert_same_exact(&0.0f64);
    assert_same_exact(&-0.0f64);
    assert_same_exact(&1.5f64);
    assert_same_exact(&f64::INFINITY);
    assert_same_exact(&f64::NEG_INFINITY);
    assert_same_exact(&f64::from_bits(0x7FF8_0000_0000_0000)); // quiet NaN
}

#[rstest]
#[case(0usize)]
#[case(31)] // fixstr upper
#[case(32)] // str8
#[case(255)]
#[case(256)] // str16
#[case(1024)]
fn strings_boundaries(#[case] len: usize) {
    let s = "a".repeat(len);
    assert_same_lossless(&s);
}

#[rstest]
#[case(0usize)]
#[case(1)]
#[case(255)] // bin8 upper
#[case(256)] // bin16
#[case(1024)]
fn binary_boundaries(#[case] len: usize) {
    let data = vec![0u8; len];
    let bb = serde_bytes::ByteBuf::from(data);
    assert_same_lossless(&bb);
}

#[test]
fn arrays_simple() {
    let v0: Vec<i64> = vec![];
    let v15: Vec<i64> = (0..15).map(|i| i as i64).collect();
    let v16: Vec<i64> = (0..16).map(|i| i as i64).collect();
    assert_same_lossless(&v0);
    assert_same_lossless(&v15);
    assert_same_lossless(&v16);
}

#[test]
fn maps_btreemap_order_and_sizes() {
    let m0: BTreeMap<u32, u32> = BTreeMap::new();
    assert_same_lossless(&m0);

    let mut m1 = BTreeMap::new();
    m1.insert(1, 10);
    assert_same_lossless(&m1);

    let mut m15 = BTreeMap::new();
    for i in 0..15u32 { m15.insert(i, i+100); }
    assert_same_lossless(&m15);

    let mut m16 = BTreeMap::new();
    for i in 0..16u32 { m16.insert(i, i+100); }
    assert_same_lossless(&m16);
}

#[test]
fn structs_as_maps() {
    #[derive(Serialize)]
    struct Named {
        a: u8,
        b: Option<&'static str>,
    }
    #[derive(Serialize)]
    struct Tuple(u8, i16);
    #[derive(Serialize)]
    struct WithRename {
        #[serde(rename = "id")]
        ident: u32,
        #[serde(rename = "msg")]
        message: &'static str,
    }

    assert_same_lossless(&Named { a: 7, b: Some("hi") });
    assert_same_lossless(&Named { a: 7, b: None });
    assert_same_lossless(&Tuple(1, -2));
    assert_same_lossless(&WithRename { ident: 1, message: "ok" });
}

#[test]
fn enums_external_and_tagged() {
    #[derive(Serialize)]
    enum E {
        A,
        B(i32),
        C(i32, i32),
        D { x: u8, y: bool },
    }
    assert_same_lossless(&E::A);
    assert_same_lossless(&E::B(10));
    assert_same_lossless(&E::C(1, 2));
    assert_same_lossless(&E::D { x: 3, y: true });

    #[derive(Serialize)]
    #[serde(tag = "t", content = "c")]
    enum Adj {
        U,
        V(u8),
        W { k: &'static str },
    }
    assert_same_lossless(&Adj::U);
    assert_same_lossless(&Adj::V(5));
    assert_same_lossless(&Adj::W { k: "v" });

    #[derive(Serialize)]
    #[serde(tag = "type")]
    enum Internal {
        A,
        B { x: i64 },
    }
    assert_same_lossless(&Internal::A);
    assert_same_lossless(&Internal::B { x: 42 });
}
