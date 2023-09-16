# A list of available rules and their signatures can be found here: https://buck2.build/docs/generated/starlark/prelude/prelude.bzl

alias(
    name = "main",
    actual = "//crates/bin/bkg:bkg",
)

alias(
    name = "rudolph",
    actual = "//crates/bin/rudolph:rudolph",
)

alias(
    name = "pb",
    actual = "//crates/bin/pb:pb",
)

alias(
    name = "pb2",
    actual = "//crates/bin/pb2:pb2",
)

alias(
    name = "incremental_salsa",
    actual = "//crates/bin/incremental_salsa:incremental_salsa",
)

alias(
    name = "pb_cache",
    actual = "//crates/bin/pb_cache:pb_cache",
)

alias(
    name = "nook",
    actual = "//crates/bin/nook:nook",
)

# Generator for libs and bins in the context of Buck2/Bazel - tool: starframe
#
# High level overview:
#
# 1. Lib: (e.g. `sf gen lib //lib/bkg`)
#   - User is prompted to enter a specific kind of lib (e.g. `proc_macro`, `cdylib`, `cxx_library`, `rust_library`, etc.)
#     OR
#     - User can pass a kind of lib via a positional argument (e.g. `sf gen rust_library //lib/bkg`)
#     OR
#     - User can pass a kind of supported language via a positional argument (e.g. `sf gen rust //lib/bkg`)
#       - In this case, the tool will prompt the user to select the desired kind of target (e.g. `rust_library`, `rust_binary`, etc.)
#
# **NOTE**: For the kind of target, any valid API from the Rules API can be used (e.g. `rust_library`,
# `rust_binary`, `rust_test`, `cxx_library`, `cxx_binary`, `cxx_test`, etc.)
#
# **NOTE**: Additional configuration can be passed in via configuration files
#   (e.g. `~/.sfrc` -  Global configuration file, `./.sfrc` - Local configuration file)
#
#   From here, the tool will:
#
#   - Create a `lib` target
#   - Create an associated `test` target
#   - Create an associated `bench` target (if `--bench` is passed)
#   - Create an associated `example` target (if `--example` is passed)
#
#   The below operations will be performed for each of the aforementioned targets:
#     - Generate associated `BUILD/BUCK` files for the targets based on the build system
#     (e.g. `buck`, `bazel`) detected in the current workspace
#
#   - Create an associated alias target at the root of the of the workspace (e.g. `//bkg`)
#      (if `--with-alias` is passed)
#
#   - Will then execute `buck build //lib/bkg` to ensure that the target builds successfully
#
# General algorithm:
#
# 1. Parse command line arguments
# 2. Apply any configuration options (e.g. `--bench`, `--example`, `--with-alias` or from configuration files)
# 3. Determine the kind of target to generate
# 4. Generate the target
# 5. Generate the associated test target
# 6. Generate the associated bench target (if `--bench` is passed)
# 7. Generate the associated example target (if `--example` is passed)
# 8. Generate the associated alias target (if `--with-alias` is passed)
# 9. Execute `buck build //lib/bkg` to ensure that the target builds successfully
#
# 2. Bin: (e.g. `sf gen bin //bin/bkg`)
#   - User is prompted to enter a specific kind of bin (e.g. `rust_binary`, `cxx_binary`, etc.)
#     OR
#     - User can pass a kind of bin via a positional argument (e.g. `sf gen rust_binary //bin/bkg`)
#     OR
#     - User can pass a kind of supported language via a positional argument (e.g. `sf gen rust //bin/bkg`)
#       - In this case, the tool will prompt the user to select the desired kind of target (e.g. `rust_binary`, `rust_test`, etc.)
#
# **NOTE**: For the kind of target, any valid API from the Rules API can be used (e.g. `rust_binary`,
# `rust_test`, `cxx_binary`, `cxx_test`, etc.)
#
# **NOTE**: Additional configuration can be passed in via configuration files
#   (e.g. `~/.sfrc` -  Global configuration file, `./.sfrc` - Local configuration file)
#
#   From here, the tool will:
#
#   - Create a `bin` target
#   - Create an associated `test` target
#
#   The below operations will be performed for each of the aforementioned targets:
#     - Generate associated `BUILD/BUCK` files for the targets based on the build system
#     (e.g. `buck`, `bazel`) detected in the current workspace
#
#   - Create an associated alias target at the root of the of the workspace (e.g. `//bkg`)
#      (this is always done for bins as a convenience for the end user)
#
#   - Will then execute `buck build //bin/bkg` to ensure that the target builds successfully
#
# General algorithm:
#
# 1. Parse command line arguments
# 2. Apply any configuration options (e.g. `--bench`, `--example`, `--with-alias` or from configuration files)
# 3. Determine the kind of target to generate
# 4. Generate the target
# 5. Generate the associated test target
# 6. Generate the associated alias target
# 7. Execute `buck build //bin/bkg` to ensure that the target builds successfully

# load("//:test_utils.bzl", "assert_file_contains")

# assert_file_contains(
#     name = "test",
#     file = ".buckconfig",
#     expected = "[repositories]",
# )

# assert_file_exists(
#     name = "buckconfig",
#     path = ".buckconfig",
# )

python_binary(
    name = "salsa_fixup",
    main = "salsa_fixup.py",
)
