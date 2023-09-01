mod postgres;
mod qb_arg;
mod query_builder;
mod row;
mod value;
mod where_clause;
use std::fmt::Debug;
use value::Value;

pub use query_builder::QueryBuilder;

#[derive(Debug)]
pub struct Sql<'a> {
    pub sql: String,
    pub binds: Vec<&'a Value<'a>>,
}

#[cfg(test)]
mod tests {
    use crate::row::RowBuilder;
    use crate::where_clause::Conditions;
    use crate::{postgres::BuildSql, query_builder::QueryBuilder, row::Row, value::ValueExt};

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
        println!("{:?}", qb.bindings.as_ref().borrow());
    }

    #[test]
    fn update() {
        let mut qb = QueryBuilder::new();

        let row = TestRow {
            abc: 1,
            my_string: "lalala".to_owned(),
        };

        qb.update(row)
            .table("my_tbl")
            .and_where("my_table.col1", "=", "my text".value())
            .or_where("my_table.col1", "=", "my 2".value());

        let s = postgres::Postgres::init().build_sql(&qb);

        println!("{}", s.sql);
        println!("{:?}", qb.bindings.as_ref().borrow());
    }

    #[test]
    fn select() {
        let mut qb = QueryBuilder::new();

        qb.select(Some(&["column"]))
            .from("my_tbl")
            .and_where("my_table.col1", "=", "my text".value())
            .or_where("my_table.col1", "=", "my 2".value());

        let s = postgres::Postgres::init().build_sql(&qb);

        println!("{}", s.sql);
        println!("{:?}", qb.bindings.as_ref().borrow());
    }
}
