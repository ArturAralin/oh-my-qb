use crate::{
    query_builder::{Value, WhereCondition},
    sql_dialect::{Sql, SqlDialect},
};
use std::borrow::Cow;

#[derive(Debug, Default, Clone)]
pub struct UpdateQuery<'a> {
    pub table: Option<Cow<'a, str>>,
    pub columns: Vec<Cow<'a, str>>,
    pub values: Vec<Value<'a>>,
    pub where_clause: Vec<WhereCondition<'a>>,
}

impl<'a> UpdateQuery<'a> {
    pub fn table(&mut self, table: &'a str) -> &mut Self {
        self.table = Some(Cow::Borrowed(table));
        self
    }

    pub fn sql<D>(&'a self) -> Sql<'a>
    where
        D: SqlDialect<'a>,
    {
        let mut builder = D::init();

        builder.build_update(self);

        builder.sql()
    }

    pub fn sqlx_qb<D: SqlDialect<'a>>(&'a self) -> D::SqlxQb {
        let mut builder = D::init();

        builder.build_update(self);

        builder.into_sqlx_qb()
    }
}
