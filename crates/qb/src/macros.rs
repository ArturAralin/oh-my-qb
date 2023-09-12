#[cfg(test)]
mod test {
    use crate::{query_builder::{SelectQuery, UpdateQuery}, *};

    #[test]
    fn select_query() {
        let qb = qb! {
          select().from("table")
        };

        matches!(qb, SelectQuery { .. });
    }

    #[test]
    fn update_query() {
        #[derive(Row)]
        struct TestRow {
            s: String,
        }

        let qb = qb! {
          update(TestRow { s: "test".to_owned() })
        };

        matches!(qb, UpdateQuery { .. });
    }
}
