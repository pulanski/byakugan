def buildscript_args(
        name: str.type,
        package_name: str.type,
        buildscript_rule: str.type,
        outfile: str.type,
        version: str.type,
        cfgs: [str.type] = [],
        features: [str.type] = []):
    """
    Generates a buildscript_args file for a given buildscript rule.

    The buildscript_args file is a list of arguments to pass to cargo
    when building the package.  The arguments are generated by running

        cargo rustc -- --print=cfg

    and parsing the output.  The output is a list of rustc cfgs, which
    are passed to cargo as --cfg arguments.

    The buildscript_args file is used by the cargo_build_script rule
    to pass the cfgs to cargo when building the package.

    Args:
        name: The name of the rule.
        package_name: The name of the package.
        buildscript_rule: The name of the buildscript rule.
        outfile: The name of the output file.
        version: The version of the package.
        cfgs: A list of additional cfgs to pass to cargo.
        features: A list of features to pass to cargo.

    Returns:
        A cargo_build_script rule.
    """

    # _ = package_name
    # _ = version
    # _ = cfgs
    # _ = features
    native.genrule(
        name = name,
        out = outfile,
        cmd = "env RUSTC=rustc TARGET= $(exe %s) | sed -n s/^cargo:rustc-cfg=/--cfg=/p > ${OUT}" % buildscript_rule,
    )

# TODO: fixup below

# Invoke something with a default cargo-like environment. This is used to invoke buildscripts
# from within a Buck rule to get it to do whatever it does (typically, either emit command-line
# options for rustc, or generate some source).
# def _make_cmd(
#         mode: str.type,
#         buildscript: str.type,
#         package_name: str.type,
#         version: str.type,
#         features: [str.type],
#         cfgs: [str.type],
#         env: {str.type: str.type},
#         target_override: str.type = None):
#     # type: (str, str, str, str, List[str], List[str], Dict[str, str], Optional[str]) -> str
#     flags = [
#         ("mode", mode),
#         ("buildscript", "$(exe " + buildscript + ")"),
#         ("package-name", package_name),
#         ("version", version),
#         ("feature", features),
#         ("cfg", cfgs),
#         ("env", env),
#         ("target", target_override),
#         # ("target", target_override or _get_native_host_triple()),
#     ]

#     cmd = "$(exe shim//third-party/macros:build_rs)"

#     # We don't want to quote the $OUT flag as it might end in \ on Windows, which would then escape the quote
#     cmd += " --output=$OUT"
#     for flag, value in flags:
#         if value == None:
#             pass
#         elif type(value) == type([]):
#             for x in value:
#                 cmd += " \"--" + flag + "=" + x + "\""
#         elif type(value) == type({}):
#             for k, v in value.items():
#                 cmd += " \"--" + flag + "=" + k + "=" + v + "\""
#         else:
#             cmd += " \"--" + flag + "=" + value + "\""
#     return cmd

# buildscript_args(
#     name = "num-bigint-0.4.3-build-script-build-args",
#     package_name = "num-bigint",
#     buildscript_rule = ":num-bigint-0.4.3-build-script-build",
#     features = [
#         "default",
#         "std",
#     ],
#     outfile = "args.txt",
#     version = "0.4.3",
# )

# def buildscript_srcs(
#         name: str.type,
#         package_name: str.type,
#         buildscript_rule: str.type,
#         files: [str.type],
#         version: str.type,
#         cfgs: [str.type] = [],
#         features: [str.type] = []):
#     # buildscript_srcs(
#     #     name = "num-bigint-0.4.3-build-script-build-srcs",
#     #     package_name = "num-bigint",
#     #     buildscript_rule = ":num-bigint-0.4.3-build-script-build",
#     #     features = [
#     #         "default",
#     #         "std",
#     #     ],
#     #     files = ["radix_bases.rs"],
#     #     version = "0.4.3",
#     # )

#     """
#     Generates a buildscript_srcs file for a given buildscript rule.

#     The buildscript_srcs file is a list of source files to pass to cargo
#     when building the package.

#     The buildscript_srcs file is used by the cargo_build_script rule
#     to pass the srcs to cargo when building the package.

#     Args:
#         name: The name of the rule.
#         package_name: The name of the package.
#         buildscript_rule: The name of the buildscript rule.
#         files: A list of files to pass to cargo.
#         version: The version of the package.
#         cfgs: A list of additional cfgs to pass to cargo.
#         features: A list of features to pass to cargo.
#     """

#     _ = package_name
#     _ = version
#     _ = cfgs
#     _ = features
#     pre = _make_cmd("srcs", buildscript_rule, package_name, version, features, cfgs, env, target)
#     native.cxx_genrule(
#         name = name,
#         srcs = srcs,
#         outs = {file: [file] for file in files},
#         cmd = pre,
#     )
