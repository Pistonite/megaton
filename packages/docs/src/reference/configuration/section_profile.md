
Megaton allows multiple different config profiles to coexist.
You can set up profiles for different build configurations like release and debug,
or different versions of a game. See the tutorial for [Profile](../../tutorial/profiles.md)
for more info.


```admonish tip
For each key, if a default value/behavior is not specified, it is required in the config.
Otherwise it is optional.
```

### Key: `profile.allow-base`
Type: `bool`

Determines if the [base profile](../../tutorial/profiles.md#base-profile) profile is allowed
to be built.

Default: `true`

### Key: `profile.default`
Type: `string`

The profile that will be built if the `-p PROFILE` flag is omitted from the CLI.
If set to `""` (the empty string), the profile must be set via CLI flag on every call.

Default: `"none"`

Examples:
1. Set to empty string
    ```toml
    [module]
    name = "example"

    [profile]
    default = "" # Must run with `megaton build -p PROFILE`
    ```
2. Set to non-empty string
    ```toml
    [module]
    name = "example"

    [profile]
    default = "foo" 
    # Run with `megaton build` -> use foo
    # Run with `megaton build -p none` -> use none (base profile)
    ```
3. Set with `allow-base = false`
    ```toml
    [module]
    name = "example"
    default = "foo" 
    allow-base = false
    # Run with `megaton build` -> use foo
    # Run with `megaton build -p none` -> error
    ```