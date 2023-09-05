pub mod postgres;
use crate::query_builder::{
    Arg, ArgValue, ConditionOp, DeleteQuery, GroupedWhereCondition, InsertQuery, Join, Query,
    QueryBuilder, Raw, SelectQuery, SingleWhereCondition, UpdateQuery, Value, WhereCondition,
};

#[derive(Debug)]
pub enum Dialect {
    Postgres,
}

pub struct Sql<'a> {
    pub sql: String,
    pub bindings: Vec<&'a Value<'a>>,

    // todo: remove it?
    pub dialect: Dialect,
}

pub trait BuildSql<'a> {
    const RELATION_QUOTE: char;
    type SqlxQb;

    fn init() -> Self;
    fn dialect() -> Dialect;
    fn sql(self) -> Sql<'a>;

    fn write_str<S: AsRef<str>>(&mut self, sql: S);
    fn write_char(&mut self, ch: char);
    fn push_binding(&mut self, binding: &'a Value<'a>) -> usize;
    fn extend_bindings(&mut self, bindings: impl IntoIterator<Item = &'a Value<'a>>);

    fn into_sqlx_qb(self) -> Self::SqlxQb;

    fn build_sql(&mut self, qb: &'a QueryBuilder<'a>) {
        match &qb.query {
            Query::Select(select) => {
                self.build_select(select);
            }
            Query::Delete(_) => {
                self.build_delete(qb);
            }
            Query::Insert(_) => {
                self.build_insert(qb);
            }
            Query::Update(_) => {
                self.build_update(qb);
            }
        }
    }

    fn write_relation(&mut self, relation: &str) {
        for (idx, relation_part) in relation.split('.').enumerate() {
            if idx > 0 {
                self.write_char('.');
            }

            if relation_part == "*" {
                self.write_char('*');
            } else {
                self.write_char(Self::RELATION_QUOTE);
                self.write_str(relation_part);
                self.write_char(Self::RELATION_QUOTE);
            }
        }
    }

    fn build_select(&mut self, select: &'a SelectQuery<'a>) {
        let SelectQuery {
            columns,
            table,
            joins,
            where_clause,
            limit,
            offset,
            ..
        } = select;

        self.write_str("select");

        match columns {
            Some(columns) => {
                self.write_char(' ');

                if columns.is_empty() {
                    self.write_char('*');
                } else {
                    columns.iter().enumerate().for_each(|(idx, column)| {
                        if idx > 0 {
                            self.write_char(',');
                            self.write_char(' ');
                        }

                        self.write_relation(column);
                    });
                }
            }
            None => {
                self.write_char(' ');
                self.write_char('*');
            }
        }

        if let Some(table) = table {
            self.write_str(" from ");
            self.write_arg(table);
        }

        if let Some(joins) = joins {
            joins.iter().for_each(|join| {
                let Join::Regular(join) = join;

                if let Some(join_type) = join.join_type {
                    self.write_char(' ');
                    self.write_str(join_type);
                }

                self.write_str(" join ");
                self.write_relation(&join.table);
                self.write_str(" on ");
                self.write_arg(&join.left);
                self.write_char(' ');
                self.write_str(&join.op);
                self.write_char(' ');
                self.write_arg(&join.right);
            });
        }

        self.build_where(where_clause, 0);

        if let Some(limit) = limit {
            self.write_str(" limit ");
            self.write_str(limit.to_string().as_str());
        }

        if let Some(offset) = offset {
            self.write_str(" offset ");
            self.write_str(offset.to_string().as_str());
        }
    }

    fn build_delete(&mut self, qb: &'a QueryBuilder<'a>) {
        if let Query::Delete(DeleteQuery {
            table,
            where_clause,
        }) = &qb.query
        {
            self.write_str("delete");

            if let Some(table) = table {
                self.write_str(" from ");
                self.write_arg(table);
            }

            self.build_where(where_clause, 0);
        }
    }

    fn build_update(&mut self, qb: &'a QueryBuilder<'a>) {
        if let Query::Update(UpdateQuery {
            columns,
            values,
            table,
            where_clause,
        }) = &qb.query
        {
            values.iter().for_each(|value| {
                self.push_binding(value);
            });

            self.write_str("update");

            if let Some(table) = table {
                self.write_char(' ');
                self.write_arg(table);
            }

            self.write_str(" set");

            columns.iter().enumerate().for_each(|(idx, column)| {
                if idx > 0 {
                    self.write_char(',');
                }

                self.write_char(' ');
                self.write_relation(column);
                self.write_str(" = $");
                self.write_str((idx + 1).to_string().as_str());
            });

            self.build_where(where_clause, 0);
        }
    }

    fn build_insert(&mut self, qb: &'a QueryBuilder<'a>) {
        if let Query::Insert(InsertQuery {
            ordered_columns,
            values,
            table,
        }) = &qb.query
        {
            self.write_str("insert");

            if let Some(table) = table {
                self.write_str(" into ");
                self.write_arg(table.as_ref());
            }

            if let Some(ordered_columns) = ordered_columns {
                self.write_char(' ');
                self.write_char('(');

                ordered_columns
                    .iter()
                    .enumerate()
                    .for_each(|(idx, column)| {
                        if idx > 0 {
                            self.write_char(',');
                            self.write_char(' ');
                        }

                        self.write_str(format!(r#""{}""#, column).as_str());
                    });

                self.write_char(')');
            }

            if !values.is_empty() {
                self.write_str(" values ");

                let mut binding_idx: usize = 1;

                for (tuple_idx, values) in values
                    .chunks(ordered_columns.map(|columns| columns.len()).unwrap_or(0))
                    .enumerate()
                {
                    if tuple_idx > 0 {
                        self.write_char(',');
                        self.write_char(' ');
                    }

                    self.write_char('(');

                    for (idx, _) in values.iter().enumerate() {
                        if idx > 0 {
                            self.write_char(',');
                            self.write_char(' ');
                        }

                        self.write_char('$');
                        self.write_str(binding_idx.to_string());
                        binding_idx += 1;
                    }

                    self.write_char(')');
                }
            }

            self.extend_bindings(values);
        }
    }

    fn build_where(&mut self, where_conditions: &'a [WhereCondition<'a>], depth: usize) {
        if !where_conditions.is_empty() && depth == 0 {
            self.write_str(" where");
        }

        where_conditions
            .iter()
            .enumerate()
            .for_each(|(idx, condition)| match condition {
                WhereCondition::Group(GroupedWhereCondition { op, conditions }) => {
                    if idx > 0 {
                        match op {
                            ConditionOp::And => {
                                self.write_str(" and");
                            }
                            ConditionOp::Or => {
                                self.write_str(" or");
                            }
                        };
                    }

                    self.write_char(' ');
                    self.write_char('(');
                    self.build_where(conditions, depth + 1);
                    self.write_char(')');
                }
                WhereCondition::Single(SingleWhereCondition {
                    op,
                    right,
                    left,
                    middle,
                    ..
                }) => {
                    if idx > 0 {
                        match op {
                            ConditionOp::And => {
                                self.write_str(" and");
                            }
                            ConditionOp::Or => {
                                self.write_str(" or");
                            }
                        };
                    }

                    self.write_char(' ');
                    self.write_arg(left);
                    self.write_char(' ');
                    self.write_str(middle);
                    self.write_char(' ');
                    self.write_arg(right);
                }
            });
    }

    fn write_arg(&mut self, arg: &'a Arg<'a>) {
        match arg {
            Arg::Relation(rel) => self.write_relation(&rel.0),
            Arg::Value(ArgValue::Value(Value::Null)) => {
                self.write_str("null");
            }
            Arg::Value(ArgValue::Value(value)) => {
                let idx = self.push_binding(value);
                self.write_char('$');
                self.write_str(idx.to_string().as_str());
            }
            Arg::Value(ArgValue::Values(v)) => {
                self.write_char('(');
                v.iter().for_each(|value| {
                    let idx = self.push_binding(value);
                    self.write_char('$');
                    self.write_str(idx.to_string().as_str());
                });
                self.write_char(')');
            }
            Arg::Raw(Raw {
                sql,
                bindings_slice,
                ..
            }) => {
                let mut idx = bindings_slice.map(|(start, _)| start).unwrap_or(0);

                for ch in sql.chars() {
                    if ch == '?' {
                        idx += 1;
                        self.write_str(format!("${}", idx).as_str());
                    } else {
                        self.write_char(ch);
                    }
                }
            }
            Arg::SubQuery(sub_query) => {
                self.write_char('(');
                self.build_sql(&sub_query.0);
                self.write_char(')');

                if let Query::Select(SelectQuery {
                    alias: Some(alias), ..
                }) = &sub_query.0.query
                {
                    self.write_str(" as ");
                    self.write_relation(alias);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::BuildSql;
    use crate::{prelude::*, query_builder::Value};
    #[derive(Debug, Default)]
    pub struct TestDialect<'a> {
        pub sql: String,
        pub bindings: Vec<&'a Value<'a>>,
    }

    impl<'a> BuildSql<'a> for TestDialect<'a> {
        const RELATION_QUOTE: char = '"';
        type SqlxQb = ();

        fn init() -> Self {
            Self::default()
        }

        fn dialect() -> super::Dialect {
            super::Dialect::Postgres
        }

        fn sql(self) -> super::Sql<'a> {
            super::Sql {
                sql: self.sql,
                bindings: self.bindings,
                dialect: Self::dialect(),
            }
        }

        fn into_sqlx_qb(self) -> Self::SqlxQb {}

        fn write_char(&mut self, ch: char) {
            self.sql.push(ch);
        }

        fn write_str<S: AsRef<str>>(&mut self, sql: S) {
            self.sql.push_str(sql.as_ref());
        }

        fn push_binding(&mut self, value: &'a Value<'a>) -> usize {
            self.bindings.push(value);
            self.bindings.len()
        }

        fn extend_bindings(&mut self, bindings: impl IntoIterator<Item = &'a Value<'a>>) {
            self.bindings.extend(bindings);
        }
    }

    #[test]
    fn select_all() {
        let mut qb = QueryBuilder::new();
        let sql = qb.select(None).from("table").sql::<TestDialect>();

        assert_eq!(sql.sql, r#"select * from "table""#);
        assert!(sql.bindings.is_empty());
    }

    #[test]
    fn select_columns() {
        let mut qb = QueryBuilder::new();
        let sql = qb
            .select(Some(&["column1", "column2", "namespace.column"]))
            .from("table")
            .sql::<TestDialect>();

        assert_eq!(
            sql.sql,
            r#"select "column1", "column2", "namespace"."column" from "table""#
        );
        assert!(sql.bindings.is_empty());
    }

    #[test]
    fn regular_join() {
        let mut qb = QueryBuilder::new();
        let sql = qb
            .select(None)
            .from("table")
            .left_join("another_table", "table.id", "=", "another_table.t_id")
            .sql::<TestDialect>();

        assert_eq!(
            sql.sql,
            r#"select * from "table" left join "another_table" on "table"."id" = "another_table"."t_id""#
        );
        assert!(sql.bindings.is_empty());
    }

    #[test]
    fn sub_query_alias() {
        let mut sub_qb = QueryBuilder::new();
        sub_qb.select(None).from("super_table").alias("my_alias");

        let mut qb = QueryBuilder::new();
        let sql = qb.select(None).from(sub_qb).sql::<TestDialect>();

        assert_eq!(
            sql.sql,
            r#"select * from (select * from "super_table") as "my_alias""#
        );
        assert!(sql.bindings.is_empty());
    }

    #[test]
    fn select_column_asterisk() {
        let mut qb = QueryBuilder::new();
        let sql = qb
            .select(Some(&["my_tbl.*"]))
            .from("my_tbl")
            .sql::<TestDialect>();

        assert_eq!(sql.sql, r#"select "my_tbl".* from "my_tbl""#);
        assert!(sql.bindings.is_empty());
    }

    #[test]
    fn update() {
        #[derive(unnamed_qb_macro::Row)]
        struct TestRow {
            a: String,
            b: i32,
            c: i64,
            f: bool,
            d: Option<i32>,
        }

        let r = TestRow {
            a: "a_val".to_owned(),
            b: 10,
            c: 20,
            f: false,
            d: None,
        };

        let mut qb = QueryBuilder::new();
        let sql = qb.update(r).table("my_tbl").sql::<TestDialect>();

        assert_eq!(
            sql.sql,
            r#"update "my_tbl" set "a" = $1, "b" = $2, "c" = $3, "f" = $4, "d" = $5"#
        );
        assert_eq!(sql.bindings.len(), 5);
    }

    #[test]
    fn insert() {
        #[derive(unnamed_qb_macro::Row)]
        struct TestRow {
            a: String,
            b: i32,
            c: i64,
            f: bool,
            d: Option<i32>,
        }

        let r = TestRow {
            a: "a_val".to_owned(),
            b: 10,
            c: 20,
            f: false,
            d: None,
        };

        let rs = vec![
            TestRow {
                a: "a_val".to_owned(),
                b: 10,
                c: 20,
                f: false,
                d: None,
            },
            TestRow {
                a: "a_val".to_owned(),
                b: 10,
                c: 20,
                f: false,
                d: None,
            },
            TestRow {
                a: "a_val".to_owned(),
                b: 10,
                c: 20,
                f: false,
                d: None,
            },
        ];

        let mut qb = QueryBuilder::new();
        let insert_qb = qb.insert().into("my_tbl");

        insert_qb.value(r);

        insert_qb.values(rs);

        let sql = insert_qb.sql::<TestDialect>();

        assert_eq!(
            sql.sql,
            r#"insert into "my_tbl" ("a", "b", "c", "f", "d") values ($1, $2, $3, $4, $5), ($6, $7, $8, $9, $10), ($11, $12, $13, $14, $15), ($16, $17, $18, $19, $20)"#
        );
        assert_eq!(sql.bindings.len(), 20);
    }
}