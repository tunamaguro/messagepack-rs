pub mod extension;
pub mod number;

pub enum ValueRef<'a> {
    Nil,
    Bool(bool),
    Bin(&'a [u8]),
    Extension,
    Number(number::Number),
    String(&'a str),
    Array(&'a [ValueRef<'a>]),
    Map(&'a [(ValueRef<'a>, ValueRef<'a>)]),
}
