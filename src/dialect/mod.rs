pub mod postgres;
use crate::query_builder::{QueryBuilder, Value};

#[derive(Debug)]
pub struct Sql<'a> {
    pub sql: String,
    pub binds: Vec<&'a Value<'a>>,
}

pub trait BuildSql<'a> {
    fn init() -> Self;
    fn build_sql(self, qb: &'a QueryBuilder) -> Sql<'a>;
}
