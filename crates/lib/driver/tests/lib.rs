#[cfg(test)]
mod driver_test_suite {
    use driver::build_tools::label::Repo;
    use pretty_assertions_sorted::assert_eq;

    #[test]
    fn test_utils() {
        assert_eq!(1, 1);
    }

    #[test]
    fn test_label() {
        let label = "@foo//bar:baz";
        let repo_component = "@foo";
        let repo = Repo::new(label, repo_component).expect("failed to parse repo");

        assert_eq!(repo.to_string(), repo_component);
    }
}
