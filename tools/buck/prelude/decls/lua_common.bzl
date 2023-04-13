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

def _srcs_arg():
    return {
        "srcs": attrs.named_set(attrs.source(), sorted = True, default = [], doc = """
    The set of `.lua` files included in this library.
"""),
    }

def _base_module_arg():
    return {
        "base_module": attrs.option(attrs.string(), default = None, doc = """
    The package for which the given specified sources and resources should reside in their final
     location in the top-level binary. If unset, the project relative directory that houses the
     BUCK file is used.
"""),
    }

lua_common = struct(
    srcs_arg = _srcs_arg,
    base_module_arg = _base_module_arg,
)
