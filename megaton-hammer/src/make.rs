//! Integration with `make` build tool.
//!
//! Megaton puts the artifacts in the `./target/megaton/<flavor>/<profile>/make` directory:
//! - `build.mk`: The Makefile
//! - `build`: The build output directory

use crate::{MegatonConfig, MegatonHammer};
use crate::error::Error;

macro_rules! format_makefile_template {
    ($($args:tt)*) => {
        format!(
r###"
# GENERATED BY MEGATON HAMMER
include $(DEVKITPRO)/libnx/switch_rules

MEGATON_MODULE_NAME := {MEGATON_MODULE_NAME}
MEGATON_MODULE_ENTRY := {MEGATON_MODULE_ENTRY}
MEGATON_MODULE_TITLE_ID := 0x{MEGATON_MODULE_TITLE_ID}
MEGATON_ROOT := ../../../../

TARGET := $(MEGATON_MODULE_NAME)

DEFAULT_ARCH_FLAGS := \
    -march=armv8-a+crc+crypto \
    -mtune=cortex-a57 \
    -mtp=soft \
    -fPIC \
    -fvisibility=hidden" \

DEFAULT_CFLAGS := \
    -g \
    -Wall \
    -Werror \
    -ffunction-sections \
    -fdata-sections \
    -O3 \

DEFAULT_CXXFLAGS := \
    -fno-rtti \
    -fomit-frame-pointer \
    -fno-exceptions \
    -fno-asynchronous-unwind-tables \
    -fno-unwind-tables \
    -enable-libstdcxx-allocator=new \
    -fpermissive \
    -std=c++20 \

DEFAULT_ASFLAGS := -g
DEFAULT_LDFLAGS := \
    -g \
    -Wl,-Map,$(TARGET).map \
    -nodefaultlibs \
    -nostartfiles \
    -Wl,--shared \
    -Wl,--export-dynamic \
    -Wl,-z,nodynamic-undefined-weak \
    -Wl,--gc-sections \
    -Wl,--build-id=sha1 \
    -Wl,--nx-module-name \
    -Wl,-init=$(MEGATON_MODULE_ENTRY) \
    -Wl,--exclude-libs=ALL \

DEFAULT_LIBS := -lgcc -lstdc++ -u malloc

{EXTRA_SECTION}

SOURCES          := $(SOURCES) {SOURCES}
ALL_SOURCE_DIRS  := $(ALL_SOURCE_DIRS) $(foreach dir,$(SOURCES),$(shell find $(dir) -type d))
VPATH            := $(VPATH) $(foreach dir,$(ALL_SOURCES_DIRS),$(CURDIR)/$(dir))

INCLUDES         := $(INCLUDES) {INCLUDES}
LIBDIRS          := $(LIBDIRS) $(PORTLIBS) $(LIBNX)
INCLUDE_FLAGS    := $(foreach dir,$(INCLUDES),-I$(CURDIR)/$(dir)) $(foreach dir,$(LIBDIRS),-I$(dir)/include)

DEFINES          := $(DEFINES) {DEFINES}

ARCH_FLAGS       := $(ARCH_FLAGS) {ARCH_FLAGS}
CFLAGS           := $(CFLAGS) $(ARCH_FLAGS) $(DEFINES) $(INCLUDE) {CFLAGS}
CXXFLAGS         := $(CFLAGS) $(CXXFLAGS) {CXXFLAGS}
ASFLAGS          := $(ASFLAGS) $(ARCH_FLAGS) {ASFLAGS}

LD_SCRIPTS       := {LD_SCRIPTS}
LD_SCRIPTS_FLAGS := $(foreach ld,$(LD_SCRIPTS),-Wl,-T,$(CURDIR)/$(ld))
LD               := $(CXX)
LDFLAGS          := $(LDFLAGS) $(ARCH_FLAGS) $(LD_SCRIPTS_FLAGS) {LDFLAGS}
LIBS             := $(LIBS) {LIBS}
LIBPATHS         := $(LIBPATHS) $(foreach dir,$(LIBDIRS),-L$(dir)/lib) 

DEPSDIR          ?= .
CFILES           := $(foreach dir,$(ALL_SOURCES_DIRS),$(notdir $(wildcard $(dir)/*.c)))
CPPFILES         := $(foreach dir,$(ALL_SOURCES_DIRS),$(notdir $(wildcard $(dir)/*.cpp)))
SFILES           := $(foreach dir,$(ALL_SOURCES_DIRS),$(notdir $(wildcard $(dir)/*.s)))
OFILES           := $(CPPFILES:.cpp=.o) $(CFILES:.c=.o) $(SFILES:.s=.o)
DFILES           := $(OFILES:.o=.d)

$(TARGET).nso: $(TARGET).elf
$(TARGET).elf: $(OFILES) $(LD_SCRIPTS)

-include $(DFILES)

"###,
        $($args)*
        )
    };
}

macro_rules! default_or_empty {
    ($make:ident, $default:expr) => {
        if $make.no_default_flags.unwrap_or_default() {
            ""
        } else {
            $default
        }
    };
}

impl MegatonConfig {
    /// Create the Makefile content from the config
    pub fn create_makefile(&self, cli: &MegatonHammer) -> Result<String, Error> {
        let make = self.make.get_profile(&cli.options.profile);

        let entry = make.entry.as_ref().ok_or(Error::NoEntryPoint)?;

        let extra_section = make.extra.iter().map(|s| format!("{} := {}", s.key, s.val)).collect::<Vec<_>>().join("\n");

        let sources = make.sources.iter().map(|s| format!("$(MEGATON_ROOT){s}")).collect::<Vec<_>>().join(" ");
        let includes = make.includes.iter().map(|s| format!("$(MEGATON_ROOT){s}")).collect::<Vec<_>>().join(" ");
        let ld_scripts = make.ld_scripts.iter().map(|s| format!("$(MEGATON_ROOT){s}")).collect::<Vec<_>>().join(" ");
        let defines = make.defines.iter().map(|s| format!("-D{s}")).collect::<Vec<_>>().join(" ");

        let makefile = format_makefile_template!(
            MEGATON_MODULE_NAME = self.module.name,
            MEGATON_MODULE_ENTRY = entry,
            MEGATON_MODULE_TITLE_ID = self.module.title_id_hex(),
            EXTRA_SECTION = extra_section,
            SOURCES = sources,
            INCLUDES = includes,
            DEFINES = defines,
            ARCH_FLAGS = default_or_empty!(make, "$(DEFAULT_ARCH_FLAGS)"),
            CFLAGS = default_or_empty!(make, "$(DEFAULT_CFLAGS)"),
            CXXFLAGS = default_or_empty!(make, "$(DEFAULT_CXXFLAGS)"),
            ASFLAGS = default_or_empty!(make, "$(DEFAULT_ASFLAGS)"),
            LD_SCRIPTS = ld_scripts,
            LDFLAGS = default_or_empty!(make, "$(DEFAULT_LDFLAGS)"),
            LIBS = default_or_empty!(make, "$(DEFAULT_LIBS)"),
        );

        Ok(makefile)
    }
}
