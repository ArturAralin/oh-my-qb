mod query_builder;
mod sql_dialect;

pub use query_builder::QueryBuilder;
use query_builder::Value;

#[cfg(test)]
mod tests {
    use crate::query_builder::*;
    use crate::sql_dialect::{self, *};

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

        let s = sql_dialect::postgres::PostgresSqlDialect::init().build_sql(&qb);

        // println!("{}", s.sql);
        // println!("{:?}", qb.bindings.as_ref().borrow());
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

        let s = sql_dialect::postgres::PostgresSqlDialect::init().build_sql(&qb);

        // println!("{}", s.sql);
        // println!("{:?}", qb.bindings.as_ref().borrow());
    }

    #[test]
    fn select() {
        let mut qb = QueryBuilder::new();

        qb.select(Some(&["column"])).from("my_tbl").or_where(
            "my_table.col1",
            "in",
            vec![false.value(), "array".value(), "value".value()],
        );

        let s = sql_dialect::postgres::PostgresSqlDialect::init().build_sql(&qb);

        // println!("{}", s.sql);
        // println!("{:?}", qb.bindings.as_ref().borrow());
    }

    #[test]
    fn select2() {
        let mut qb = QueryBuilder::new();

        qb.select(Some(&["column", "column2", "column3"]))
            .from("my_tbl")
            .and_where("ok", "ilike", "pattern*".value())
            .and_where_grouped(|and_where| {
                and_where.and_where("column", "=", "column2");
                and_where.and_where_grouped(|and_where| {
                    and_where.or_where("nested", "=", "column2");
                    and_where.or_where("nested", "=", "column2");
                });
            });

        let s = sql_dialect::postgres::PostgresSqlDialect::init().build_sql(&qb);

        // println!("{}", s.sql);
        // println!("{:?}", qb.bindings.as_ref().borrow());
    }

    #[test]
    fn select_and_where_null() {
        let mut qb = QueryBuilder::new();

        qb.select(None)
            .from("my_tbl")
            .and_where_null("my_column")
            .or_where_null("another_column");

        let s = sql_dialect::postgres::PostgresSqlDialect::init().build_sql(&qb);

        // println!("{}", s.sql);
        // println!("{:?}", qb.bindings.as_ref().borrow());
    }

    #[test]
    fn raw_arg() {
        let mut qb = QueryBuilder::new();

        qb.select(None).from("my_tbl").and_where(
            "column",
            "eq",
            "'{?, ?}'::int[]"
                .raw()
                .bindings(vec![10.value(), 20.value()]),
        );

        let s = sql_dialect::postgres::PostgresSqlDialect::init().build_sql(&qb);

        // println!("{}", s.sql);
        // println!("{:?}", qb.bindings.as_ref().borrow());
    }

    #[test]
    fn option_handling() {
        struct TestR {
            option_column: Option<String>,
        }

        impl<'a> Row<'a> for TestR {
            fn columns(&self) -> &'static [&'static str] {
                &["option_column"]
            }

            fn into_row(self, builder: &mut RowBuilder<'a>) {
                builder.append_binding(self.option_column.value());
            }
        }

        let mut qb = QueryBuilder::new();

        qb.insert().into("my_tbl").values(vec![
            TestR {
                option_column: Some("test_str".to_owned()),
            },
            TestR {
                option_column: None,
            },
        ]);

        let s = sql_dialect::postgres::PostgresSqlDialect::init().build_sql(&qb);

        // println!("{}", s.sql);
        // println!("{:?}", qb.bindings.as_ref().borrow());
    }

    #[test]
    fn sub_query() {
        struct TestR {
            option_column: Option<String>,
        }

        impl<'a> Row<'a> for TestR {
            fn columns(&self) -> &'static [&'static str] {
                &["option_column"]
            }

            fn into_row(self, builder: &mut RowBuilder<'a>) {
                builder.append_binding(self.option_column.value());
            }
        }

        let mut qb = QueryBuilder::new();
        let mut sub_qb = QueryBuilder::new();

        sub_qb
            .select(Some(&["id"]))
            .from("another_table")
            .and_where("left", "=", 25.value());

        qb.select(None)
            .from("table")
            .and_where("v", "=", 10.value())
            .and_where("left", "in", sub_qb.clone());

        qb.select(None)
            .from("table")
            .and_where("v", "=", 10.value())
            .and_where("left", "in", sub_qb);

        sql_dialect::postgres::PostgresSqlDialect::init().build_sql(&qb);

        // println!("{}", s.sql);
        // println!("{:?}", qb.bindings.as_ref().borrow());
    }

    #[test]
    fn sub_query_2() {
        struct TestR {
            option_column: Option<String>,
        }

        impl<'a> Row<'a> for TestR {
            fn columns(&self) -> &'static [&'static str] {
                &["option_column"]
            }

            fn into_row(self, builder: &mut RowBuilder<'a>) {
                builder.append_binding(self.option_column.value());
            }
        }

        let mut qb = QueryBuilder::new();
        let mut sub_qb = QueryBuilder::new();

        sub_qb
            .select(Some(&["id"]))
            .from("another_table")
            .and_where("left", "=", 25.value());

        qb.select(None)
            .from("table")
            .and_where("v", "=", 10.value())
            .and_where("left", "in", sub_qb.clone());

        qb.select(None)
            .from("table")
            .and_where("v", "=", 10.value())
            .and_where("left", "in", sub_qb);

        sql_dialect::postgres::PostgresSqlDialect::init().build_sql(&qb);

        // println!("{}", s.sql);
        // println!("{:?}", qb.bindings.as_ref().borrow());
    }

    #[test]
    fn builder() {
        struct TestR {
            option_column: Option<String>,
        }

        impl<'a> Row<'a> for TestR {
            fn columns(&self) -> &'static [&'static str] {
                &["option_column"]
            }

            fn into_row(self, builder: &mut RowBuilder<'a>) {
                builder.append_binding(self.option_column.value());
            }
        }

        let mut qb = QueryBuilder::new();
        let mut sub_qb = QueryBuilder::new();

        sub_qb
            .select(Some(&["id"]))
            .from("another_table")
            .and_where("left", "=", 25.value());

        let s = qb
            .select(None)
            .from("table")
            .and_where("v", "=", 10.value())
            .and_where("left", "in", sub_qb)
            .sql::<sql_dialect::postgres::PostgresSqlDialect>()
            .into_sqlx_qb()
            .into_sql();

        println!("{}", s);
    }
}

// pub trait Qb<'a> {
//     fn bindings(self, bindings: Vec<&'a Value<'a>>) -> Self;
// }

// impl<'a> Qb<'a> for sqlx::QueryBuilder<'a, sqlx::Postgres> {
//     fn bindings(mut self, bindings: Vec<&'a Value<'a>>) -> Self {
//         bindings.into_iter().for_each(|value| match value {
//             Value::Integer(v) => self.bind,
//             _ => unimplemented!("unimpl"),
//         });

//         self
//     }
// }
