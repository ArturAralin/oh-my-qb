use std::borrow::Cow;

use crate::query_builder::Arg;

#[derive(Debug, Clone)]
pub struct RegularJoin<'a> {
    pub join_type: Option<&'static str>,
    pub table: Cow<'a, str>,
    pub left: Arg<'a>,
    pub op: Cow<'a, str>,
    pub right: Arg<'a>,
}

#[derive(Debug, Clone)]
pub enum Join<'a> {
    Regular(RegularJoin<'a>),
    // Raw
}
