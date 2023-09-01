use super::value::Value;
use std::{borrow::Cow, vec};

#[derive(Debug)]
pub struct Column<'a>(pub Cow<'a, str>);

#[derive(Debug)]
pub struct Raw<'a> {
    pub sql: Cow<'a, str>,
}

#[derive(Debug)]
pub enum ArgValue<'a> {
    Value(Value<'a>),
    Values(Vec<Value<'a>>),
    Binding((usize, usize)),
}

impl<'a> ArgValue<'a> {
    pub fn binding(&mut self, start_idx: usize) -> Vec<Value<'a>> {
        let count = match self {
            Self::Value(_) => 1,
            Self::Values(values) => values.len(),
            _ => {
                unreachable!("cant be replaced twice")
            }
        };

        match std::mem::replace(self, ArgValue::Binding((start_idx, start_idx + count))) {
            Self::Value(value) => vec![value],
            Self::Values(values) => values,
            _ => {
                unreachable!("cant be replaced twice")
            }
        }
    }
}

#[derive(Debug)]
pub enum Arg<'a> {
    Column(Column<'a>),
    Value(ArgValue<'a>),
    Raw(Raw<'a>),
}

impl<'a> Arg<'a> {
    pub fn bindings(&mut self, idx: usize) -> Vec<Value<'a>> {
        if let Self::Value(v) = self {
            v.binding(idx)
        } else {
            vec![]
        }
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
