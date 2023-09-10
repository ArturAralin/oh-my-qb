use std::borrow::Cow;

use crate::query_builder::subquery::SubQuery;
use crate::{
    query_builder::{Row, RowBuilder, Value},
    sql_dialect::{Sql, SqlDialect},
};

#[derive(Debug, Default, Clone)]
pub struct InsertWithValues<'a> {
    pub bindings: Vec<Value<'a>>,
}

#[derive(Debug, Clone)]
pub struct InsertFromSubQuery<'a> {
    pub sub_query: SubQuery<'a>,
}

#[derive(Debug, Clone)]
pub enum InsertType<'a> {
    WithValues(InsertWithValues<'a>),
    FromSubQuery(InsertFromSubQuery<'a>),
}

impl<'a> Default for InsertType<'a> {
    fn default() -> Self {
        Self::WithValues(InsertWithValues::default())
    }
}

#[derive(Debug, Default, Clone)]
pub struct InsertQuery<'a> {
    pub table: Option<Cow<'a, str>>,
    pub ordered_columns: Option<Vec<&'a str>>,
    pub inner: InsertType<'a>,
}

impl<'a> InsertQuery<'a> {
    pub fn into_(&mut self, table: &'a str) -> &mut Self {
        self.table = Some(Cow::Borrowed(table));

        self
    }

    pub fn value<R: Row<'a>>(&mut self, row: R) -> &mut Self {
        match &mut self.inner {
            InsertType::FromSubQuery(_) => {
                panic!("cant")
            }
            InsertType::WithValues(insert) => {
                self.ordered_columns =
                    Some(R::columns().into_iter().map(|column| *column).collect());
                let mut builder = RowBuilder::default();

                row.into_row(&mut builder);

                insert.bindings.extend(builder.values);
            }
        };

        self
    }

    pub fn values<R: Row<'a>>(&mut self, rows: impl IntoIterator<Item = R>) -> &mut Self {
        match &mut self.inner {
            InsertType::FromSubQuery(_) => {
                panic!("cant")
            }
            InsertType::WithValues(insert) => {
                self.ordered_columns =
                    Some(R::columns().into_iter().map(|column| *column).collect());

                for row in rows.into_iter() {
                    let mut builder = RowBuilder::default();
                    row.into_row(&mut builder);

                    insert.bindings.extend(builder.values);
                }
            }
        };

        self
    }

    pub fn push_column(&mut self, column: &'a str) -> &mut Self {
        if let Some(columns) = &mut self.ordered_columns {
            columns.push(column);
        } else {
            self.ordered_columns = Some(vec![column]);
        }

        self
    }

    pub fn extend_columns(&mut self, new_columns: impl IntoIterator<Item = &'a &'a str>) -> &mut Self {
        if let Some(columns) = &mut self.ordered_columns {
            columns.extend(new_columns);
        } else {
            self.ordered_columns = Some(new_columns.into_iter().map(|column| *column).collect());
        }

        self
    }

    pub fn columns(&mut self, columns: impl IntoIterator<Item = &'a &'a str>) -> &mut Self {
        self.ordered_columns = Some(columns.into_iter().map(|column| *column).collect());

        self
    }

    pub fn from_sub_query(&mut self, s: impl Into<SubQuery<'a>>) -> &mut Self {
        self.inner = InsertType::FromSubQuery(InsertFromSubQuery { sub_query: s.into() });

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
