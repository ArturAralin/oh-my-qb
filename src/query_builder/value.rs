use std::borrow::Cow;

#[derive(Debug, Clone)]
pub enum Value<'a> {
    String(Cow<'a, str>),
    Integer(i32),
    BigInt(i64),
    Boolean(bool),
    Null,
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

impl<'a, T: Into<Value<'a>>> ValueExt<'a> for Option<T> {
    fn value(self) -> Value<'a> {
        match self {
            Some(value) => value.into(),
            None => Value::Null,
        }
    }
}

impl<'a> From<i32> for Value<'a> {
    fn from(value: i32) -> Self {
        Self::Integer(value)
    }
}

impl<'a> From<i64> for Value<'a> {
    fn from(value: i64) -> Self {
        Self::BigInt(value)
    }
}

impl<'a> From<&'a str> for Value<'a> {
    fn from(value: &'a str) -> Self {
        Self::String(Cow::Borrowed(value))
    }
}

impl<'a> From<String> for Value<'a> {
    fn from(value: String) -> Self {
        Self::String(Cow::Owned(value))
    }
}

impl<'a, T: Into<Value<'a>>> From<Option<T>> for Value<'a> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => value.into(),
            None => Value::Null,
        }
    }
}
