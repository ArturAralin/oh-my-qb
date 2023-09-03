use super::BuildSql;
use crate::query_builder::*;

#[derive(Debug, Default)]
pub struct PostgresSqlDialect<'a> {
    pub sql: String,
    pub bindings: Vec<&'a Value<'a>>,
}

impl<'a> BuildSql<'a> for PostgresSqlDialect<'a> {
    const RELATION_QUOTE: char = '"';

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
