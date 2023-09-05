use super::BuildSql;
use crate::query_builder::*;
use sqlx::Arguments;

#[derive(Debug, Default)]
pub struct PostgresSqlDialect<'a> {
    pub sql: String,
    pub bindings: Vec<&'a Value<'a>>,
}

impl<'a> BuildSql<'a> for PostgresSqlDialect<'a> {
    const RELATION_QUOTE: char = '"';

    type SqlxQb = sqlx::QueryBuilder<'a, sqlx::postgres::Postgres>;

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

    fn into_sqlx_qb(self) -> Self::SqlxQb {
        let mut args = sqlx::postgres::PgArguments::default();

        self.bindings.into_iter().for_each(|binding| match binding {
            Value::Integer(v) => args.add(v),
            Value::BigInt(v) => args.add(v),
            // todo: check it
            Value::Null => args.add::<Option<i32>>(None),
            Value::String(s) => args.add(s),
            _ => panic!("unsuppoted"),
        });

        sqlx::QueryBuilder::with_arguments(self.sql, args)
    }
}
