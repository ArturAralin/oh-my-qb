use std::cell::RefCell;
use std::rc::Rc;

use super::value::Value;
use super::GroupedWhereCondition;
use super::{
    qb_arg::Arg,
    where_clause::{SingleWhereCondition, WhereCondition},
};

#[derive(Debug)]
pub enum ConditionOp {
    And,
    Or,
}

pub struct GroupBuilder<'a> {
    bindings: Rc<RefCell<Vec<Value<'a>>>>,
    group: GroupedWhereCondition<'a>,
}

pub trait Conditions<'a> {
    // todo: move to another trait ConditionsInternal
    fn push_cond(&mut self, cond: WhereCondition<'a>);
    fn get_bindings(&self) -> Rc<RefCell<Vec<Value<'a>>>>;

    fn and_where<L: Into<Arg<'a>>, R: Into<Arg<'a>>>(
        &mut self,
        left: L,
        op: &str,
        right: R,
    ) -> &mut Self {
        let bindings = self.get_bindings();
        let mut offset = { bindings.as_ref().borrow().len() };
        let mut bindings = bindings.as_ref().borrow_mut();

        let mut left: Arg<'a> = left.into();

        let left_values = left.bindings(offset);

        offset += left_values.len();

        let mut right: Arg<'a> = right.into();

        bindings.extend(right.bindings(offset).into_iter());

        self.push_cond(WhereCondition::Single(SingleWhereCondition {
            op: ConditionOp::And,
            left,
            middle: op.to_owned(),
            right,
        }));

        self
    }

    fn or_where<L: Into<Arg<'a>>, R: Into<Arg<'a>>>(
        &mut self,
        left: L,
        op: &str,
        right: R,
    ) -> &mut Self {
        let bindings = self.get_bindings();
        let mut offset = { bindings.as_ref().borrow().len() };
        let mut bindings = bindings.as_ref().borrow_mut();

        let mut left: Arg<'a> = left.into();

        let left_values = left.bindings(offset);

        offset += left_values.len();

        bindings.extend(left_values.into_iter());

        let mut right: Arg<'a> = right.into();

        bindings.extend(right.bindings(offset).into_iter());

        self.push_cond(WhereCondition::Single(SingleWhereCondition {
            op: ConditionOp::Or,
            left,
            middle: op.to_owned(),
            right,
        }));

        self
    }

    fn and_where_grouped<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut GroupBuilder<'a>),
    {
        let mut group_builder = GroupBuilder {
            group: GroupedWhereCondition {
                op: ConditionOp::And,
                conditions: vec![],
            },
            bindings: self.get_bindings(),
        };

        f(&mut group_builder);

        self.push_cond(WhereCondition::Group(group_builder.group));

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
            bindings: self.get_bindings(),
        };

        f(&mut group_builder);

        self.push_cond(WhereCondition::Group(group_builder.group));

        self
    }
}

impl<'a> Conditions<'a> for GroupBuilder<'a> {
    fn get_bindings(&self) -> Rc<RefCell<Vec<Value<'a>>>> {
        Rc::clone(&self.bindings)
    }

    fn push_cond(&mut self, cond: WhereCondition<'a>) {
        self.group.conditions.push(cond);
    }
}
