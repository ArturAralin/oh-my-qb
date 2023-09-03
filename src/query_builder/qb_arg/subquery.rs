use crate::QueryBuilder;

#[derive(Debug, Clone)]
pub struct SubQuery<'a>(pub QueryBuilder<'a>);
