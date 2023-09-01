use super::{
    conditions::ConditionOp,
    qb_arg::{Arg, Raw},
};

#[derive(Debug)]
pub struct SingleWhereCondition<'a> {
    pub op: ConditionOp,
    pub left: Arg<'a>,
    pub middle: String,
    pub right: Arg<'a>,
}

#[derive(Debug)]
pub struct GroupedWhereCondition<'a> {
    pub op: ConditionOp,
    pub conditions: Vec<WhereCondition<'a>>,
}

#[derive(Debug)]
pub enum WhereCondition<'a> {
    Group(GroupedWhereCondition<'a>),
    Single(SingleWhereCondition<'a>),
}

pub trait RawExt<'a> {
    fn raw(self) -> Raw<'a>;
}

impl<'a> RawExt<'a> for &'a str {
    fn raw(self) -> Raw<'a> {
        Raw {
            sql: std::borrow::Cow::Borrowed(self),
        }
    }
}

impl<'a> GroupedWhereCondition<'a> {
    pub fn new(op: ConditionOp) -> Self {
        Self {
            op,
            conditions: Default::default(),
        }
    }
}
