use miette::Result;

pub use query::*;
// ,build::*,
// ,run::*,
// ,test::*,

mod query {
    use super::*;

    async fn all_targets() -> Result<Vec<String>> {
        // TODO: in the future, convert this to a Result<Vec<Target>> where Target is a struct
        // that contains target information (e.g. label, rule, etc.)
        todo!()
    }
}
