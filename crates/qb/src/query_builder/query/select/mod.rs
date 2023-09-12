pub mod column;
pub mod join;
pub mod ordering;

use crate::query_builder::conditions;
use crate::{
    query_builder::{Arg, PushCondition, TryIntoArg},
    sql_dialect::{Sql, SqlDialect},
    Conditions,
};
use std::{borrow::Cow, rc::Rc};

#[derive(Debug, Default, Clone)]
pub struct SelectQuery<'a> {
    pub columns: Option<Vec<column::Column<'a>>>,
    pub table: Option<Rc<Arg<'a>>>,
    pub joins: Option<Vec<join::Join<'a>>>,
    pub where_: Vec<conditions::WhereCondition<'a>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub ordering: Option<Vec<ordering::Ordering<'a>>>,
    pub group_by: Option<Vec<Arg<'a>>>,
    pub alias: Option<Cow<'a, str>>,
}

impl<'a> SelectQuery<'a> {
    pub fn columns(
        &mut self,
        columns: impl IntoIterator<Item = impl column::TryIntoColumn<'a>>,
    ) -> &mut Self {
        self.columns = Some(
            columns
                .into_iter()
                .map(|a| a.try_into_column().unwrap())
                .collect(),
        );

        self
    }

    pub fn extend_columns(
        &mut self,
        new_columns: impl IntoIterator<Item = impl column::TryIntoColumn<'a>>,
    ) -> &mut Self {
        let columns_iter = new_columns
            .into_iter()
            .map(|column| column.try_into_column().unwrap());
        if let Some(columns) = &mut self.columns {
            columns.extend(columns_iter);
        } else {
            self.columns = Some(columns_iter.collect());
        }

        self
    }

    pub fn push_column(&mut self, column: impl column::TryIntoColumn<'a>) -> &mut Self {
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

    pub fn order_by(&mut self, ordering: impl ordering::TryIntoOrdering<'a>) -> &mut Self {
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
        let join = join::Join::Regular(join::RegularJoin {
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

    pub fn group_by(&mut self, group: impl TryIntoArg<'a>) -> &mut Self {
        let arg = group.try_into_arg().unwrap();

        if let Some(group_by) = &mut self.group_by {
            group_by.push(arg);
        } else {
            self.group_by = Some(vec![arg]);
        }

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
    fn push_cond(&mut self, cond: conditions::WhereCondition<'a>) {
        self.where_.push(cond);
    }
}

impl<'a> Conditions<'a> for SelectQuery<'a> {}

#[cfg(test)]
mod test {
    use super::SelectQuery;
    use crate::{query_builder::Arg, ColumnExt};

    fn compare_columns(qb: &SelectQuery, expected_columns: &[&str]) {
        let columns = qb
            .columns
            .as_ref()
            .map(|columns| {
                columns
                    .iter()
                    .map(|column| match &column.arg {
                        Arg::Relation(r) => &r.0,
                        _ => panic!("not relation"),
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap();

        assert_eq!(columns, expected_columns);
    }

    #[test]
    fn columns() {
        let mut qb = SelectQuery::default();
        qb.columns(["column0"]);

        compare_columns(&qb, &["column0"]);

        qb.columns(["column1"]);

        compare_columns(&qb, &["column1"]);

        qb.extend_columns(["column2", "column3"]);

        compare_columns(&qb, &["column1", "column2", "column3"]);

        qb.push_column("column4");

        compare_columns(&qb, &["column1", "column2", "column3", "column4"]);

        let dynamic_column = format!("column{}", 5);

        qb.push_column(&dynamic_column);

        compare_columns(
            &qb,
            &["column1", "column2", "column3", "column4", "column5"],
        );
    }

    #[test]
    fn alias() {
        let mut qb = SelectQuery::default();
        qb.push_column("column".alias("another_name"));

        let column = &qb.columns.as_ref().unwrap()[0];

        match &column.arg {
            Arg::Relation(r) => assert_eq!(r.0, "column"),
            _ => {
                assert!(false)
            }
        };

        assert_eq!(column.alias, Some(std::borrow::Cow::Borrowed("another_name")));
    }
}
