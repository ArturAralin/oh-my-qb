mod dialect;
mod query_builder;

pub use query_builder::QueryBuilder;

#[cfg(test)]
mod tests {
    use crate::dialect::{self, *};
    use crate::query_builder::*;

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

        let s = dialect::postgres::Postgres::init().build_sql(&qb);

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

        let s = dialect::postgres::Postgres::init().build_sql(&qb);

        println!("{}", s.sql);
        println!("{:?}", qb.bindings.as_ref().borrow());
    }

    #[test]
    fn select() {
        let mut qb = QueryBuilder::new();

        qb.select(Some(&["column"])).from("my_tbl").or_where(
            "my_table.col1",
            "in",
            vec![false.value(), "array".value(), "value".value()],
        );

        let s = dialect::postgres::Postgres::init().build_sql(&qb);

        println!("{}", s.sql);
        println!("{:?}", qb.bindings.as_ref().borrow());
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

        let s = dialect::postgres::Postgres::init().build_sql(&qb);

        println!("{}", s.sql);
        println!("{:?}", qb.bindings.as_ref().borrow());
    }
}
