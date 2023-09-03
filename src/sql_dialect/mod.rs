pub mod postgres;

use crate::query_builder::{QueryBuilder, Value};
use sqlx::Arguments;

#[derive(Debug)]
pub enum Dialect {
    Postgres,
}

pub struct Sql<'a> {
    pub sql: String,
    pub bindings: Vec<&'a Value<'a>>,

    // todo: remove it?
    pub dialect: Dialect,
}

impl<'a> Sql<'a> {
    // feature postgres and sqlx
    pub fn into_sqlx_qb(self) -> sqlx::QueryBuilder<'a, sqlx::Postgres> {
        let mut args = sqlx::postgres::PgArguments::default();

        self.bindings.into_iter().for_each(|binding| match binding {
            Value::Integer(v) => args.add(v),
            Value::BigInt(v) => args.add(v),
            // todo: check it
            Value::Null => args.add::<Option<i32>>(None),
            Value::String(s) => args.add(s),
            _ => panic!("unsuppoted"),
        });

        sqlx::QueryBuilder::with_arguments(self.sql, args)
    }
}

pub trait BuildSql<'a> {
    fn init() -> Self;
    fn dialect() -> Dialect;
    fn build_sql(&mut self, qb: &'a QueryBuilder<'a>);
    fn sql(self) -> Sql<'a>;
}
