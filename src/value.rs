use std::borrow::Cow;

#[derive(Debug, Clone)]
pub enum Value<'a> {
    Str(Cow<'a, str>),
    I32(i32),
}

pub trait ValueExt<'a> {
    fn value(self) -> Value<'a>;
}

impl<'a> ValueExt<'a> for &'a str {
    fn value(self) -> Value<'a> {
        Value::Str(Cow::Borrowed(self))
    }
}

impl<'a> ValueExt<'a> for i32 {
    fn value(self) -> Value<'a> {
        Value::I32(self)
    }
}

impl<'a> ValueExt<'a> for String {
    fn value(self) -> Value<'a> {
        Value::Str(Cow::Owned(self))
    }
}
