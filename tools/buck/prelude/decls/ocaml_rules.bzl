# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is licensed under both the MIT license found in the
# LICENSE-MIT file in the root directory of this source tree and the Apache
# License, Version 2.0 found in the LICENSE-APACHE file in the root directory
# of this source tree.

# TODO(cjhopman): This was generated by scripts/hacks/rules_shim_with_docs.py,
# but should be manually editted going forward. There may be some errors in
# the generated docs, and so those should be verified to be accurate and
# well-formatted (and then delete this TODO)

load(":common.bzl", "buck", "prelude_rule")
load(":ocaml_common.bzl", "ocaml_common")

ocaml_binary = prelude_rule(
    name = "ocaml_binary",
    docs = """
        A ocaml\\_binary() rule builds both native and bytecode executables from the supplied set of OCaml and C source files
         and dependencies.


         Note: Buck is currently tested with 4.X OCaml series.
    """,
    examples = """
        For more examples, check out our [integration tests](https://github.com/facebook/buck/tree/dev/test/com/facebook/buck/features/ocaml/testdata/).


        ```

        ocaml_binary(
          name='greet',
          srcs=[
            'main.ml',
            'lex.mll',
            'parser.mly',
            'hashtable.c',
          ],
          deps=[
            ':greeting',
            ':bridge',
          ],
        )

        ocaml_library(
          name='greeting',
          srcs=[
            'greeting.ml',
          ],
          deps=[
            ':join',
          ],
        )

        ocaml_library(
          name='join',
          srcs=[
            'join.ml',
          ],
        )

        cxx_library(
          name='bridge',
          srcs=[
            'bridge.c',
          ],
        )

        ```
    """,
    further = None,
    attrs = (
        # @unsorted-dict-items
        ocaml_common.srcs_arg() |
        ocaml_common.deps_arg() |
        buck.platform_deps_arg() |
        ocaml_common.compiler_flags_arg() |
        {
            "bytecode_only": attrs.option(attrs.bool(), default = None),
            "contacts": attrs.list(attrs.string(), default = []),
            "default_host_platform": attrs.option(attrs.configuration_label(), default = None),
            "labels": attrs.list(attrs.string(), default = []),
            "licenses": attrs.list(attrs.source(), default = []),
            "linker_flags": attrs.list(attrs.string(), default = []),
            "ocamldep_flags": attrs.list(attrs.arg(), default = []),
            "platform": attrs.option(attrs.string(), default = None),
            "platform_compiler_flags": attrs.list(attrs.tuple(attrs.regex(), attrs.list(attrs.arg())), default = []),
            "platform_linker_flags": attrs.list(attrs.tuple(attrs.regex(), attrs.list(attrs.string())), default = []),
            "warnings_flags": attrs.option(attrs.string(), default = None),
            "within_view": attrs.option(attrs.option(attrs.list(attrs.string())), default = None),
        }
    ),
)

ocaml_library = prelude_rule(
    name = "ocaml_library",
    docs = """
        A ocaml\\_library() rule builds a native and a bytecode libraries from the
         supplied set of OCaml source files and dependencies.


         Note: Buck is currently tested with 4.X OCaml series.
    """,
    examples = """
        For more examples, check out our [integration tests](https://github.com/facebook/buck/tree/dev/test/com/facebook/buck/features/ocaml/testdata/).


        ```

        ocaml_library(
          name='greeting',
          srcs=[
            'greeting.ml',
          ],
          deps=[
            ':join',
          ],
        )

        ```
    """,
    further = None,
    attrs = (
        # @unsorted-dict-items
        ocaml_common.srcs_arg() |
        ocaml_common.deps_arg() |
        buck.platform_deps_arg() |
        ocaml_common.compiler_flags_arg() |
        {
            "bytecode_only": attrs.bool(default = False),
            "contacts": attrs.list(attrs.string(), default = []),
            "default_host_platform": attrs.option(attrs.configuration_label(), default = None),
            "labels": attrs.list(attrs.string(), default = []),
            "licenses": attrs.list(attrs.source(), default = []),
            "linker_flags": attrs.list(attrs.arg(), default = []),
            "native_plugin": attrs.bool(default = False),
            "ocamldep_flags": attrs.list(attrs.arg(), default = []),
            "platform_compiler_flags": attrs.list(attrs.tuple(attrs.regex(), attrs.list(attrs.arg())), default = []),
            "warnings_flags": attrs.option(attrs.string(), default = None),
            "within_view": attrs.option(attrs.option(attrs.list(attrs.string())), default = None),
        }
    ),
)

prebuilt_ocaml_library = prelude_rule(
    name = "prebuilt_ocaml_library",
    docs = "",
    examples = None,
    further = None,
    attrs = (
        # @unsorted-dict-items
        {
            "bytecode_c_libs": attrs.list(attrs.string(), default = []),
            "bytecode_lib": attrs.option(attrs.string(), default = None),
            "bytecode_only": attrs.bool(default = False),
            "c_libs": attrs.list(attrs.string(), default = []),
            "contacts": attrs.list(attrs.string(), default = []),
            "default_host_platform": attrs.option(attrs.configuration_label(), default = None),
            "deps": attrs.list(attrs.dep(), default = []),
            "include_dir": attrs.string(default = ""),
            "labels": attrs.list(attrs.string(), default = []),
            "lib_dir": attrs.string(default = ""),
            "lib_name": attrs.string(default = ""),
            "licenses": attrs.list(attrs.source(), default = []),
            "native_c_libs": attrs.list(attrs.string(), default = []),
            "native_lib": attrs.option(attrs.string(), default = None),
            "platform_deps": attrs.list(attrs.tuple(attrs.regex(), attrs.set(attrs.dep(), sorted = True)), default = []),
            "within_view": attrs.option(attrs.option(attrs.list(attrs.string())), default = None),
        }
    ),
)

ocaml_rules = struct(
    ocaml_binary = ocaml_binary,
    ocaml_library = ocaml_library,
    prebuilt_ocaml_library = prebuilt_ocaml_library,
)
