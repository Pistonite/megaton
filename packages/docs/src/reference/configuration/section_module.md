
In a Megaton project, the module is the application that the build tool will target, i.e. the mod. Compiles to a .nso file.

Example:
```toml
[module]
name = "my-mod"
title-id = 0x01007ef00011e000
```

```admonish tip
This root of a particular project is the directory that contains the config (Megaton.toml).
For all values that determine a path, unless otherwise specified,
the path is relative to the project root.

For each key, if a default value/behavior is not specified, the entry is required in the config.
```

### Key: `module.name` (`string`)
The name of the module, i.e., your mod's name. The final binary will use this name.

Restrictions: Cannot be `""` or `"lib"`. Only alphanumeric characters, `-`, and `_` are allowed.

### Key: `module.title-id` (`integer`)
The title ID for the targeted game. Needed to generate the NPDM file. 
(Note this needs to be an integer in the config, not a hex string)

### Key: `module.target` (string)
The path to the working directory for the Megaton build tool.
All the library and generated artifacts will be placed here under a subdirectory `megaton`

Default: `"target"`

### Key: `module.compdb` (string)
A database of compiler commands to be used for clangd integration.

Default: `"compile_commands.json"`
