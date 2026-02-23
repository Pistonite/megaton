Configuration for the Megaton build tool and library.

Example:
```toml
[megaton]
version = "1"
```

### Key: `megaton.version` (string)
Version of Megaton the project is supposed to use. Megaton will abort if its minor version does not
match this value. (i.e. `"2"` will match `0.2.x`). For future-proof, this value is a string
rather than an integer.

### Key: `megaton.custom-entry` (string)
The entry point (main function) symbol passed to the linker.

```admonish warning
Megaton library is required to use Rust
```

If non-empty, Megaton library will NOT be compiled or linked when running `megaton build`.
Use this option to use Megaton as a standalone build tool, without the Megaton library.
When using Megaton library, the `megaton_main` function is not the real "main function"; instead,
it is wrapped with a function that initializes Megaton and calls `megaton_main`.

Default: `""` (use Megaton library)
