use super::BuildSql;
use crate::query_builder::*;

#[derive(Debug, Default)]
pub struct PostgresSqlDialect<'a> {
    pub sql: String,
    pub bindings: Vec<&'a Value<'a>>,
}

fn write_relation(sql: &mut String, relation: &str) {
    for (idx, relation_part) in relation.split('.').enumerate() {
        if idx > 0 {
            sql.push('.');
        }

        sql.push('"');
        sql.push_str(relation_part);
        sql.push('"');
    }
}

impl<'a> BuildSql<'a> for PostgresSqlDialect<'a> {
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
}

impl<'a> PostgresSqlDialect<'a> {
    fn build_select(&mut self, select: &'a SelectQuery<'a>) {
        let SelectQuery {
            columns,
            table,
            where_clause,
            limit,
            offset,
        } = select;

        self.sql.push_str("select");

        match columns {
            Some(columns) => {
                self.sql.push(' ');

                if columns.is_empty() {
                    self.sql.push('*');
                } else {
                    columns.iter().enumerate().for_each(|(idx, column)| {
                        if idx > 0 {
                            self.sql.push(',');
                            self.sql.push(' ');
                        }

                        write_relation(&mut self.sql, column);
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
            write_relation(&mut self.sql, table);
        }

        self.build_where(where_clause, 0);

        if let Some(limit) = limit {
            self.sql.push_str(" limit ");
            self.sql.push_str(limit.to_string().as_str());
        }

        if let Some(offset) = offset {
            self.sql.push_str(" offset ");
            self.sql.push_str(offset.to_string().as_str());
        }
    }

    fn build_delete(&mut self, qb: &'a QueryBuilder<'a>) {
        if let Query::Delete(DeleteQuery {
            table,
            where_clause,
        }) = &qb.query
        {
            self.sql.push_str("delete");

            if let Some(table) = table {
                self.sql.push_str(" from ");
                write_relation(&mut self.sql, table);
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
            self.sql.push_str("update");

            if let Some(table) = table {
                self.sql.push(' ');
                write_relation(&mut self.sql, table);
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
                write_relation(&mut self.sql, table);
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
        if !where_conditions.is_empty() && depth == 0 {
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

    fn build_arg(&mut self, arg: &'a Arg<'a>) {
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
            Arg::Value(ArgValue::Value(a)) => {
                self.bindings.push(a);
                self.sql.push('$');
                self.sql.push_str(self.bindings.len().to_string().as_str());
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
            Arg::SubQuery(x) => {
                self.sql.push('(');
                self.build_sql(&x.0);
                self.sql.push(')');
            }
            _ => {
                unreachable!("Invalid case reached")
            }
        }
    }
}
