use crate::{
    qb_arg::{Arg, Raw},
    value::Value,
};

#[derive(Debug)]
pub enum ConditionOp {
    And,
    Or,
}

#[derive(Debug)]
pub struct ConditionInner<'a> {
    pub op: ConditionOp,
    pub left: Arg<'a>,
    pub middle: String,
    pub right: Arg<'a>,
}

#[derive(Debug)]
pub enum Condition<'a> {
    Group(ConditionsGroup<'a>),
    Condition(ConditionInner<'a>),
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

#[derive(Debug)]
pub struct ConditionsGroup<'a> {
    op: ConditionOp,
    conditions: Vec<Condition<'a>>,
}

impl<'a> ConditionsGroup<'a> {
    pub fn new(op: ConditionOp) -> Self {
        Self {
            op,
            conditions: Default::default(),
        }
    }
}

pub trait Conditions<'a> {
    fn push_cond(&mut self, cond: Condition<'a>);
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

        self.push_cond(Condition::Condition(ConditionInner {
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

        self.push_cond(Condition::Condition(ConditionInner {
            op: ConditionOp::Or,
            left,
            middle: op.to_owned(),
            right,
        }));

        self
    }
}

// impl<'a> Conditions<'a> for ConditionsGroup<'a> {
//     fn push_cond(&mut self, cond: Condition<'a>) {
//         self.conditions.push(cond);
//     }

//     fn get_binding_idx(&self) -> usize {
//         0
//     }

//     fn push_bindings<I>(&mut self, values: I)
//     where
//         I: Iterator<Item = Value<'a>>,
//     {
//     }
// }
