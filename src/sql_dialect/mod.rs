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

    fn push_sql_str(&mut self, sql: &str);
    fn push_sql_char(&mut self, ch: char);
    fn push_binding(&mut self, value: &'a Value<'a>) -> usize;

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
                self.push_sql_char('.');
            }

            self.push_sql_char(Self::RELATION_QUOTE);
            self.push_sql_str(relation_part);
            self.push_sql_char(Self::RELATION_QUOTE);
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
        } = select;

        self.push_sql_str("select");

        match columns {
            Some(columns) => {
                self.push_sql_char(' ');

                if columns.is_empty() {
                    self.push_sql_char('*');
                } else {
                    columns.iter().enumerate().for_each(|(idx, column)| {
                        if idx > 0 {
                            self.push_sql_char(',');
                            self.push_sql_char(' ');
                        }

                        self.write_relation(column);
                    });
                }
            }
            None => {
                self.push_sql_char(' ');
                self.push_sql_char('*');
            }
        }

        if let Some(table) = table {
            self.push_sql_str(" from ");
            self.write_relation(table);
        }

        if let Some(joins) = joins {
            joins.iter().for_each(|join| {
                let Join::Regular(join) = join;

                if let Some(join_type) = join.join_type {
                    self.push_sql_char(' ');
                    self.push_sql_str(join_type);
                }

                self.push_sql_str(" join ");
                self.write_relation(&join.table);
                self.push_sql_str(" on ");
                self.write_arg(&join.left);
                self.push_sql_char(' ');
                self.push_sql_str(&join.op);
                self.push_sql_char(' ');
                self.write_arg(&join.right);
            });
        }

        self.build_where(where_clause, 0);

        if let Some(limit) = limit {
            self.push_sql_str(" limit ");
            self.push_sql_str(limit.to_string().as_str());
        }

        if let Some(offset) = offset {
            self.push_sql_str(" offset ");
            self.push_sql_str(offset.to_string().as_str());
        }
    }

    fn build_delete(&mut self, qb: &'a QueryBuilder<'a>) {
        if let Query::Delete(DeleteQuery {
            table,
            where_clause,
        }) = &qb.query
        {
            self.push_sql_str("delete");

            if let Some(table) = table {
                self.push_sql_str(" from ");
                self.write_relation(table);
            }

            self.build_where(where_clause, 0);
        }
    }

    fn build_update(&mut self, qb: &'a QueryBuilder<'a>) {
        if let Query::Update(UpdateQuery {
            columns,
            table,
            where_clause,
        }) = &qb.query
        {
            self.push_sql_str("update");

            if let Some(table) = table {
                self.push_sql_char(' ');
                self.write_relation(table);
            }

            self.push_sql_str(" set");

            columns
                .iter()
                .enumerate()
                .for_each(|(idx, (column, binding_idx))| {
                    if idx > 0 {
                        self.push_sql_char(',');
                    }

                    self.push_sql_char(' ');
                    self.push_sql_str(column);
                    self.push_sql_str(" = $");
                    self.push_sql_str(format!("{}", binding_idx).as_str());
                });

            self.build_where(where_clause, 0);
        }
    }

    fn build_insert(&mut self, qb: &'a QueryBuilder) {
        if let Query::Insert(InsertQuery {
            ordered_columns,
            rows,
            table,
        }) = &qb.query
        {
            self.push_sql_str("insert");

            if let Some(table) = table {
                self.push_sql_str(" into ");
                self.write_relation(table);
            }

            if let Some(ordered_columns) = ordered_columns {
                self.push_sql_char(' ');
                self.push_sql_char('(');

                ordered_columns
                    .iter()
                    .enumerate()
                    .for_each(|(idx, column)| {
                        if idx > 0 {
                            self.push_sql_char(',');
                            self.push_sql_char(' ');
                        }

                        self.push_sql_str(format!(r#""{}""#, column).as_str());
                    });

                self.push_sql_char(')');
            }

            if !rows.is_empty() {
                self.push_sql_str(" values ");

                rows.iter().enumerate().for_each(|(row_idx, (start, end))| {
                    if row_idx > 0 {
                        self.push_sql_char(',');
                        self.push_sql_char(' ');
                    }

                    self.push_sql_char('(');
                    for (idx, bind_idx) in ((*start)..(*end)).enumerate() {
                        if idx > 0 {
                            self.push_sql_char(',');
                            self.push_sql_char(' ');
                        }
                        self.push_sql_str(format!("${}", bind_idx).as_str());
                    }
                    self.push_sql_char(')');
                });
            }
        }
    }

    fn build_where(&mut self, where_conditions: &'a [WhereCondition<'a>], depth: usize) {
        if !where_conditions.is_empty() && depth == 0 {
            self.push_sql_str(" where");
        }

        where_conditions
            .iter()
            .enumerate()
            .for_each(|(idx, condition)| match condition {
                WhereCondition::Group(GroupedWhereCondition { op, conditions }) => {
                    if idx > 0 {
                        match op {
                            ConditionOp::And => {
                                self.push_sql_str(" and");
                            }
                            ConditionOp::Or => {
                                self.push_sql_str(" or");
                            }
                        };
                    }

                    self.push_sql_char(' ');
                    self.push_sql_char('(');
                    self.build_where(conditions, depth + 1);
                    self.push_sql_char(')');
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
                                self.push_sql_str(" and");
                            }
                            ConditionOp::Or => {
                                self.push_sql_str(" or");
                            }
                        };
                    }

                    self.push_sql_char(' ');
                    self.write_arg(left);
                    self.push_sql_char(' ');
                    self.push_sql_str(middle);
                    self.push_sql_char(' ');
                    self.write_arg(right);
                }
            });
    }

    fn write_arg(&mut self, arg: &'a Arg<'a>) {
        match arg {
            Arg::Column(c) => {
                let col =
                    c.0.split('.')
                        .map(|col| format!(r#""{col}""#))
                        .collect::<Vec<_>>()
                        .join(".");

                self.push_sql_str(col.as_str());
            }
            Arg::Value(ArgValue::Binding((start, end))) => {
                let count = end - start;

                if count > 1 {
                    self.push_sql_char('(');
                }

                for (idx, binding_idx) in (*start..*end).enumerate() {
                    if idx > 0 && count > 1 {
                        self.push_sql_char(',');
                        self.push_sql_char(' ');
                    }

                    self.push_sql_str(format!("${}", binding_idx).as_str())
                }

                if count > 1 {
                    self.push_sql_char(')');
                }
            }
            Arg::Value(ArgValue::Value(Value::Null)) => {
                self.push_sql_str("null");
            }
            Arg::Value(ArgValue::Value(a)) => {
                let idx = self.push_binding(a);
                self.push_sql_char('$');
                self.push_sql_str(idx.to_string().as_str());
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
                        self.push_sql_str(format!("${}", idx).as_str());
                    } else {
                        self.push_sql_char(ch);
                    }
                }
            }
            Arg::SubQuery(x) => {
                self.push_sql_char('(');
                self.build_sql(&x.0);
                self.push_sql_char(')');
            }
            _ => {
                unreachable!("Invalid case reached")
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

        fn push_sql_char(&mut self, ch: char) {
            self.sql.push(ch);
        }

        fn push_sql_str(&mut self, sql: &str) {
            self.sql.push_str(sql);
        }

        fn push_binding(&mut self, value: &'a Value<'a>) -> usize {
            self.bindings.push(value);
            self.bindings.len()
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
}
