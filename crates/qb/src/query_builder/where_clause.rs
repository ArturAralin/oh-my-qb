// todo: move to condition
use crate::Conditions;

use super::{conditions::ConditionOp, qb_arg::Arg, PushCondition, TryIntoArg};

#[derive(Debug, Clone)]
pub struct SingleWhereCondition<'a> {
    pub op: ConditionOp,
    pub left: Arg<'a>,
    pub middle: String,
    pub right: Arg<'a>,
}

#[derive(Debug, Clone)]
pub struct GroupedWhereCondition<'a> {
    pub op: ConditionOp,
    pub conditions: Vec<WhereCondition<'a>>,
}

#[derive(Debug, Clone)]
pub enum WhereCondition<'a> {
    Group(GroupedWhereCondition<'a>),
    Single(SingleWhereCondition<'a>),
}

impl<'a> WhereCondition<'a> {
    pub fn set_op(&mut self, op: ConditionOp) {
        match self {
            Self::Group(cond) => cond.op = op,
            Self::Single(cond) => cond.op = op,
        };
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

impl<'a> PushCondition<'a> for GroupedWhereCondition<'a> {
    fn push_cond(&mut self, cond: WhereCondition<'a>) {
        self.conditions.push(cond);
    }
}

impl<'a> Conditions<'a> for GroupedWhereCondition<'a> {}

pub trait TryIntoCondition<'a> {
    fn try_into_condition(self) -> Result<WhereCondition<'a>, ()>;
}

impl<'a, T1: TryIntoArg<'a>, T2: TryIntoArg<'a>> TryIntoCondition<'a> for (T1, T2) {
    fn try_into_condition(self) -> Result<WhereCondition<'a>, ()> {
        Ok(WhereCondition::Single(SingleWhereCondition {
            op: ConditionOp::And,
            left: self.0.try_into_arg().unwrap(),
            middle: "=".to_owned(),
            right: self.1.try_into_arg().unwrap(),
        }))
    }
}

impl<'a, T1: TryIntoArg<'a>, T2: TryIntoArg<'a>> TryIntoCondition<'a> for (T1, &'a str, T2) {
    fn try_into_condition(self) -> Result<WhereCondition<'a>, ()> {
        Ok(WhereCondition::Single(SingleWhereCondition {
            op: ConditionOp::And,
            left: self.0.try_into_arg().unwrap(),
            middle: self.1.to_owned(),
            right: self.2.try_into_arg().unwrap(),
        }))
    }
}
