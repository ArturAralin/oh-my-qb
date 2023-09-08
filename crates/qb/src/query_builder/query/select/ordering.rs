use crate::query_builder::{Arg, SqlKeyword, TryIntoArg};

#[derive(Debug, Clone)]
pub struct Ordering<'a> {
    pub left: Arg<'a>,
    pub right: Arg<'a>,
    // only for PG
    pub null_first: Option<bool>,
}

pub trait TryIntoOrdering<'a> {
    fn try_into_ordering(self) -> Result<Ordering<'a>, ()>;
}

impl<'a, T1: TryIntoArg<'a>, T2: TryIntoArg<'a>> TryIntoOrdering<'a> for (T1, T2) {
    fn try_into_ordering(self) -> Result<Ordering<'a>, ()> {
        Ok(Ordering {
            left: self.0.try_into_arg().unwrap(),
            right: self.1.try_into_arg().unwrap(),
            null_first: None,
        })
    }
}

impl<'a, T1: TryIntoArg<'a>, T2: TryIntoArg<'a>> TryIntoOrdering<'a> for (T1, T2, SqlKeyword) {
    fn try_into_ordering(self) -> Result<Ordering<'a>, ()> {
        Ok(Ordering {
            left: self.0.try_into_arg().unwrap(),
            right: self.1.try_into_arg().unwrap(),
            null_first: Some(matches!(self.2, SqlKeyword::NullsFirst)),
        })
    }
}
