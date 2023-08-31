mod postgres;
mod qb_arg;
mod query_builder;
mod row;
mod value;
mod where_clause;
use std::{borrow::Cow, cell::RefCell, fmt::Debug, rc::Rc};

use query_builder::{InsertQuery, QueryType, SelectQuery};
use row::{Row, RowBuilder};
use value::Value;
use where_clause::{Condition, Conditions};

pub struct QueryBuilder<'a> {
    query_type: QueryType<'a>,
    table: Option<Cow<'a, str>>,
    where_conditions: Vec<Condition<'a>>,
    // bindings: Vec<Value<'a>>,
    bb: Rc<RefCell<Vec<Value<'a>>>>,
}

#[derive(Debug)]
pub struct Sql<'a> {
    pub sql: String,
    pub binds: Vec<&'a Value<'a>>,
}

impl<'a> QueryBuilder<'a> {
    pub fn new() -> Self {
        Self {
            query_type: QueryType::Select(Default::default()),
            table: None,
            where_conditions: Default::default(),
            // bindings: Default::default(),
            bb: Default::default(),
        }
    }

    pub fn with_table(table: &'a str) -> Self {
        let mut qb: Self = Self::new();

        qb.table(table);

        qb
    }

    pub fn table(&mut self, table: &'a str) -> &mut Self {
        self.table = Some(Cow::Borrowed(table));

        self
    }

    pub fn into(&mut self, table: &'a str) -> &mut Self {
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
        self.query_type = QueryType::Select(SelectQuery { columns });

        self
    }

    pub fn delete(&mut self) -> &mut Self {
        self.query_type = QueryType::Delete;

        self
    }

    pub fn insert(&mut self) -> &mut Self {
        self.query_type = QueryType::Insert(InsertQuery {
            rows: Default::default(),
            ordered_columns: Default::default(),
        });

        self
    }

    pub fn value<R: Row<'a>>(&mut self, row: R) -> &mut Self {
        let mut builder = RowBuilder::new(&self.bb);
        let columns = row.columns();

        row.into_row(&mut builder);

        if let QueryType::Insert(InsertQuery {
            rows,
            ordered_columns,
        }) = &mut self.query_type
        {
            *ordered_columns = Some(columns);
            rows.push(builder.into_slice());
        }

        self
    }
}

impl<'a> Conditions<'a> for QueryBuilder<'a> {
    fn push_cond(&mut self, cond: Condition<'a>) {
        self.where_conditions.push(cond);
    }

    fn push_bindings<I>(&mut self, values: I)
    where
        I: Iterator<Item = Value<'a>>,
    {
        self.bb.as_ref().borrow_mut().extend(values);
    }

    fn get_binding_idx(&self) -> usize {
        self.bb.as_ref().borrow().len() + 1
    }
}

#[cfg(test)]
mod tests {
    use crate::{postgres::BuildSql, row::Row, value::ValueExt};

    use super::*;

    struct TestRow {
        abc: i32,
        my_string: String,
    }

    impl<'a> Row<'a> for TestRow {
        fn columns(&self) -> &'static [&'static str] {
            &["abc", "my_string"]
        }

        fn into_row(self, builder: &mut RowBuilder<'a>) {
            builder.append_binding(self.abc.value());
            builder.append_binding(self.my_string.value());
        }
    }

    #[test]
    fn qb() {
        let mut qb = QueryBuilder::new();

        qb.insert()
            .into("ok")
            .value(TestRow {
                abc: 1,
                my_string: "lalala".to_owned(),
            })
            .value(TestRow {
                abc: 2,
                my_string: "lololo".to_owned(),
            })
            .and_where("my_table.col1", "=", "my text".value())
            .or_where("my_table.col1", "=", "my 2".value());

        let s = postgres::Postgres::init().build_sql(&qb);

        println!("{}", s.sql);
        println!("{:?}", qb.bb.as_ref().borrow());
    }
}
