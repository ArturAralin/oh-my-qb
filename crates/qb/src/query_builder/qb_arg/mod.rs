pub mod raw;
pub mod subquery;

use super::value::Value;
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct Relation<'a>(pub Cow<'a, str>);

impl<'a> From<&'a str> for Relation<'a> {
    fn from(value: &'a str) -> Self {
        Relation(Cow::Borrowed(value))
    }
}

#[derive(Debug, Clone)]
pub enum ArgValue<'a> {
    Value(Value<'a>),
    Values(Vec<Value<'a>>),
}

#[derive(Debug, Clone)]
pub enum SqlKeyword {
    Asc,
    Desc,
    // only for PG
    NullsFirst,
    NullsLast,
}

#[derive(Debug, Clone)]
pub enum Arg<'a> {
    Relation(Relation<'a>),
    Value(ArgValue<'a>),
    Raw(raw::Raw<'a>),
    SubQuery(subquery::SubQuery<'a>),
    Keyword(SqlKeyword),
}

pub trait TryIntoArg<'a>: Sized {
    type E: std::error::Error;

    fn try_into_arg(self) -> Result<Arg<'a>, Self::E>;
}

impl<'a> TryIntoArg<'a> for &'a str {
    type E = crate::error::Error;

    fn try_into_arg(self) -> Result<Arg<'a>, Self::E> {
        Ok(Arg::Relation(Relation(Cow::Borrowed(self))))
    }
}

impl<'a> TryIntoArg<'a> for Vec<Value<'a>> {
    type E = crate::error::Error;

    fn try_into_arg(self) -> Result<Arg<'a>, Self::E> {
        Ok(Arg::Value(ArgValue::Values(self)))
    }
}

impl<'a> TryIntoArg<'a> for SqlKeyword {
    type E = crate::error::Error;

    fn try_into_arg(self) -> Result<Arg<'a>, Self::E> {
        Ok(Arg::Keyword(self))
    }
}
