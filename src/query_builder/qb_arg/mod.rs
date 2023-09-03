mod raw;
mod subquery;

use self::subquery::SubQuery;
use super::value::Value;
use crate::QueryBuilder;
pub use raw::*;
use std::{borrow::Cow, vec};
pub use subquery::*;

#[derive(Debug, Clone)]
pub struct Column<'a>(pub Cow<'a, str>);

#[derive(Debug, Clone)]
pub enum ArgValue<'a> {
    Value(Value<'a>),
    Values(Vec<Value<'a>>),
    Binding((usize, usize)),
}

impl<'a> ArgValue<'a> {
    pub fn binding(&mut self, start_idx: usize) -> Vec<Value<'a>> {
        // let count = match self {
        //     Self::Value(_) => 1,
        //     Self::Values(values) => values.len(),
        //     _ => {
        //         unreachable!("cant be replaced twice")
        //     }
        // };

        // match std::mem::replace(self, ArgValue::Binding((start_idx, start_idx + count))) {
        //     Self::Value(value) => vec![value],
        //     Self::Values(values) => values,
        //     _ => {
        //         unreachable!("cant be replaced twice")
        //     }
        // }

        vec![]
    }
}

#[derive(Debug, Clone)]
pub enum Arg<'a> {
    Column(Column<'a>),
    Value(ArgValue<'a>),
    Raw(Raw<'a>),
    SubQuery(SubQuery<'a>),
}

impl<'a> Arg<'a> {
    pub fn bindings(&mut self, start_idx: usize) -> Vec<Value<'a>> {
        // match self {
        //     Self::Value(v) => v.binding(start_idx),
        //     Self::Raw(r) => r.binding(start_idx).unwrap_or_default(),
        //     _ => vec![],
        // }

        vec![]
    }
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
