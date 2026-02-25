# Megaton TOML Config Spec

This config allows the user to set several options which change how the mod is built by the build tool.

This root of a particular project is the directory that contains the config (Megaton.toml).
For all values that determine a path, unless otherwise specified,
the path is relative to the project root.

For each key, if a default value/behavior is not specified, the entry is required in the config.

The expected type for each key is listed in its heading. Additional restrictions may be specified in the description if a specific form is expected.

## Profile enabled keys/sections
Profile enabled keys are options which can have a unique value for different profiles. To set a value for a specific profile, use the key format `{section}.profiles.{profile-name}.{key-name}`. If a profile enabled key is set without specifying a profile, i.e. `{section}.{key-name}`, the value will be set for the base profile. If a section is marked as profile enabled, all keys under that section are profile enabled.

### Profile inheritance behavior
Each config options set for a profile uses one of two inheritance behaviors. The parent of a user-specified profile is the base profile. The "parent" of the base profile is the base profile's default value. If a profile enabled key is not an array type, it will always override its parent.

- Append: The value for this key is the appended to that of its parent. This means that a profile will always extend the default behavior but cannot disable it.
- Override: The value for this key will override that of its parent. These values can still optionally extend their parent by including "<default>" in their value. If the values is specified as [], the parent value will be completely disabled.

