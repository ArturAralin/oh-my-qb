use crate::query_builder::WhereCondition;
use std::borrow::Cow;

#[derive(Debug, Default, Clone)]
pub struct SelectQuery<'a> {
    pub columns: Option<Vec<Cow<'a, str>>>,
    pub table: Option<Cow<'a, str>>,
    pub where_clause: Vec<WhereCondition<'a>>,
}
