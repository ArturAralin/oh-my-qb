# <qb_name>

Query builder inspired by Knex.js.

# Examples

## select
```rust
let mut qb = QueryBuilder::select();
let result = qb
  .columns(&[
    "table.column1",
    "table.column2"
  ])
  .from("table")
  .and_where("table.id", "=", 10.value())
  .sql::<PostgresSqlDialect>();

result.sql // select "table"."column1", "table"."column2" from "table" where "table"."id" = $1
result.bindings // [Integer(10)]
```

## sub query
```rust
let mut sub_query = QueryBuilder::select();

sub_query
  .columns(&[
    "id",
  ])
  .from("table")
  .and_where("table.type", "=", "my_type".value())

let mut qb = QueryBuilder::select();
let result = qb
  .columns(&[
    "table.column1",
    "table.column2"
  ])
  .from("table")
  .and_where("table.id", "in", sub_query)
  .sql::<PostgresSqlDialect>();

result.sql // select "table"."column1", "table"."column2" from "table" where "table"."id" in (select "id" from "table" where "table"."type" = $1)
result.bindings // [String("my_type")]
```

## insert
```rust
#[derive(Row)]
struct MyRow {
  a: String,
  b: i32
}

let mut qb = QueryBuilder::insert();
let result = qb
  .into_("table")
  .value(MyRow { a: "abc".to_owned(), b: 10 })
  .sql::<PostgresSqlDialect>();

result.sql // insert into ("a", "b") values ($1, $2)
result.bindings // [String("abc"), Integer(10)]
```

# sqlx integration
```rust
let mut qb = QueryBuilder::select();
let sql = qb
  .columns(&[
    "table.column1",
    "table.column2"
  ])
  .from("table")
  .and_where("table.id", "=", 10.value())
  .sqlx_qb::<PostgresSqlDialect>() // here sqlx::QueryBuilder<'_, Postgres>
  .into_sql();

sql // select "table"."column1", "table"."column2" from "table" where "table"."id" = $1
```
