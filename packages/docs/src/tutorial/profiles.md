# Profiles

The profile system allows you to have different configurations/targets
for your project. An example is having different build flags
and dependencies when targeting different versions of a game.

The [`[build]`](../reference/configuration/section_build.md)
and [`[check]`](../reference/configuration/section_check.md)
sections supports profiles using the `profiles` key in the section.
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
For the example above, running `megaton build -p foo`
will include both `src` and `src_foo` as source directories,
while running `megaton build` will only include `src` as the source directory.
The configs without any explicit profiles is known as the "base profile", and
has the name `"none"`. (This word is reserved you cannot name your custom profile `"none"`).

Nested map properties like `build.flags` are recursively merged,
and should be specified like `[build.profiles.foo.flags]`, **NOT** `[build.flags.profiles.foo]`

## Inheriting the Base Profile
Each config option specified on a custom profile inherits from the base profile.
The inheritance uses one of two inheritance behaviors.
- Append: The value for this key is the appended to that of its parent. This means that a profile will always extend the default behavior but cannot disable it.
  - This is always the case for scalar (non-array) values
- Override: The value for this key will override that of its parent.
  - If the value is array type, it can still optionally extend the base by including `"<default>"`
    in the array.

See the reference for [`Build`](../reference/configuration/section_build.md)
and [`Check`](../reference/configuration/section_check.md) sections for the behavior of each config option.

## Configure Defaults
You can customize the profile selection behavior of the CLI with the
[`[profile]`](../reference/configuration/section_profile.md) section of the config,
for example, select a profile by default if nothing is specified on the CLI, or disallow
the `"none"` profile (i.e. the base profile) from the CLI (useful if only using the base profile)
for inheritance.

See the reference for [`Profile`]((../reference/configuration/section_profile.md)) section for
more information.

