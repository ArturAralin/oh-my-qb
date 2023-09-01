use std::borrow::Cow;

#[derive(Debug, Clone)]
pub enum Value<'a> {
    String(Cow<'a, str>),
    Integer(i32),
    BigInt(i64),
    Boolean(bool),
}

pub trait ValueExt<'a> {
    fn value(self) -> Value<'a>;
}

impl<'a> ValueExt<'a> for &'a str {
    fn value(self) -> Value<'a> {
        Value::String(Cow::Borrowed(self))
    }
}

impl<'a> ValueExt<'a> for i32 {
    fn value(self) -> Value<'a> {
        Value::Integer(self)
    }
}

impl<'a> ValueExt<'a> for String {
    fn value(self) -> Value<'a> {
        Value::String(Cow::Owned(self))
    }
}

impl<'a> ValueExt<'a> for i64 {
    fn value(self) -> Value<'a> {
        Value::BigInt(self)
    }
}

impl<'a> ValueExt<'a> for bool {
    fn value(self) -> Value<'a> {
        Value::Boolean(self)
    }
}
