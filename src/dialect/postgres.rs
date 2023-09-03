use crate::query_builder::*;
use crate::query_builder::{
    Arg, ArgValue, ConditionOp, DeleteQuery, QueryBuilder, Raw, SingleWhereCondition, UpdateQuery,
    WhereCondition,
};

use super::{BuildSql, Sql};

#[derive(Debug, Default)]
pub struct Postgres<'a> {
    pub sql: String,
    pub bindings: Vec<&'a Value<'a>>,
}

impl<'a> BuildSql<'a> for Postgres<'a> {
    fn init() -> Self {
        Self::default()
    }

    fn build_sql(mut self, qb: &'a QueryBuilder) -> Sql<'a> {
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

        Sql {
            sql: self.sql,
            binds: self.bindings,
        }
    }
}

impl<'a> Postgres<'a> {
    fn build_select(&mut self, select: &'a SelectQuery) {
        let SelectQuery {
            columns,
            table,
            where_clause,
        } = select;

        self.sql.push_str("select");

        match columns {
            Some(columns) => {
                self.sql.push(' ');

                if columns.is_empty() {
                    self.sql.push('*');
                } else {
                    columns.iter().enumerate().for_each(|(idx, column)| {
                        // todo: handle colum
                        if idx > 0 {
                            self.sql.push(',');
                            self.sql.push(' ');
                        }

                        self.sql.push_str(column);
                    });
                }
            }
            None => {
                self.sql.push(' ');
                self.sql.push('*');
            }
        }

        if let Some(table) = table {
            self.sql.push_str(" from ");
            // todo: format table name
            self.sql.push_str(table);
        }

        self.build_where(where_clause, 0);
    }

    fn build_delete(&mut self, qb: &'a QueryBuilder) {
        if let Query::Delete(DeleteQuery {
            table,
            where_clause,
        }) = &qb.query
        {
            self.sql.push_str("delete");

            if let Some(table) = table {
                self.sql.push_str(" from ");
                // todo: format table name
                self.sql.push_str(table);
            }

            self.build_where(where_clause, 0);
        }
    }

    fn build_update(&mut self, qb: &'a QueryBuilder) {
        if let Query::Update(UpdateQuery {
            columns,
            table,
            where_clause,
        }) = &qb.query
        {
            self.sql.push_str("update");

            if let Some(table) = table {
                self.sql.push(' ');
                // todo: format table name
                self.sql.push_str(table);
            }

            self.sql.push_str(" set");

            columns
                .iter()
                .enumerate()
                .for_each(|(idx, (column, binding_idx))| {
                    if idx > 0 {
                        self.sql.push(',');
                    }

                    self.sql.push(' ');
                    self.sql.push_str(column);
                    self.sql.push_str(" = $");
                    self.sql.push_str(format!("{}", binding_idx).as_str());
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
            self.sql.push_str("insert");

            if let Some(table) = table {
                self.sql.push_str(" into ");
                // todo: format table name
                self.sql.push_str(table);
            }

            if let Some(ordered_columns) = ordered_columns {
                self.sql.push(' ');
                self.sql.push('(');

                ordered_columns
                    .iter()
                    .enumerate()
                    .for_each(|(idx, column)| {
                        if idx > 0 {
                            self.sql.push(',');
                            self.sql.push(' ');
                        }

                        self.sql.push_str(format!(r#""{}""#, column).as_str());
                    });

                self.sql.push(')');
            }

            if !rows.is_empty() {
                self.sql.push_str(" values ");

                rows.iter().enumerate().for_each(|(row_idx, (start, end))| {
                    if row_idx > 0 {
                        self.sql.push(',');
                        self.sql.push(' ');
                    }

                    self.sql.push('(');
                    for (idx, bind_idx) in ((*start)..(*end)).enumerate() {
                        if idx > 0 {
                            self.sql.push(',');
                            self.sql.push(' ');
                        }
                        self.sql.push_str(format!("${}", bind_idx).as_str());
                    }
                    self.sql.push(')');
                });
            }
        }
    }

    fn build_where(&mut self, where_conditions: &'a [WhereCondition<'a>], depth: usize) {
        if depth == 0 {
            self.sql.push_str(" where");
        }

        where_conditions
            .iter()
            .enumerate()
            .for_each(|(idx, condition)| match condition {
                WhereCondition::Group(GroupedWhereCondition { op, conditions }) => {
                    if idx > 0 {
                        match op {
                            ConditionOp::And => {
                                self.sql.push_str(" and");
                            }
                            ConditionOp::Or => {
                                self.sql.push_str(" or");
                            }
                        };
                    }

                    self.sql.push(' ');
                    self.sql.push('(');
                    self.build_where(conditions, depth + 1);
                    self.sql.push(')');
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
                                self.sql.push_str(" and");
                            }
                            ConditionOp::Or => {
                                self.sql.push_str(" or");
                            }
                        };
                    }

                    self.sql.push(' ');
                    self.build_arg(left);
                    self.sql.push(' ');
                    self.sql.push_str(middle);
                    self.sql.push(' ');
                    self.build_arg(right);
                }
            });
    }

    fn build_arg(&mut self, arg: &'a Arg) {
        match arg {
            Arg::Column(c) => {
                let col =
                    c.0.split('.')
                        .map(|col| format!(r#""{col}""#))
                        .collect::<Vec<_>>()
                        .join(".");

                self.sql.push_str(col.as_str());
            }
            Arg::Value(ArgValue::Binding((start, end))) => {
                let count = end - start;

                if count > 1 {
                    self.sql.push('(');
                }

                for (idx, binding_idx) in (*start..*end).enumerate() {
                    if idx > 0 && count > 1 {
                        self.sql.push(',');
                        self.sql.push(' ');
                    }

                    self.sql.push_str(format!("${}", binding_idx).as_str())
                }

                if count > 1 {
                    self.sql.push(')');
                }
            }
            Arg::Value(ArgValue::Value(Value::Null)) => {
                self.sql.push_str("null");
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
                        self.sql.push_str(format!("${}", idx).as_str());
                    } else {
                        self.sql.push(ch);
                    }
                }
            }
            _ => {
                unreachable!("Invalid case reached")
            }
        }
    }
}
