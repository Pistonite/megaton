# Megaton TOML Config Spec

This config allows the user to set several options which change how the mod is built by the build tool.

This root of a particular project is the directory that contains the config (Megaton.toml). For all values that determine a path, unless otherwise specified, the path is relative to the project root.

For each key, if a default value/behavior is not specified, the entry is required in the config.

The expected type for each key is listed in its heading. Additional restrictions may be specified in the description if a specific form is expected.

## Profile enabled keys/sections
Profile enabled keys are options which can have a unique value for different profiles. To set a value for a specific profile, use the key format `{section}.profiles.{profile-name}.{key-name}`. If a profile enabled key is set without specifying a profile, i.e. `{section}.{key-name}`, the value will be set for the base profile. If a section is marked as profile enabled, all keys under that section are profile enabled.

### Profile inheritance behavior
Each config options set for a profile uses one of two inheritance behaviors. The parent of a user-specified profile is the base profile. The "parent" of the base profile is the base profile's default value. If a profile enabled key is not an array type, it will always override its parent.

- Append: The value for this key is the appended to that of its parent. This means that a profile will always extend the default behavior but cannot disable it.
- Override: The value for this key will override that of its parent. These values can still optionally extend their parent by including "<default>" in their value. If the values is specified as [], the parent value will be completely disabled.

## Section: `module`
In a Megaton project, the module is the application that the build tool will target, i.e. the mod. Compiles to a .nso file.

### Key: `module.name` (string)
The name of the module, i.e., your mod's name. The final binary will use this name.

Restrictions: Cannot be "" or "lib". Only alphanumeric characters, -, and _ are allowed.

### Key: `module.title-id` (integer)
The title ID for the targeted game. Needed to generate the NPDM file. 

### Key: `module.target` (string)
The path to the working directory for the Megaton build tool. All the library and generated artifacts will be placed here.

Default: "target"

### Key: `module.compdb` (string)
A database of compiler commands to be used for clangd integration. Not used for Megaton's internal compile DB.

Default: "compile_commands.json"

## Section: `profile`
Megaton allows multiple different config profiles to coexist. This allows the user to set up profiles for different build tasks like release and debug. The profile used in a particular command can be set via CLI flag, otherwise the default profile will be used.

### Key: `profile.allow-base` (boolean)
Determines if the 'base' profile is allowed to be built.
If this is set to false, the -p flag is omitted, and `profile-default = "base"`, the build will fail

Default: true

### Key: `profile.default` (string)
The profile that will be built if the -p <PROFILE> flag is omitted from the CLI.
Restrictions: If set to "", the profile must be set via CLI flag on every call.

Default: "base"

### Key: `megaton.version` (string)
Version of Megaton the project is supposed to use. Megaton will abort if it's major and minor version do not match this value.

Restrictions: Must be in the form "{major}.{minor}".

### Key: `megaton.custom-entry` (string)
The entry point passed to the linker. If specified, the Megaton library will be disabled, including Rust support. This allows the user to use the Megaton build tool with another runtime library. If set to the empty string, Megaton will use the Megaton library as the entry point.

Default: "" (use Megaton library)

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

## Section: `check` (Profile Enabled)
Megaton performs a check step after linking the module into an ELF and before converting it to a .nso file. This check allows Megaton to blacklist certain symbols and instructions, preventing an unstable binary from being created.

### Key: `check.ignore` (array of strings)
A list of symbols that the checker will ignore when checking the built binary. 

Inheritance: Override

Default: ["<default>"]

### Key: `check.symbols` (array of strings)
Paths to symbol files generated by `objdump -T`.

Inheritance: Append

Default: []

### Key: `check.disallowed-instructions` (array of strings)
Instructions that are disallowed in the final binary. Place instructions that are known to crash here. (Mostly needed for Megaton tool development).

Inheritance: Override

Default: ["<default>"]

## Section: `build` (Profile Enabled)
This section allows the user to configure the build step for C/C++/Assembly. This can be used to modify compiler/linker flags.

### Key: `build.sources` (array of strings)
Paths to source directories to recursively scan for C/C++/Assembly source files. Sources generated by Megaton are automatically included and do not need to be specified. If the mod contains only Rust code, this can be omitted. Sources will be detected and matched based on file extension. The following extensions will be detected:

- C: `.c`
- C++: `.cc`, `.c++`, `.cpp`
- Assembly: `.s`, `.asm`

Inheritance: Append

Default: []

### Key: `build.includes` (array of strings)
Paths to include directories to be passed to the compiler as -I flags. Headers generated by Megaton are automatically included and do not need to be specified.

Inheritance: Append

Default: []

### Key: `build.libpaths` (array of strings)
Paths to library directories to be passed to the compiler as -L flags.

Inheritance: Append

Default: []

### Key: `build.libraries` (array of strings)
Libraries to be linked. These will be passed as -l flags to the linker. The names of libraries here must be discoverable in the directories specified in `build.libpaths`. For example, to link the library "foo", add "foo" to this array and place the file `libfoo.so` in one of the library paths.

Inheritance: Append

Default: []

### Key: `build.objects` (array of strings)
Additional .o and .a objects to link with, i.e. compiled objects that are not generated by Megaton.

Inheritance: Append

Default: []

### Key: `build.flags'
Build flags to pass to the different tools on the build toolchain. All `flags` keys have the same inheritance behavior. 
In order to add a build flag, specify the value like this: `[<"default">, -DDEBUG]`. If the value is specified as `[]`, The default flags will be disabled for that profile.
See the [Documentation](https://megaton.pistonite.dev/config/build_flags.html) to see what the default flags are.

Inheritance: Append

Default: ["<default>"]

#### Key: `build.flags.common`
Flags for all tools except rust and cargo.

#### Key: `build.flags.c`
Flags for the C compiler.

#### Key: `build.flags.cxx`
Flags for the C++ compiler.

#### Key: `build.flags.as`
Flags for the assembler to use with assembly sources.

#### Key: `build.flags.ld`
Flags for the linker.

#### Key: `build.flags.rust`
Flags for rust. Corresponds to the RUSTFLAGS environment variable. Rust flags can also be specified in Cargo.toml, but placing them here allows them to be dynamically enabled using Megaton's profile system.

Restrictions: Can only be specified if `cargo.enabled = true`

#### Key: `build.flags.cargo`
Flags to be passed to cargo, such as feature flags. 

Restrictions: Can only be specified if `cargo.enabled = true`

