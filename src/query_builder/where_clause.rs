use super::{conditions::ConditionOp, qb_arg::Arg};

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

impl<'a> GroupedWhereCondition<'a> {
    pub fn new(op: ConditionOp) -> Self {
        Self {
            op,
            conditions: Default::default(),
        }
    }
}
