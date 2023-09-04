use crate::query_builder::WhereCondition;
use std::borrow::Cow;

use super::join::Join;

#[derive(Debug, Default, Clone)]
pub struct SelectQuery<'a> {
    pub columns: Option<Vec<Cow<'a, str>>>,
    pub table: Option<Cow<'a, str>>,
    pub joins: Option<Vec<Join<'a>>>,
    pub where_clause: Vec<WhereCondition<'a>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub alias: Option<Cow<'a, str>>,
}
