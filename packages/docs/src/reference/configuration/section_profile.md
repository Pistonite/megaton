
Megaton allows multiple different config profiles to coexist.
This allows the user to set up profiles for different build tasks like release and debug.
The profile used in a particular command can be set via CLI flag,
otherwise the default profile will be used.

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

### Key: `megaton.version` (string)
Version of Megaton the project is supposed to use. Megaton will abort if it's major and minor version do not match this value.

Restrictions: Must be in the form "{major}.{minor}".

### Key: `megaton.custom-entry` (string)
The entry point passed to the linker. If specified, the Megaton library will be disabled, including Rust support. This allows the user to use the Megaton build tool with another runtime library. If set to the empty string, Megaton will use the Megaton library as the entry point.

Default: "" (use Megaton library)
