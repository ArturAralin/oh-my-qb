pub mod error;
pub mod prelude;
mod query_builder;
pub mod sql_dialect;

pub use query_builder::select::column::ColumnExt;
pub use query_builder::Conditions;
pub use query_builder::QueryBuilder;
pub use query_builder::RawExt;
pub use query_builder::ValueExt;

// fn test() {
//     // scope(QueryBuilder::select, |select| {
//     //   select.columns(columns)
//     // })

//     // select!().columns();
//     // update!(row)
// }
