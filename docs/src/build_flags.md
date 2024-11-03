# Build Flags

You can customize build and linker flags applied to your C, C++, and assembly
source files in `Megaton.toml`, using the `build.flags` object.

There are 5 properties that can be set:
- `common`: Common flags for all processes
- `c` : Flags for compiling C sources `.c`
- `cxx` : Flags for compiling C++ sources `.cpp`, `.cc`, `.cxx`, `.c++`
- `as`: Flags for compiling assembly sources `.s`, `.asm`
- `ld`: Flags for linking

Each flag property is an array of strings:

```toml
[build.flags]
common = ["<default>"] 
c = ["<default>", "-DDEBUG"] 
cxx = ["<default>"]
as = ["<default>"]
ld = ["<default>"]
```
```admonish warning
C++ compiler is used for linking. Flags for `ld`
should be specified as `-Wl,--flag` instead of `--flag`.
```

The string `<default>` is a special token used to include the
default flags that are maintained by the megaton build tool
plus extending from another set of flags (for example, the 
default flags of `cxx` include all `c` flags).

The default flags can be found [here](), and the flag extension
behavior is detailed below.

| Property | `<default>` includes  |
|----------|-----------------------|
| `common` | None                  |
| `c`      | `common`              |
| `cxx`    | `c`                   |
| `as`     | `cpp`                 |
| `ld`     | `common`              |

Each property is also optional. If it's not set, it's equivalent
to `["<default>"]`. Setting a property to an empty array `[]` means
not add any flags.

## Includes and Libraries
The build tool will complain if the build flags contain 
any include paths (`-I`), library paths (`-L`), or library names (`-l`).
These flags should be set in the `[module]` top-level section, so the build tool
can monitor their timestamps for incremental builds. They are
converted to the appropriate flags for the compiler and linker by the build tool.

The build tool will also automatically include the headers for
`libmegaton` and link with your project's rust code if the `[rust]` top-level section
is configured.
```admonish danger
Rust support is not yet available
```

## Profiles
The build flags also support the [profile]() system.

The profile-specific flags for a profile `foo` is defined at `build.profiles.foo.flags`.
The flags are merged with flags from the base `build.flags` in the following way:
- If a property is not specified, it will be the same as the ones in `build.flags`
- If a property is specified, it will be appended to the ones in `build.flags`

For example:
```toml
[build.flags]
c = ["<default>", "-DDEBUG"]

[build.profiles.foo.flags]
c = ["-DFOO=1"]
```

The C flags for the base profile will be the default flags plus `-DDEBUG`,
while the C flags for the `foo` profile will be the default flags plus `-DDEBUG -DFOO=1`.

There isn't a way to remove flags from the base profile.
Typically, when you find yourself needing to remove a flag,
you can restructure the flags and [configure profile defaults](./profiles.md#configure-defaults)
to get the desired behavior for selecting profile with command line.

For example, if you want to build with `-DDEBUG` by default,
and change it to `-DNODEBUG` for the `release` profile, you can do:
```toml
[module]
default-profile = "debug"
disallow-base-profile = true

[build.flags]

[build.profiles.debug.flags]
c = ["<default>", "-DDEBUG"]

[build.profiles.release.flags]
c = ["<default>", "-DNODEBUG"]
```
With this configuration `megaton build` will build with `-DDEBUG`
and `megaton build -p release` will build with `-DNODEBUG`.
