use std::borrow::Cow;

use crate::query_builder::subquery::SubQuery;
use crate::{
    query_builder::{Arg, Row, RowBuilder, Value},
    sql_dialect::{Sql, SqlDialect},
};

#[derive(Debug, Clone)]
pub struct InsertWithValues {}

#[derive(Debug, Clone)]
pub enum InsertType<'a> {
    WithValues(InsertWithValues),
    FromSubQuery(SubQuery<'a>),
}

#[derive(Debug, Default, Clone)]
pub struct InsertQuery<'a> {
    pub inner: InsertType<'a>,
    pub table: Option<Cow<'a, str>>,
    pub ordered_columns: Option<&'static [&'static str]>,
    pub bindings: Vec<Value<'a>>,
}

impl<'a> InsertQuery<'a> {
    pub fn into_(&mut self, table: &'a str) -> &mut Self {
        self.table = Some(Cow::Borrowed(table));

        self
    }

    pub fn value<R: Row<'a>>(&mut self, row: R) -> &mut Self {
        let mut builder = RowBuilder::default();

        row.into_row(&mut builder);

        self.ordered_columns = Some(R::columns());
        self.bindings.extend(builder.values);

        self
    }

    pub fn values<R: Row<'a>>(&mut self, rows: impl IntoIterator<Item = R>) -> &mut Self {
        self.ordered_columns = Some(R::columns());

        for row in rows.into_iter() {
            let mut builder = RowBuilder::default();
            row.into_row(&mut builder);

            self.bindings.extend(builder.values);
        }

        self
    }

    pub fn sql<D>(&'a self) -> Sql<'a>
    where
        D: SqlDialect<'a>,
    {
        let mut builder = D::init();

        builder.build_insert(self);

        builder.sql()
    }

    pub fn sqlx_qb<D: SqlDialect<'a>>(&'a self) -> D::SqlxQb {
        let mut builder = D::init();

        builder.build_insert(self);

        builder.into_sqlx_qb()
    }
}
