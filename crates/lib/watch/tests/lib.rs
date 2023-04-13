#[cfg(test)]
mod watch_test_suite {
    #[test]
    fn test_greeting() {
        assert_eq!("Hello, World!", watch::greeting());
    }
}
