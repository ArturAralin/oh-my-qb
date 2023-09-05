use std::borrow::Cow;

use crate::{
    query_builder::{PushCondition, WhereCondition},
    sql_dialect::{Sql, SqlDialect},
    Conditions,
};

#[derive(Debug, Default, Clone)]
pub struct DeleteQuery<'a> {
    pub table: Option<Cow<'a, str>>,
    pub where_clause: Vec<WhereCondition<'a>>,
}

impl<'a> DeleteQuery<'a> {
    pub fn from(&mut self, table: &'a str) -> &mut Self {
        self.table = Some(Cow::Borrowed(table));
        self
    }

    pub fn sql<D>(&'a self) -> Sql<'a>
    where
        D: SqlDialect<'a>,
    {
        let mut builder = D::init();

        builder.build_delete(self);

        builder.sql()
    }

    pub fn sqlx_qb<D: SqlDialect<'a>>(&'a self) -> D::SqlxQb {
        let mut builder = D::init();

        builder.build_delete(self);

        builder.into_sqlx_qb()
    }
}

impl<'a> PushCondition<'a> for DeleteQuery<'a> {
    fn push_cond(&mut self, cond: WhereCondition<'a>) {
        self.where_clause.push(cond);
    }
}

impl<'a> Conditions<'a> for DeleteQuery<'a> {}
