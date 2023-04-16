use super::is_binary_installed;

/// Check if `cargo` is installed and available on the `PATH`
///
/// **NOTE**: This operation is cached between runs of the program, meaning that
/// the first time this function is called, it will be a blocking operation
/// performed at runtime, however from that point on, it will a fast lookup
/// operation reading from a cached value.
pub fn is_installed() -> bool {
    is_binary_installed("cargo")
}
