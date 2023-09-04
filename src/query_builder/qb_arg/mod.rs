mod raw;
mod subquery;

use self::subquery::SubQuery;
use super::value::Value;
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

pub trait TryIntoArg<'a>: Sized {
    type E: std::error::Error;

    fn try_into_arg(value: Self) -> Result<Arg<'a>, Self::E>;
}

impl<'a> TryIntoArg<'a> for &'a str {
    type E = crate::error::Error;

    fn try_into_arg(value: Self) -> Result<Arg<'a>, Self::E> {
        Ok(Arg::Column(Column(Cow::Borrowed(value))))
    }
}

impl<'a> TryIntoArg<'a> for Vec<Value<'a>> {
    type E = crate::error::Error;

    fn try_into_arg(value: Self) -> Result<Arg<'a>, Self::E> {
        Ok(Arg::Value(ArgValue::Values(value)))
    }
}
