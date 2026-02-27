
Megaton can be used to build mods that contain both C++ and Rust code. Internally Megaton uses Cargo to compile Rust sources. It also provides support for the [cxx](https://cxx.rs/) crate to handle interop between C++ and Rust (if needed).  Rust crates should be configured, as usual, in the `Cargo.toml` file. This sections controls Megaton options for Cargo and CXX.

```admonish tip
The root of a particular project is the directory that contains the config (Megaton.toml).
For all values that determine a path, unless otherwise specified,
the path is relative to the project root.

For each key, if a default value/behavior is not specified, it is required in the config.
Otherwise it is optional.
```

### Key: `cargo.enabled`
Type: `bool`

Determines if Rust support is enabled. Megaton will only try to do Rust stuff like `cargo build` if this is true.

Restrictions: May not be true if `megaton.custom-entry` is not `""`.
See [`[megaton]`](./section_megaton.md) config section.

Default: Megaton will try to determine if Rust support should be enabled using the following checks, in order:
- If `cargo.manifest` is specified -> `true`
  - It will error if the specified path cannot be found.
- If the file `Cargo.toml` exists in project root -> `true`
- Else -> `false`

### Key: `cargo.manifest`
Type: `string`

The manifest file path for cargo. This will be passed to `--manifest-path` when cargo is run.

Default: `"Cargo.toml"` if it exists, `cargo.enabled` will be `false` if it does not exist.

### Key: `cargo.sources`
Type: `string[]` (array of strings)

The Rust source directories to scan for `.rs` files. These files will be scanned for `cxx::bridge` attributes to generate FFI code.

Default: `["src"]`

### Key: `cargo.header-suffix`
Type: `string`

Suffix to generated headers. The user can put the empty string here so they can do `#include <foo/lib.rs>` instead of `<foo/lib.rs.h>`

Default: `".h"`

