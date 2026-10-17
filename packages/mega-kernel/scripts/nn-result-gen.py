#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
# Copyright (c) 2025-2026 Megaton contributors
"""
Generate the Rust binding for nn::Result module and description codes.

The data is scraped from the Atmosphere libvapours result headers vendored in
`include/vapours/results` (refreshed by `task update-lib`), which declare every
result through the `R_DEFINE_*` macro family.

Only concrete results (`R_DEFINE_ERROR_RESULT` and `R_DEFINE_ERROR_RESULT_NS`)
become enum variants; the `*_RANGE` and `*_ABSTRACT_*` macros describe spans of
description codes rather than single ones and are ignored.

Usage: python3 scripts/nn-result-gen.py
"""

import re
from pathlib import Path

ROOT = Path(__file__).parent.parent
RESULTS_DIR = ROOT / "include" / "vapours" / "results"
OUT_FILE = ROOT / "src" / "sys" / "nn_result_gen.rs"

# see ams::result::impl::ResultTraits in results_common.hpp
MODULE_BITS = 9
DESCRIPTION_BITS = 13

# R_DEFINE_NAMESPACE_RESULT_MODULE(ams::fs, 2);
MODULE_RE = re.compile(r"R_DEFINE_NAMESPACE_RESULT_MODULE\(\s*([\w:]+)\s*,\s*(\d+)\s*\)")
# R_DEFINE_ERROR_RESULT(PathNotFound, 1);
RESULT_RE = re.compile(r"R_DEFINE_ERROR_RESULT\(\s*(\w+)\s*,\s*(\d+)\s*\)")
# R_DEFINE_ERROR_RESULT_NS(cheat, CheatNotAttached, 6500);
RESULT_NS_RE = re.compile(r"R_DEFINE_ERROR_RESULT_NS\(\s*\w+\s*,\s*(\w+)\s*,\s*(\d+)\s*\)")
# marks the point in a header past which every result is an Atmosphere extension
AMS_RE = re.compile(r"/[/*].*(atmosphere|not official)", re.IGNORECASE)

DERIVE = "#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]"


class Module:
    def __init__(self, namespace: str, value: int, source: str):
        self.namespace = namespace  # e.g. "ams::sf::hipc"
        self.value = value
        self.source = source
        self.name = namespace.rsplit("::", 1)[-1]  # e.g. "hipc"
        self.descriptions: list[tuple[str, int]] = []

    @property
    def variant(self) -> str:
        """Variant name in `nn_ResultModule`."""
        return self.name[0].upper() + self.name[1:]

    @property
    def rust_type(self) -> str:
        """Name of the generated description enum."""
        return f"nn_ResultDescription_{self.name}"

    def add(self, name: str, value: int, ams_ext: bool):
        if ams_ext:
            name = f"AMS_{name}"
        self.descriptions.append((name, value))

    def validate(self):
        if self.value >= (1 << MODULE_BITS):
            raise ValueError(f"{self.source}: module {self.namespace} does not fit in {MODULE_BITS} bits")
        seen_names: dict[str, int] = {}
        seen_values: dict[int, str] = {}
        for name, value in self.descriptions:
            if value >= (1 << DESCRIPTION_BITS):
                raise ValueError(f"{self.source}: {name} = {value} does not fit in {DESCRIPTION_BITS} bits")
            if name in seen_names:
                raise ValueError(f"{self.source}: duplicated description name {name}")
            if value in seen_values:
                raise ValueError(f"{self.source}: {name} and {seen_values[value]} are both description {value}")
            seen_names[name] = value
            seen_values[value] = name
        self.descriptions.sort(key=lambda d: d[1])


def parse_header(path: Path) -> Module | None:
    """Parse one `*_results.hpp`. Returns None if the header declares no result module."""
    module = None
    ams_ext = False
    for line in path.read_text().splitlines():
        if AMS_RE.search(line):
            # the extension sections all run to the end of the header
            ams_ext = True
        found = MODULE_RE.search(line)
        if found:
            if module is not None:
                raise ValueError(f"{path.name}: more than one result module in a header")
            module = Module(found.group(1), int(found.group(2)), path.name)
            continue
        results = [(m.group(1), int(m.group(2))) for m in RESULT_RE.finditer(line)]
        results += [(m.group(1), int(m.group(2))) for m in RESULT_NS_RE.finditer(line)]
        if results:
            if module is None:
                raise ValueError(f"{path.name}: result defined before the module is declared")
            for name, value in results:
                module.add(name, value, ams_ext)
    return module


def parse_all() -> list[Module]:
    modules: list[Module] = []
    seen: dict[int, Module] = {}
    for path in sorted(RESULTS_DIR.glob("*_results.hpp")):
        if path.name == "results_common.hpp":
            continue
        module = parse_header(path)
        if module is None:
            # headers such as cal_results.hpp only re-export results of other modules
            continue
        module.validate()
        if module.value in seen:
            raise ValueError(f"{path.name}: module {module.value} already declared by {seen[module.value].source}")
        seen[module.value] = module
        modules.append(module)
    modules.sort(key=lambda m: m.value)
    return modules


def emit(modules: list[Module]) -> str:
    out: list[str] = []
    w = out.append

    w("// SPDX-License-Identifier: GPL-3.0-or-later")
    w("// Copyright (c) 2025-2026 Megaton contributors")
    w("// * * * * *")
    w("// This file is autogenerated by scripts/nn-result-gen.py. Don't modify this manually.")
    w("//")
    w("// The module and description codes are scraped from the Atmosphere libvapours")
    w("// headers in include/vapours/results (see include/LICENSE-libvapours).")
    w("#![allow(non_camel_case_types, non_snake_case, dead_code)]")
    w("")
    w("/// Metadata of a description enum generated for a single [`nn_ResultModule`].")
    w("pub trait nn_ResultDescription: Sized + Copy + 'static {")
    w("    /// The module these descriptions belong to")
    w("    const MODULE: nn_ResultModule;")
    w("")
    w("    /// Get the module these descriptions belong to")
    w("    #[inline]")
    w("    fn module() -> nn_ResultModule {")
    w("        Self::MODULE")
    w("    }")
    w("")
    w("    /// Convert a raw description code, `None` if it is not a known description")
    w("    fn from_raw(raw: u32) -> Option<Self>;")
    w("")
    w("    /// Get the raw description code")
    w("    fn to_raw(self) -> u32;")
    w("}")
    w("")
    w("/// Module part of a `nn::Result`")
    w("#[repr(u32)]")
    w("#[non_exhaustive]")
    w(DERIVE)
    w("pub enum nn_ResultModule {")
    for module in modules:
        w(f"    /// `{module.namespace}`")
        w(f"    {module.variant} = {module.value},")
    w("}")
    w("")
    w("impl nn_ResultModule {")
    w("    /// Convert a raw module code, `None` if it is not a known module")
    w("    pub const fn from_raw(raw: u32) -> Option<Self> {")
    w("        match raw {")
    for module in modules:
        w(f"            {module.value} => Some(Self::{module.variant}),")
    w("            _ => None,")
    w("        }")
    w("    }")
    w("")
    w("    /// Get the raw module code")
    w("    #[inline]")
    w("    pub const fn to_raw(self) -> u32 {")
    w("        self as u32")
    w("    }")
    w("}")

    for module in modules:
        w("")
        if not module.descriptions:
            w(f"// `{module.namespace}` (module {module.value}) only declares ranges, no descriptions")
            continue
        w(f"/// Description part of a `nn::Result` of [`nn_ResultModule::{module.variant}`]")
        w("///")
        w(f"/// (module {module.value}, `{module.namespace}`)")
        w("#[repr(u32)]")
        w("#[non_exhaustive]")
        w(DERIVE)
        w(f"pub enum {module.rust_type} {{")
        for name, value in module.descriptions:
            w(f"    {name} = {value},")
        w("}")
        w("")
        w(f"impl nn_ResultDescription for {module.rust_type} {{")
        w(f"    const MODULE: nn_ResultModule = nn_ResultModule::{module.variant};")
        w("")
        w("    fn from_raw(raw: u32) -> Option<Self> {")
        w("        match raw {")
        for name, value in module.descriptions:
            w(f"            {value} => Some(Self::{name}),")
        w("            _ => None,")
        w("        }")
        w("    }")
        w("")
        w("    #[inline]")
        w("    fn to_raw(self) -> u32 {")
        w("        self as u32")
        w("    }")
        w("}")

    w("")
    return "\n".join(out)


def main():
    modules = parse_all()
    OUT_FILE.write_text(emit(modules))
    count = sum(len(m.descriptions) for m in modules)
    print(f"{OUT_FILE.relative_to(ROOT)}: {len(modules)} modules, {count} descriptions")

if __name__ == "__main__":
    main();
