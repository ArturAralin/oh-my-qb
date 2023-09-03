mod raw;
mod subquery;

use self::subquery::SubQuery;
use super::value::Value;
use crate::QueryBuilder;
pub use raw::*;
use std::borrow::Cow;
pub use subquery::*;

#[derive(Debug, Clone)]
pub struct Column<'a>(pub Cow<'a, str>);

#[derive(Debug, Clone)]
pub enum ArgValue<'a> {
    Value(Value<'a>),
    Values(Vec<Value<'a>>),
    Binding((usize, usize)),
}

#[derive(Debug, Clone)]
pub enum Arg<'a> {
    Column(Column<'a>),
    Value(ArgValue<'a>),
    Raw(Raw<'a>),
    SubQuery(SubQuery<'a>),
}

impl<'a> From<Value<'a>> for Arg<'a> {
    fn from(value: Value<'a>) -> Self {
        Self::Value(ArgValue::Value(value))
    }
}

impl<'a> From<Raw<'a>> for Arg<'a> {
    fn from(value: Raw<'a>) -> Self {
        Self::Raw(value)
    }
}

impl<'a> From<&'a str> for Arg<'a> {
    fn from(value: &'a str) -> Self {
        Self::Column(Column(Cow::Borrowed(value)))
    }
}

impl<'a> From<Vec<Value<'a>>> for Arg<'a> {
    fn from(value: Vec<Value<'a>>) -> Self {
        Self::Value(ArgValue::Values(value))
    }
}

impl<'a> From<QueryBuilder<'a>> for Arg<'a> {
    fn from(value: QueryBuilder<'a>) -> Self {
        Self::SubQuery(SubQuery(value))
    }
}
