use crate::query_builder::{Arg, Relation, TryIntoArg};
use std::borrow::Cow;

use super::SelectQuery;

#[derive(Debug, Clone)]
pub struct Column<'a> {
    pub arg: Arg<'a>,
    pub alias: Option<Cow<'a, str>>,
}

pub trait ColumnExt<'a> {
    fn alias(self, alias: &'a str) -> Column<'a>;
}

impl<'a> ColumnExt<'a> for &'a str {
    fn alias(self, alias: &'a str) -> Column<'a> {
        Column {
            arg: Arg::Relation(Relation(Cow::Borrowed(self))),
            alias: Some(Cow::Borrowed(alias)),
        }
    }
}

pub trait TryIntoColumn<'a> {
    fn try_into_column(self) -> Result<Column<'a>, ()>;
}

impl<'a> TryIntoColumn<'a> for &'a str {
    fn try_into_column(self) -> Result<Column<'a>, ()> {
        Ok(Column {
            arg: self.try_into_arg().unwrap(),
            alias: None,
        })
    }
}

impl<'a> TryIntoColumn<'a> for &'a &'a str {
    fn try_into_column(self) -> Result<Column<'a>, ()> {
        Ok(Column {
            arg: (*self).try_into_arg().unwrap(),
            alias: None,
        })
    }
}


impl<'a> TryIntoColumn<'a> for SelectQuery<'a> {
    fn try_into_column(self) -> Result<Column<'a>, ()> {
        Ok(Column {
            arg: self.try_into_arg().unwrap(),
            alias: None,
        })
    }
}

impl<'a> TryIntoColumn<'a> for Column<'a> {
    fn try_into_column(self) -> Result<Column<'a>, ()> {
        Ok(self)
    }
}
