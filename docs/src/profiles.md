# Profiles

The profile system allows you to have different configurations/targets
for your project. An example is having different build flags
and dependencies when targeting different versions of a game.

The profile system applies to the following top-level sections:
- `[build]`
- `[rust]`
- `[check]`
```admonish danger
Rust support is not yet available
```

Profiles for a top-level section like `[build]`
are specified in the `profiles` object,
which is a map of profile names to the configuration.
For example, `[build.profiles.foo]` has the same
schema as `[build]`, and should contain configuration for
the `foo` profile.

```toml
[build]
sources = ["src"]

[build.profiles.foo]
sources = ["src_foo"]
```

Profiles can be selected when building by using
`megaton build --profile PROFILE`.
For the example above, running `megaton build --profile foo`
will include both `src` and `src_foo` as source directories.
```admonish note
`none` is a reserved profile name for the base profile
and cannot be used as a profile name. Likewise, `megaton build --profile none`
will use the base profile, which is the default behavior
```

When both the base profile and a custom profile specify the same
property, like the example above with the `sources` property,
the values are merged in the following manner:
- If the property is an array, the values from the custom profile
  is appended to the base profile
- Otherwise, value from the custom profile overrides the base
```admonish info
Nested map properties like `build.flags` are recursively merged,
and should be specified like `[build.profiles.foo.flags]`, **NOT** `[build.flags.profiles.foo]`

```

## Configure Defaults
The `module.default-profile` optional property can be used to:
1. Specify which profile to use when `megaton build` is run without `-p`/`--profile`
2. Require a profile to be specified when running `megaton build`

When the property is not specified, `megaton build` will use the base profile (i.e. the `none` profile)

When the property is set to empty string, a profile must be specified when running `megaton build`.
This is useful when multiple profiles are present, but none of them makes sense to be the default.
```toml
[module]
name = "example"
default-profile = "" 
# Run with `megaton build -p PROFILE`
```

When there makes sense to be a default, you can set `default-profile` to that
so `megaton build` will select that profile when `-p` is not specified
```toml
[module]
name = "example"
default-profile = "foo" 
# Run with `megaton build` -> use foo
# Run with `megaton build -p none` -> use none (base profile)
```

If the base profile is not meant to be run, use the `disallow-base-profile`
property:
```toml
[module]
name = "example"
default-profile = "foo" 
disallow-base-profile = true
# Run with `megaton build` -> use foo
# Run with `megaton build -p none` -> error
```
