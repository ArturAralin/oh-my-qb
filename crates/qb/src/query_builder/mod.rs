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
pub use self::query::update::*;
pub use conditions::*;
pub use qb_arg::*;
pub use row::*;
use std::borrow::Cow;
pub use value::*;
pub use where_clause::*;

pub struct QueryBuilder;

impl QueryBuilder {
    pub fn select<'b>() -> SelectQuery<'b> {
        SelectQuery::default()
    }

    pub fn delete<'b>() -> DeleteQuery<'b> {
        DeleteQuery::default()
    }

    pub fn insert<'b>() -> InsertQuery<'b> {
        InsertQuery::default()
    }

    pub fn update<'b, R: Row<'b>>(row: R) -> UpdateQuery<'b> {
        let mut builder = RowBuilder::default();

        let columns = R::columns()
            .iter()
            .map(|column| Cow::Borrowed(*column))
            .collect();

        row.into_row(&mut builder);

        UpdateQuery {
            columns,
            values: builder.values,
            ..Default::default()
        }
    }
}
