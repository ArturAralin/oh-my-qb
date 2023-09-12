mod macros;
pub mod error;
pub mod prelude;
mod query_builder;
pub mod sql_dialect;

pub use query_builder::raw::RawExt;
pub use query_builder::select::column::ColumnExt;
pub use query_builder::Conditions;
pub use query_builder::QueryBuilder;
pub use query_builder::ValueExt;
pub use unnamed_qb_macro::Row;

#[macro_export]
macro_rules! qb {
  { $q:ident($($ia:expr),*) $(.$f:ident( $( $a:expr ),* ))* } => {
    {
      #[allow(unused)]
      let mut qb = QueryBuilder::$q($($ia)*);

      $(
        qb.$f($($a),*);
      )*

      qb
    }
  }
}
