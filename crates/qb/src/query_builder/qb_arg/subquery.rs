use crate::query_builder::SelectQuery;

use super::{Arg, TryIntoArg};

#[derive(Debug, Clone)]
pub struct SubQuery<'a>(pub SelectQuery<'a>);

impl<'a> TryIntoArg<'a> for SelectQuery<'a> {
    type E = crate::error::Error;

    fn try_into_arg(self) -> Result<Arg<'a>, Self::E> {
        Ok(Arg::SubQuery(SubQuery(self)))
    }
}
