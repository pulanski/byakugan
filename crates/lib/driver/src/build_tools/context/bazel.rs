use super::is_binary_installed;

/// Check if `bazel` or `bazelisk` is installed and available on the `PATH`.
///
/// **NOTE**: This operation is cached between runs of the program, meaning that
/// the first time this function is called, it will be a blocking operation
/// performed at runtime, however from that point on, it will be a non-blocking
/// operation reading from a cache file.
pub fn is_installed() -> bool {
    is_binary_installed("bazel") || is_binary_installed("bazelisk")
}
