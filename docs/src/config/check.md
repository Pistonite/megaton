# Check

The build tool can check the built ELF to help catch issues early,
saving you time debugging crashes.

## Dynamic Symbols
Your module is built as a shared object, which means any unresolved
symbols are treated as dynamic symbols imported at runtime by RTLD.
However, if no such symbol exists, RTLD will abort the process.

The build tool allows you to check the ELF against a list of known
dynamic symbols that exist at runtime. The symbol listing
can be obtained by running `objdump -T` on the other libraries
loaded at runtime.

Once you have the symbols dumped, add the following `check` configuration
to `Megaton.toml`:

```toml
[check]
symbols = [
    "path/to/symbol-file.syms",
    # ... add more symbol files here
]
```
```admonish info
Like other paths in `Megaton.toml`, paths are relative to the project root.
```

Rebuild the project, and the checker should run after linking the ELF
```bash
megaton build
```

If there are false positives, you can add them to the `ignore` list:

```toml
[check]
ignore = [
    "__magic_symbol",
    # ... add more symbols to ignore here
]
symbols = [
    "path/to/symbol-file.syms",
    # ... add more symbol files here
]
```
```admonish tip
Any symbol that starts with `.` are automatically ignored.
```

## Disallowed Instructions

By default, the checker also checks the instructions in the ELF
to make sure there aren't any instructions that will crash
when executed, such as privileged instructions or `hlt`.

You can add more instructions to the disallowed list, by
specifying a regex pattern for the mnemonic of the instruction:

```toml
[check]
disallowed-instructions = [
    # ... add more instructions here
]
```

