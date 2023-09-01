use super::value::Value;
use super::{
    qb_arg::Arg,
    where_clause::{SingleWhereCondition, WhereCondition},
};

#[derive(Debug)]
pub enum ConditionOp {
    And,
    Or,
}

pub trait Conditions<'a> {
    fn push_cond(&mut self, cond: WhereCondition<'a>);
    fn push_bindings<I>(&mut self, values: I)
    where
        I: Iterator<Item = Value<'a>>;
    fn get_binding_idx(&self) -> usize;

    fn and_where<L: Into<Arg<'a>>, R: Into<Arg<'a>>>(
        &mut self,
        left: L,
        op: &str,
        right: R,
    ) -> &mut Self {
        let mut left: Arg<'a> = left.into();

        self.push_bindings(left.bindings(self.get_binding_idx()).into_iter());

        let mut right: Arg<'a> = right.into();

        self.push_bindings(right.bindings(self.get_binding_idx()).into_iter());

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
        let mut left: Arg<'a> = left.into();

        self.push_bindings(left.bindings(self.get_binding_idx()).into_iter());

        let mut right: Arg<'a> = right.into();

        self.push_bindings(right.bindings(self.get_binding_idx()).into_iter());

        self.push_cond(WhereCondition::Single(SingleWhereCondition {
            op: ConditionOp::Or,
            left,
            middle: op.to_owned(),
            right,
        }));

        self
    }
}
