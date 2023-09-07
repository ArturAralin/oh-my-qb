use super::value::Value;
use super::{
    qb_arg::Arg,
    where_clause::{SingleWhereCondition, WhereCondition},
};
use super::{GroupedWhereCondition, TryIntoArg, TryIntoCondition};

#[derive(Debug, Clone)]
pub enum ConditionOp {
    And,
    Or,
}

pub struct GroupBuilder<'a> {
    group: GroupedWhereCondition<'a>,
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

impl<'a> PushCondition<'a> for GroupBuilder<'a> {
    fn push_cond(&mut self, cond: WhereCondition<'a>) {
        self.group.conditions.push(cond);
    }
}
