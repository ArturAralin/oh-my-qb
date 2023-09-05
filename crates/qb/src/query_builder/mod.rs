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

    fn table_internal<T: TryIntoArg<'a>>(&mut self, table: T) {
        match &mut self.query {
            Query::Delete(delete) => {
                delete.table = Some(Rc::new(<T as TryIntoArg>::try_into_arg(table).unwrap()));
            }
            Query::Insert(insert) => {
                insert.table = Some(Rc::new(<T as TryIntoArg>::try_into_arg(table).unwrap()));
            }
            Query::Select(select) => {
                select.table = Some(Rc::new(<T as TryIntoArg>::try_into_arg(table).unwrap()));
            }
            Query::Update(update) => {
                update.table = Some(Rc::new(<T as TryIntoArg>::try_into_arg(table).unwrap()));
            }
        };
    }

    pub fn table(&mut self, table: &'a str) -> &mut Self {
        self.table_internal(table);

        self
    }

    pub fn into(&mut self, table: &'a str) -> &mut Self {
        self.table_internal(table);

        self
    }

    pub fn from<T: TryIntoArg<'a>>(&mut self, table: T) -> &mut Self {
        self.table_internal(table);

        self
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
            ..Default::default()
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
            // rows: Default::default(),
            values: Default::default(),
            ordered_columns: Default::default(),
        });

        self
    }

    pub fn update<R: Row<'a>>(&mut self, row: R) -> &mut Self {
        let mut builder = RowBuilder::default();

        let columns = R::columns()
            .iter()
            .map(|column| Cow::Borrowed(*column))
            .collect();

        row.into_row(&mut builder);

        self.query = Query::Update(UpdateQuery {
            columns,
            values: builder.values,
            table: None,
            where_clause: Default::default(),
        });

        self
    }

    pub fn value<R: Row<'a>>(&mut self, row: R) -> &mut Self {
        let mut builder = RowBuilder::default();

        row.into_row(&mut builder);

        if let Query::Insert(InsertQuery {
            values,
            ordered_columns,
            ..
        }) = &mut self.query
        {
            *ordered_columns = Some(R::columns());
            values.extend(builder.values);
        }

        self
    }

    pub fn values<R: Row<'a>>(&mut self, rows: impl IntoIterator<Item = R>) -> &mut Self {
        if let Query::Insert(InsertQuery {
            values,
            ordered_columns,
            ..
        }) = &mut self.query
        {
            *ordered_columns = Some(R::columns());

            for row in rows.into_iter() {
                let mut builder = RowBuilder::default();
                row.into_row(&mut builder);

                values.extend(builder.values);
            }
        } else {
            // return error
        }

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

    pub fn alias(&mut self, alias: &'a str) -> &mut Self {
        if let Query::Select(select) = &mut self.query {
            select.alias = Some(Cow::Borrowed(alias));
        } else {
            // todo: error here
        }

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
    pub table: Option<Rc<Arg<'a>>>,
    pub ordered_columns: Option<&'static [&'static str]>,
    pub values: Vec<Value<'a>>,
}

#[derive(Debug, Clone)]
pub struct UpdateQuery<'a> {
    pub table: Option<Rc<Arg<'a>>>,
    pub columns: Vec<Cow<'a, str>>,
    pub values: Vec<Value<'a>>,
    pub where_clause: Vec<WhereCondition<'a>>,
}

#[derive(Debug, Clone)]
pub struct DeleteQuery<'a> {
    pub table: Option<Rc<Arg<'a>>>,
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
