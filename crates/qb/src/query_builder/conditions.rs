use super::value::Value;
use super::{qb_arg::Arg, TryIntoArg};

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

#[derive(Debug, Clone)]
pub enum ConditionOp {
    And,
    Or,
}

pub trait PushCondition<'a> {
    fn push_cond(&mut self, cond: WhereCondition<'a>);
}

pub trait Conditions<'a>: PushCondition<'a> {
    fn and_where(&mut self, condition: impl TryIntoCondition<'a>) -> &mut Self {
        let condition = condition.try_into_condition().unwrap();
        self.push_cond(condition);

        self
    }

    fn or_where(&mut self, condition: impl TryIntoCondition<'a>) -> &mut Self {
        let mut condition = condition.try_into_condition().unwrap();

        condition.set_op(ConditionOp::Or);

        self.push_cond(condition);

        self
    }

    fn and_where_grouped<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut GroupedWhereCondition<'a>),
    {
        let mut condition = GroupedWhereCondition {
            op: ConditionOp::And,
            conditions: vec![],
        };

        f(&mut condition);

        self.push_cond(WhereCondition::Group(condition));

        self
    }

    fn or_where_grouped<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut GroupedWhereCondition<'a>),
    {
        let mut condition = GroupedWhereCondition {
            op: ConditionOp::Or,
            conditions: vec![],
        };

        f(&mut condition);

        self.push_cond(WhereCondition::Group(condition));

        self
    }

    fn and_where_null<L: TryIntoArg<'a>>(&mut self, left: L) -> &mut Self {
        self.push_cond(WhereCondition::Single(SingleWhereCondition {
            op: ConditionOp::And,
            left: left.try_into_arg().unwrap(),
            middle: "is".to_owned(),
            right: Arg::Value(super::ArgValue::Value(Value::Null)),
        }));

        self
    }

    fn or_where_null<L: TryIntoArg<'a>>(&mut self, left: L) -> &mut Self {
        self.push_cond(WhereCondition::Single(SingleWhereCondition {
            op: ConditionOp::Or,
            left: <L as TryIntoArg>::try_into_arg(left).unwrap(),
            middle: "is".to_owned(),
            right: Arg::Value(super::ArgValue::Value(Value::Null)),
        }));

        self
    }

    fn and_where_not_null<L: TryIntoArg<'a>>(&mut self, left: L) -> &mut Self {
        self.push_cond(WhereCondition::Single(SingleWhereCondition {
            op: ConditionOp::And,
            left: <L as TryIntoArg>::try_into_arg(left).unwrap(),
            middle: "is not".to_owned(),
            right: Arg::Value(super::ArgValue::Value(Value::Null)),
        }));

        self
    }

    fn or_where_not_null<L: TryIntoArg<'a>>(&mut self, left: L) -> &mut Self {
        self.push_cond(WhereCondition::Single(SingleWhereCondition {
            op: ConditionOp::Or,
            left: <L as TryIntoArg>::try_into_arg(left).unwrap(),
            middle: "is not".to_owned(),
            right: Arg::Value(super::ArgValue::Value(Value::Null)),
        }));

        self
    }
}
