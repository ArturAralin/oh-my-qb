use crate::{
    query_builder::{Arg, PushCondition, TryIntoArg, WhereCondition},
    sql_dialect::{Sql, SqlDialect},
    Conditions,
};
use std::{borrow::Cow, rc::Rc};

use super::join::{Join, RegularJoin};

#[derive(Debug, Default, Clone)]
pub struct SelectQuery<'a> {
    pub columns: Option<Vec<Cow<'a, str>>>,
    pub table: Option<Rc<Arg<'a>>>,
    pub joins: Option<Vec<Join<'a>>>,
    pub where_: Vec<WhereCondition<'a>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub alias: Option<Cow<'a, str>>,
}

impl<'a> SelectQuery<'a> {
    pub fn columns(&mut self, columns: Option<&'a [&'a str]>) -> &mut Self {
        self.columns = columns.map(|columns| {
            columns
                .as_ref()
                .iter()
                .map(|column| Cow::Borrowed(*column))
                .collect::<Vec<_>>()
        });

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
            left: <L as TryIntoArg>::try_into_arg(left).unwrap(),
            op: Cow::Borrowed(op),
            right: <R as TryIntoArg>::try_into_arg(right).unwrap(),
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
