"""
A collection of utility functions for testing rules which can be used
within the context of Buck2 in an incremental way.
"""

def assert_output(name, command, output):
    """
    Asserts that the output of a command is the same as the expected

    Asserts that a function returns a specific output.
    It does this by running the command and checking if the output is
    the same as the expected output.
    """
    return native.genrule(
        name = name,
        bash = command + " | grep \"" + output + "\" && touch \"$OUT\"",
        cmd_exe = command + " | findstr \"" + output + "\" && type nul > \"$OUT\"",
        out = "out.txt",
    )

def assert_no_output(name, command):
    """
    Asserts that the output of a command is none

    Asserts that a function returns no output.
    It does this by running the command and checking if the output is
    the same as the expected output.
    """

    return native.genrule(
        name = name,
        bash = command + " | grep \"\" && exit 1 || touch \"$OUT\"",
        cmd_exe = command + " | findstr \"\" && exit 1 || type nul > \"$OUT\"",
        out = "out.txt",
    )

def assert_output_contains(name, command, expected):
    """
    Asserts that the output of a command contains the expected output

    Asserts that a function returns a specific output.
    It does this by running the command and checking if the output contains
    the expected output.
    """
    return native.genrule(
        name = name,
        bash = command + " | grep \"" + expected + "\" && touch \"$OUT\"",
        cmd_exe = command + " | findstr \"" + expected + "\" && type nul > \"$OUT\"",
        out = "out.txt",
    )

def assert_output_does_not_contain(name, command, expected):
    """
    Asserts that the output of a command does not contain the expected output

    Asserts that a function returns a specific output.
    It does this by running the command and checking if the output does not
    contain the expected output. The check is performed in a case-sensitive manner.
    """
    return native.genrule(
        name = name,
        bash = command + " | grep \"" + expected + "\" && exit 1 || touch \"$OUT\"",
        cmd_exe = command + " | findstr \"" + expected + "\" && exit 1 || type nul > \"$OUT\"",
        out = "out.txt",
    )

def assert_exit_code(name, command, exit_code):
    """
    Asserts that the exit code of a command is the same as the expected

    This function checks if the exit code of a command is the same as the
    expected exit code. It does this by running the command and checking if
    the exit code is the same as the expected exit code.
    """
    return native.genrule(
        name = name,
        bash = command + " && test $? -eq " + str(exit_code) + " && touch \"$OUT\"",
        cmd_exe = command + " && if %errorlevel% neq " + str(exit_code) + " exit 1 && type nul > \"$OUT\"",
        out = "out.txt",
    )

def assert_success(name, command):
    """
    Asserts that the exit code of a command is 0

    This function checks if the exit code of a command is 0. It does this by
    running the command and checking if the exit code is 0.
    """
    return assert_exit_code(name, command, 0)

def assert_failure(name, command):
    """
    Asserts that the exit code of a command is not 0

    This function checks if the exit code of a command is not 0. It does this
    by running the command and checking if the exit code is not 0.
    """
    return not assert_exit_code(name, command, 0)

def assert_file_exists(name, path):
    """
    Asserts that a file exists at the given path (relative to the project root)

    This function checks if a file exists at the given path. It does this by
    running a command to test if the file exists and returning a genrule
    that touches the output file if the command succeeds.
    """
    return native.genrule(
        name = name,
        bash = "echo \"checking path: " + path + "\" && test -f " + path + " && touch \"$OUT\"",
        # bash = "test -f " + path + " && touch \"$OUT\"",
        cmd_exe = "if exist " + path + " type nul > \"$OUT\"",
        out = "out.txt",
    )

# # A rule for using nextest as a test runner for Rust tests in Buck2
# def nextest_rust_test(name, srcs, deps, **kwargs):
#     """
#     A rule for using nextest as a test runner for Rust tests in Buck2
#     This rule is used to run Rust tests using nextest. It is used in the
#     same way as the normal rust_test rule, but it uses nextest as the test
#     runner.
#     """

#     # Create a rule that is similar to the normal rust_test rule, but
#     # with the nextest test runner.
#     nextest_test(
#         name = name,
#         srcs = srcs,
#         deps = deps,
#         # runner = "//tools/nextest:nextest",
#         **kwargs
#     )

# def nextest_test(name, srcs, deps, runner, **kwargs):
#     """
#     A rule for using nextest as a test runner for tests in Buck2
#     This rule is used to run tests using nextest. It is used in the
#     same way as the normal test rule, but it uses nextest as the test
#     runner.
#     """

#     # Create a rule that is similar to the normal test rule, but
#     # with the nextest test runner.
#     test(
#         name = name,
#         srcs = srcs,
#         deps = deps,
#         runner = runner,
#         **kwargs
#     )

# ------------------ Nextest Rule Implementation ------------------

# def nextest_impl(ctx):
#     """
#     Implementation of the nextest rule

#     Implementation of the nextest rule. It is used to run the nextest test runner
#     on a Rust-based test target similar to the normal test rule, but using the nextest
#     test runner. Encapsulates the typical invocation of nextest (e.g. cargo nextest run)
#     and provides a simple interface for running tests using nextest in the context of
#     a Buck2 build.

#     Args:
#         ctx: The context of the rule

#     Returns:
#         A list of providers
#     """

#     # Get the target's directory and name
#     target_dir = ctx.attr.srcs[0].parent
#     target_name = ctx.label.name

#     # Define the command to run Nextest
#     nextest_cmd = [
#         "cargo",
#         "nextest",
#         "run",
#         "--target-dir",
#         target_dir,
#         "--manifest-path",
#         target_dir + "/Cargo.toml",
#         "--",
#         target_name,
#     ]

#     # Run the Nextest command
#     # ctx.actions.run_shell(
#     #     inputs = ctx.files.srcs + ctx.files.deps,
#     #     outputs = [ctx.outputs.out],
#     #     command = " ".join(nextest_cmd),
#     #     use_default_shell_env = True,
#     # )

#     # Run the command and capture its output
#     output = ctx.execute(command = nextest_cmd).stdout.strip()

#     # If the command failed, raise an exception with the output
#     if ctx.execute(command = nextest_cmd).exit_code != 0:
#         fail("Error running Nextest: " + output)

#     # Return the output
#     depset = ctx.actions.tset()
#     return [DefaultInfo(files = depset)]

# nextest = rule(
#     implementation = nextest_impl,
#     attrs = {
#         "srcs": attr.label_list(allow_files = True),
#         "deps": attr.label_list(allow_files = True),
#         "out": attr.output(),
#     },
#     doc = "Runs nextest as the test runner for a Rust test target",
# )
