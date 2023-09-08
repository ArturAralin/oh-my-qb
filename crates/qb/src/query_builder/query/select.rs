use crate::{
    query_builder::{Arg, PushCondition, Relation, SqlKeyword, TryIntoArg, WhereCondition},
    sql_dialect::{Sql, SqlDialect},
    Conditions,
};
use std::{borrow::Cow, rc::Rc};

use super::join::{Join, RegularJoin};

#[derive(Debug, Default, Clone)]
pub struct SelectQuery<'a> {
    pub columns: Option<Vec<Column<'a>>>,
    pub table: Option<Rc<Arg<'a>>>,
    pub joins: Option<Vec<Join<'a>>>,
    pub where_: Vec<WhereCondition<'a>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub ordering: Option<Vec<Ordering<'a>>>,
    pub alias: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone)]
pub struct Ordering<'a> {
    pub left: Arg<'a>,
    pub right: Arg<'a>,
    // only for PG
    pub null_first: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct Column<'a> {
    pub arg: Arg<'a>,
    pub alias: Option<Cow<'a, str>>,
}

pub trait TryIntoOrdering<'a> {
    fn try_into_ordering(self) -> Result<Ordering<'a>, ()>;
}

impl<'a, T1: TryIntoArg<'a>, T2: TryIntoArg<'a>> TryIntoOrdering<'a> for (T1, T2) {
    fn try_into_ordering(self) -> Result<Ordering<'a>, ()> {
        Ok(Ordering {
            left: self.0.try_into_arg().unwrap(),
            right: self.1.try_into_arg().unwrap(),
            null_first: None,
        })
    }
}

impl<'a, T1: TryIntoArg<'a>, T2: TryIntoArg<'a>> TryIntoOrdering<'a> for (T1, T2, SqlKeyword) {
    fn try_into_ordering(self) -> Result<Ordering<'a>, ()> {
        Ok(Ordering {
            left: self.0.try_into_arg().unwrap(),
            right: self.1.try_into_arg().unwrap(),
            null_first: Some(matches!(self.2, SqlKeyword::NullsFirst)),
        })
    }
}

pub trait ColumnExt<'a> {
    fn alias(self, alias: &'a str) -> Column<'a>;
}

impl<'a> ColumnExt<'a> for &'a str {
    fn alias(self, alias: &'a str) -> Column<'a> {
        Column {
            arg: Arg::Relation(Relation(Cow::Borrowed(self))),
            alias: Some(Cow::Borrowed(alias)),
        }
    }
}

pub trait TryIntoColumn<'a> {
    fn try_into_column(self) -> Result<Column<'a>, ()>;
}

impl<'a> TryIntoColumn<'a> for &'a str {
    fn try_into_column(self) -> Result<Column<'a>, ()> {
        Ok(Column {
            arg: self.try_into_arg().unwrap(),
            alias: None,
        })
    }
}

impl<'a> TryIntoColumn<'a> for SelectQuery<'a> {
    fn try_into_column(self) -> Result<Column<'a>, ()> {
        Ok(Column {
            arg: self.try_into_arg().unwrap(),
            alias: None,
        })
    }
}

impl<'a> TryIntoColumn<'a> for Column<'a> {
    fn try_into_column(self) -> Result<Column<'a>, ()> {
        Ok(self)
    }
}

impl<'a> SelectQuery<'a> {
    pub fn columns(&mut self, columns: Vec<impl TryIntoColumn<'a>>) -> &mut Self {
        // todo: rework to extend?
        self.columns = Some(
            columns
                .into_iter()
                .map(|a| a.try_into_column().unwrap())
                .collect(),
        );

        self
    }

    pub fn push_column(&mut self, column: impl TryIntoColumn<'a>) -> &mut Self {
        if let Some(columns) = &mut self.columns {
            columns.push(column.try_into_column().unwrap());
        } else {
            self.columns = Some(vec![column.try_into_column().unwrap()]);
        }

        self
    }

    pub fn from<T: TryIntoArg<'a>>(&mut self, table: T) -> &mut Self {
        self.table = Some(Rc::new(<T as TryIntoArg>::try_into_arg(table).unwrap()));

        self
    }

    pub fn limit(&mut self, limit: usize) -> &mut Self {
        self.limit = Some(limit);

        self
    }

    pub fn offset(&mut self, offset: usize) -> &mut Self {
        self.offset = Some(offset);

        self
    }

    pub fn order_by(&mut self, ordering: impl TryIntoOrdering<'a>) -> &mut Self {
        let order = ordering.try_into_ordering().unwrap();

        if let Some(ordering) = &mut self.ordering {
            ordering.push(order);
        } else {
            self.ordering = Some(vec![order])
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
    ) {
        let join = Join::Regular(RegularJoin {
            join_type,
            table: Cow::Borrowed(table),
            left: left.try_into_arg().unwrap(),
            op: Cow::Borrowed(op),
            right: right.try_into_arg().unwrap(),
        });

        if let Some(joins) = &mut self.joins {
            joins.push(join)
        } else {
            self.joins = Some(vec![join]);
        }
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
        self.alias = Some(Cow::Borrowed(alias));

        self
    }

    // todo: pub fn column(&mut self, column: &str)

    pub fn sql<D>(&'a self) -> Sql<'a>
    where
        D: SqlDialect<'a>,
    {
        let mut builder = D::init();

        builder.build_select(self);

        builder.sql()
    }

    pub fn sqlx_qb<D: SqlDialect<'a>>(&'a self) -> D::SqlxQb {
        let mut builder = D::init();

        builder.build_select(self);

        builder.into_sqlx_qb()
    }
}

impl<'a> PushCondition<'a> for SelectQuery<'a> {
    fn push_cond(&mut self, cond: WhereCondition<'a>) {
        self.where_.push(cond);
    }
}

impl<'a> Conditions<'a> for SelectQuery<'a> {}
