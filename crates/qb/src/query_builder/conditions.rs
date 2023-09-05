use super::value::Value;
use super::{
    qb_arg::Arg,
    where_clause::{SingleWhereCondition, WhereCondition},
};
use super::{GroupedWhereCondition, TryIntoArg};

#[derive(Debug, Clone)]
pub enum ConditionOp {
    And,
    Or,
}

pub struct GroupBuilder<'a> {
    // bindings: Rc<RefCell<Vec<Value<'a>>>>,
    group: GroupedWhereCondition<'a>,
}

pub trait PushCondition<'a> {
    fn push_cond(&mut self, cond: WhereCondition<'a>);
}

pub trait Conditions<'a>: PushCondition<'a> {
    // todo: move to another trait ConditionsInternal

    // fn get_bindings(&self) -> Rc<RefCell<Vec<Value<'a>>>>;

    fn and_where<L: TryIntoArg<'a>, R: TryIntoArg<'a>>(
        &mut self,
        left: L,
        op: &str,
        right: R,
    ) -> &mut Self {
        self.push_cond(WhereCondition::Single(SingleWhereCondition {
            op: ConditionOp::And,
            left: <L as TryIntoArg>::try_into_arg(left).unwrap(),
            middle: op.to_owned(),
            right: <R as TryIntoArg>::try_into_arg(right).unwrap(),
        }));

        self
    }

    fn or_where<L: TryIntoArg<'a>, R: TryIntoArg<'a>>(
        &mut self,
        left: L,
        op: &str,
        right: R,
    ) -> &mut Self {
        self.push_cond(WhereCondition::Single(SingleWhereCondition {
            op: ConditionOp::Or,
            left: <L as TryIntoArg>::try_into_arg(left).unwrap(),
            middle: op.to_owned(),
            right: <R as TryIntoArg>::try_into_arg(right).unwrap(),
        }));

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
        F: FnOnce(&mut GroupBuilder<'a>),
    {
        let mut group_builder = GroupBuilder {
            group: GroupedWhereCondition {
                op: ConditionOp::Or,
                conditions: vec![],
            },
            // bindings: self.get_bindings(),
        };

        f(&mut group_builder);

        self.push_cond(WhereCondition::Group(group_builder.group));

        self
    }

    fn and_where_null<L: TryIntoArg<'a>>(&mut self, left: L) -> &mut Self {
        self.push_cond(WhereCondition::Single(SingleWhereCondition {
            op: ConditionOp::And,
            left: <L as TryIntoArg>::try_into_arg(left).unwrap(),
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
    // fn get_bindings(&self) -> Rc<RefCell<Vec<Value<'a>>>> {
    //     Rc::clone(&self.bindings)
    // }

    fn push_cond(&mut self, cond: WhereCondition<'a>) {
        self.group.conditions.push(cond);
    }
}
