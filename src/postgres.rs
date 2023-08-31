use crate::{
    qb_arg::{Arg, ArgValue, Raw},
    query_builder::{InsertQuery, QueryType, SelectQuery},
    value::Value,
    where_clause::{Condition, ConditionInner, ConditionOp},
    QueryBuilder, Sql,
};

pub trait BuildSql<'a> {
    fn init() -> Self;
    fn build_sql(self, qb: &'a QueryBuilder) -> Sql<'a>;
}

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
        match qb.query_type {
            QueryType::Select(_) => {
                self.build_select(qb);
            }
            QueryType::Delete => {
                self.build_delete(qb);
            }
            QueryType::Insert(_) => {
                self.build_insert(qb);
            }
            _ => {}
        }

        self.build_where(qb);

        Sql {
            sql: self.sql,
            binds: self.bindings,
        }
    }
}

impl<'a> Postgres<'a> {
    fn build_select(&mut self, qb: &'a QueryBuilder) {
        if let QueryType::Select(SelectQuery { columns, .. }) = &qb.query_type {
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

            if let Some(table) = &qb.table {
                self.sql.push_str(" from ");
                // todo: format table name
                self.sql.push_str(table);
            }
        }
    }

    fn build_delete(&mut self, qb: &'a QueryBuilder) {
        if let QueryType::Delete = &qb.query_type {
            self.sql.push_str("delete");

            if let Some(table) = &qb.table {
                self.sql.push_str(" from ");
                // todo: format table name
                self.sql.push_str(table);
            }
        }
    }

    fn build_insert(&mut self, qb: &'a QueryBuilder) {
        if let QueryType::Insert(InsertQuery {
            ordered_columns,
            rows,
            ..
        }) = &qb.query_type
        {
            self.sql.push_str("insert");

            if let Some(table) = &qb.table {
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

    fn build_where(&mut self, qb: &'a QueryBuilder) {
        if qb.where_conditions.is_empty() {
            return;
        }

        self.sql.push_str(" where");

        qb.where_conditions
            .iter()
            .enumerate()
            .for_each(|(idx, condition)| match condition {
                Condition::Group(_) => {}
                Condition::Condition(ConditionInner {
                    op,
                    right,
                    left,
                    middle,
                    ..
                }) => {
                    if idx > 0 {
                        match op {
                            ConditionOp::And => {
                                self.sql.push_str(" and ");
                            }
                            ConditionOp::Or => {
                                self.sql.push_str(" or ");
                            }
                        };
                    }

                    if idx == 0 {
                        self.sql.push(' ');
                    }

                    self.build_arg(left);
                    self.sql.push(' ');
                    self.sql.push_str(middle);
                    self.sql.push(' ');
                    self.build_arg(right);
                }
            });

        // where_clause
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
            Arg::Value(v) => {
                if let ArgValue::Binding((start, end)) = v {
                    for (_, binding_idx) in ((*start)..(*end)).enumerate() {
                        self.sql.push_str(format!("${}", binding_idx).as_str())
                    }
                }
            }
            Arg::Raw(Raw { sql, .. }) => self.sql.push_str(sql),
        }
    }
}
