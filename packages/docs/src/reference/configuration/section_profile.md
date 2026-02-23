
Megaton allows multiple different config profiles to coexist.
This allows the user to set up profiles for different build configurations like release and debug,
or different versions of a game. See the tutorial for [Profile](../../tutorial/profiles.md)
for more info.

### Profile inheritance behavior
Each config options set for a profile inherit from the parent of the profile:
- The parent of a user-specified profile is the base profile.
- The "parent" of the base profile is the base profile's default value.

The inheritance uses one of two inheritance behaviors.
- Append: The value for this key is the appended to that of its parent. This means that a profile will always extend the default behavior but cannot disable it.
- Override: The value for this key will override that of its parent. These values can still optionally extend their parent by including "<default>" in their value. If the values is specified as [], the parent value will be completely disabled.

If a profile enabled key is not an array type, it will always override its parent.

### Key: `profile.allow-base` (boolean)
Determines if the 'base' profile is allowed to be built.
If this is set to false, the -p flag is omitted, and `profile-default = "base"`, the build will fail

Default: true

### Key: `profile.default` (string)
The profile that will be built if the -p <PROFILE> flag is omitted from the CLI.
Restrictions: If set to "", the profile must be set via CLI flag on every call.

Default: "base"

The `profile.default` optional property can be used to:
1. Specify which profile to use when `megaton build` is run without `-p`/`--profile`
2. Require a profile to be specified when running `megaton build`

When the property is not specified, `megaton build` will use the base profile (i.e. the `none` profile)

When the property is set to empty string, a profile must be specified when running `megaton build`.
This is useful when multiple profiles are present, but none of them makes sense to be the default.
```toml
[module]
name = "example"

[profile]
default = "" 
# Run with `megaton build -p PROFILE`
```

When there makes sense to be a default, you can set `default` to that
so `megaton build` will select that profile when `-p` is not specified
```toml
[module]
name = "example"
default = "foo" 
# Run with `megaton build` -> use foo
# Run with `megaton build -p none` -> use none (base profile)
```

```admonish tip
If a profile other than `none` should be the default for `clangd`,
also change the `CompileDatabase` property in `.clangd` to the desired 
output location
```

If the base profile is not meant to be run, set the `allow-base`
property to `false`
```toml
[module]
name = "example"
default = "foo" 
allow-base = false
# Run with `megaton build` -> use foo
# Run with `megaton build -p none` -> error
```
