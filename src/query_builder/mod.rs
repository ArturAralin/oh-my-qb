mod conditions;
mod qb_arg;
mod query;
mod row;
mod value;
mod where_clause;

use self::query::join::RegularJoin;
pub use self::query::join::*;
pub use self::query::select::*;
use crate::sql_dialect::{BuildSql, Sql};
pub use conditions::*;
pub use qb_arg::*;
pub use row::*;
use std::{borrow::Cow, cell::RefCell, rc::Rc};
pub use value::*;
pub use where_clause::*;

#[derive(Default, Debug, Clone)]
pub struct QueryBuilder<'a> {
    pub query: Query<'a>,
    pub bindings: Rc<RefCell<Vec<Value<'a>>>>,
}

impl<'a> QueryBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn table(&mut self, table: &'a str) -> &mut Self {
        match &mut self.query {
            Query::Delete(delete) => {
                delete.table = Some(Cow::Borrowed(table));
            }
            Query::Insert(insert) => {
                insert.table = Some(Cow::Borrowed(table));
            }
            Query::Select(select) => {
                select.table = Some(Cow::Borrowed(table));
            }
            Query::Update(update) => {
                update.table = Some(Cow::Borrowed(table));
            }
        };

        self
    }

    pub fn into(&mut self, table: &'a str) -> &mut Self {
        self.table(table)
    }

    pub fn from(&mut self, table: &'a str) -> &mut Self {
        self.table(table)
    }

    pub fn select(&mut self, columns: Option<&'a [&'a str]>) -> &mut Self {
        let columns = columns.map(|columns| {
            columns
                .as_ref()
                .iter()
                .map(|column| Cow::Borrowed(*column))
                .collect::<Vec<_>>()
        });
        self.query = Query::Select(SelectQuery {
            columns,
            table: None,
            joins: None,
            where_clause: Default::default(),
            limit: None,
            offset: None,
        });

        self
    }

    pub fn delete(&mut self) -> &mut Self {
        self.query = Query::Delete(DeleteQuery {
            table: None,
            where_clause: Default::default(),
        });

        self
    }

    pub fn insert(&mut self) -> &mut Self {
        self.query = Query::Insert(InsertQuery {
            table: None,
            rows: Default::default(),
            ordered_columns: Default::default(),
        });

        self
    }

    pub fn update<R: Row<'a>>(&mut self, row: R) -> &mut Self {
        let columns = row.columns();
        let mut builder = RowBuilder::new(&self.bindings);
        row.into_row(&mut builder);

        let (mut start, _) = builder.into_slice();

        let columns = columns
            .iter()
            .map(|col| {
                start += 1;
                (Cow::Borrowed(*col), start)
            })
            .collect::<Vec<_>>();

        self.query = Query::Update(UpdateQuery {
            columns,
            table: None,
            where_clause: Default::default(),
        });

        self
    }

    pub fn value<R: Row<'a>>(&mut self, row: R) -> &mut Self {
        let mut builder = RowBuilder::new(&self.bindings);
        let columns = row.columns();

        row.into_row(&mut builder);

        if let Query::Insert(InsertQuery {
            rows,
            ordered_columns,
            ..
        }) = &mut self.query
        {
            *ordered_columns = Some(columns);
            rows.push(builder.into_slice());
        }

        self
    }

    pub fn values<R: Row<'a>>(&mut self, rows: Vec<R>) -> &mut Self {
        let columns = rows[0].columns();

        rows.into_iter().for_each(|row| {
            let mut builder = RowBuilder::new(&self.bindings);
            row.into_row(&mut builder);

            if let Query::Insert(InsertQuery {
                rows,
                ordered_columns,
                ..
            }) = &mut self.query
            {
                *ordered_columns = Some(columns);
                rows.push(builder.into_slice());
            }
        });

        self
    }

    pub fn limit(&mut self, limit: usize) -> &mut Self {
        if let Query::Select(select) = &mut self.query {
            select.limit = Some(limit);
        } else {
            // todo: error here
        }

        self
    }

    pub fn offset(&mut self, offset: usize) -> &mut Self {
        if let Query::Select(select) = &mut self.query {
            select.offset = Some(offset);
        } else {
            // todo: error here
        }

        self
    }

    fn join_internal<L: TryIntoArg<'a>, R: TryIntoArg<'a>>(
        &mut self,
        join_type: Option<&'static str>,
        table: &'a str,
        left: L,
        op: &'a str,
        right: R,
    ) -> &mut Self {
        if let Query::Select(select) = &mut self.query {
            let join = query::join::Join::Regular(RegularJoin {
                join_type,
                table: Cow::Borrowed(table),
                left: <L as TryIntoArg>::try_into_arg(left).unwrap(),
                op: Cow::Borrowed(op),
                right: <R as TryIntoArg>::try_into_arg(right).unwrap(),
            });
            if let Some(joins) = &mut select.joins {
                joins.push(join)
            } else {
                select.joins = Some(vec![join]);
            }
        } else {
            // todo: error here
        }

        self
    }

    pub fn join<L: TryIntoArg<'a>, R: TryIntoArg<'a>>(
        &mut self,
        table: &'a str,
        left: L,
        op: &'a str,
        right: R,
    ) -> &mut Self {
        self.join_internal(None, table, left, op, right);

        self
    }

    pub fn left_join<L: TryIntoArg<'a>, R: TryIntoArg<'a>>(
        &mut self,
        table: &'a str,
        left: L,
        op: &'a str,
        right: R,
    ) -> &mut Self {
        self.join_internal(Some("left"), table, left, op, right);

        self
    }

    pub fn right_join<L: TryIntoArg<'a>, R: TryIntoArg<'a>>(
        &mut self,
        table: &'a str,
        left: L,
        op: &'a str,
        right: R,
    ) -> &mut Self {
        self.join_internal(Some("right"), table, left, op, right);

        self
    }

    pub fn inner_join<L: TryIntoArg<'a>, R: TryIntoArg<'a>>(
        &mut self,
        table: &'a str,
        left: L,
        op: &'a str,
        right: R,
    ) -> &mut Self {
        self.join_internal(Some("inner"), table, left, op, right);

        self
    }

    pub fn sql<D>(&'a self) -> Sql<'a>
    where
        D: BuildSql<'a>,
    {
        let mut builder = D::init();

        builder.build_sql(self);

        builder.sql()
    }

    pub fn sqlx_qb<D: BuildSql<'a>>(&'a self) -> D::SqlxQb {
        let mut builder = D::init();

        builder.build_sql(self);

        builder.into_sqlx_qb()
    }
}

impl<'a> Conditions<'a> for QueryBuilder<'a> {
    fn push_cond(&mut self, cond: WhereCondition<'a>) {
        match &mut self.query {
            Query::Select(query) => query.where_clause.push(cond),
            Query::Update(query) => query.where_clause.push(cond),
            Query::Delete(query) => query.where_clause.push(cond),
            _ => {
                unimplemented!("where unsupported yet");
            }
        }
    }

    fn get_bindings(&self) -> Rc<RefCell<Vec<Value<'a>>>> {
        Rc::clone(&self.bindings)
    }
}

#[derive(Debug, Clone)]
pub struct InsertQuery<'a> {
    pub table: Option<Cow<'a, str>>,
    pub rows: Vec<(usize, usize)>,
    pub ordered_columns: Option<&'static [&'static str]>,
}

#[derive(Debug, Clone)]
pub struct UpdateQuery<'a> {
    pub table: Option<Cow<'a, str>>,
    pub columns: Vec<(Cow<'a, str>, usize)>,
    pub where_clause: Vec<WhereCondition<'a>>,
}

#[derive(Debug, Clone)]
pub struct DeleteQuery<'a> {
    pub table: Option<Cow<'a, str>>,
    pub where_clause: Vec<WhereCondition<'a>>,
}

#[derive(Debug, Clone)]
pub enum Query<'a> {
    Select(SelectQuery<'a>),
    Update(UpdateQuery<'a>),
    Delete(DeleteQuery<'a>),
    Insert(InsertQuery<'a>),
}

impl<'a> Default for Query<'a> {
    fn default() -> Self {
        Self::Select(SelectQuery::default())
    }
}
