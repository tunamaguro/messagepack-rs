use messagepack_core::extension::ExtensionOwned;
use messagepack_serde::{Value, from_slice, to_vec, value::Number};
use proptest::prelude::*;

fn arb_number() -> impl Strategy<Value = Number> {
    prop_oneof![
        any::<u64>().prop_map(Number::from),
        any::<i64>().prop_map(Number::from),
        any::<f64>().prop_map(Number::from),
    ]
}

fn arb_extension() -> impl Strategy<Value = ExtensionOwned> {
    (any::<i8>(), any::<Vec<u8>>()).prop_map(|(ext_type, data)| ExtensionOwned::new(ext_type, data))
}

fn arb_value() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Nil),
        any::<bool>().prop_map(Value::Bool),
        any::<Vec<u8>>().prop_map(Value::Bin),
        arb_extension().prop_map(Value::Extension),
        arb_number().prop_map(Value::Number),
        any::<String>().prop_map(Value::String),
    ];

    leaf.prop_recursive(8, 64, 8, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..=16).prop_map(Value::Array),
            prop::collection::vec((inner.clone(), inner.clone()), 0..=16).prop_map(Value::Map)
        ]
    })
}

fn number_eq(a: &Number, b: &Number) -> bool {
    match (a, b) {
        (Number::PositiveInt(x), Number::PositiveInt(y)) => x == y,
        (Number::NegativeInt(x), Number::NegativeInt(y)) => x == y,
        (Number::Float(x), Number::Float(y)) => (x.is_nan() && y.is_nan()) || (x == y),
        _ => false,
    }
}

fn value_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Nil, Value::Nil) => true,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Bin(x), Value::Bin(y)) => x == y,
        (Value::Extension(x), Value::Extension(y)) => x == y,
        (Value::Number(x), Value::Number(y)) => number_eq(x, y),
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Array(xs), Value::Array(ys)) => {
            xs.len() == ys.len() && xs.iter().zip(ys.iter()).all(|(x, y)| value_eq(x, y))
        }
        (Value::Map(xs), Value::Map(ys)) => {
            xs.len() == ys.len()
                && xs
                    .iter()
                    .zip(ys.iter())
                    .all(|((kx, vx), (ky, vy))| value_eq(kx, ky) && value_eq(vx, vy))
        }

        _ => false,
    }
}

proptest! {
    #[test]
    fn roundtrip_value(x in arb_value()) {
        let buf = to_vec(&x).unwrap();
        let y:Value = from_slice(buf.as_slice()).unwrap();

        assert!(value_eq(&x, &y))
    }
}
