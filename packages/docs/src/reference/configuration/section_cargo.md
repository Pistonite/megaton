
## Section: `cargo`
Megaton can be used to build mods that contain both C++ and Rust code. Internally Megaton uses Cargo to compile Rust sources. It also uses [CXX](https://cxx.rs/) to handle interop between C++ and Rust.  Rust crates should be configured, as usual, in the Cargo.toml file. This sections controls Megaton options for Cargo and CXX.

### Key: `cargo.enabled` (boolean)
Determines if Rust support is enabled. Megaton will only try to do Rust stuff like `cargo build` if this is true.

Restrictions: May not be true if `megaton.custom-entry` is specified and not "".

Default: Megaton will try to determine if Rust support should be enabled using the following checks, in order:
- If `megaton.custom-entry` is specified and not "" -> false
- If `cargo.manifest` is specified -> true
- If the file `Cargo.toml` exists in project root -> true
- Else -> false

### Key: `cargo.manifest` (string)
The manifest file path for cargo. This will be passed to --manifest-path when cargo is run.

Default: "Cargo.toml"

### Key: `cargo.sources` (array of strings)
The Rust source directories to scan for .rs files. These files will be scanned for `cxx::bridge` attributes to generate FFI code.

Default: ["src"]

### Key: `cargo.header-suffix` (string)
Suffix to generated headers. The user can put the empty string here so they can do `#include <foo/lib.rs>` instead of `<foo/lib.rs.h>`

Default: ".h"

