mod conditions;
mod qb_arg;
mod query;
mod row;
mod value;
mod where_clause;

pub use self::query::delete::*;
pub use self::query::insert::*;
pub use self::query::join::*;
pub use self::query::select::*;
use crate::sql_dialect::{Sql, SqlDialect};
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
            Query::Delete(_) => {
                // delete.table = Some(Rc::new(<T as TryIntoArg>::try_into_arg(table).unwrap()));
            }
            Query::Insert(_) => {
                // insert.table = Some(Rc::new(<T as TryIntoArg>::try_into_arg(table).unwrap()));
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

    pub fn from<T: TryIntoArg<'a>>(&mut self, table: T) -> &mut Self {
        self.table_internal(table);

        self
    }

    pub fn select<'b>() -> SelectQuery<'b> {
        SelectQuery::default()
    }

    pub fn delete<'b>() -> DeleteQuery<'b> {
        DeleteQuery::default()
    }

    pub fn insert<'b>() -> InsertQuery<'b> {
        InsertQuery::default()
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

    pub fn sql<D>(&'a self) -> Sql<'a>
    where
        D: SqlDialect<'a>,
    {
        let mut builder = D::init();

        builder.build_sql(self);

        builder.sql()
    }

    pub fn sqlx_qb<D: SqlDialect<'a>>(&'a self) -> D::SqlxQb {
        let mut builder = D::init();

        builder.build_sql(self);

        builder.into_sqlx_qb()
    }
}

impl<'a> PushCondition<'a> for QueryBuilder<'a> {
    fn push_cond(&mut self, cond: WhereCondition<'a>) {
        match &mut self.query {
            Query::Select(query) => query.where_.push(cond),
            Query::Update(query) => query.where_clause.push(cond),
            Query::Delete(query) => query.where_clause.push(cond),
            _ => {
                unimplemented!("where unsupported yet");
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct UpdateQuery<'a> {
    pub table: Option<Rc<Arg<'a>>>,
    pub columns: Vec<Cow<'a, str>>,
    pub values: Vec<Value<'a>>,
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
