use buildcommon::print;
use clap::{Args, Parser, Subcommand, ValueEnum};

static LOGO: &str = r#" __    __ ______ ______ ______ ______ ______ __   __  
/\ "-./  \\  ___\\  ___\\  __ \\__  _\\  __ \\ "-.\ \ 
\ \ \-./\ \\  __\ \ \__ \\  __ \_/\ \/ \ \/\ \\ \-.  \
 \ \_\ \ \_\\_____\\_____\\_\ \_\\ \_\\ \_____\\_\\"\_\
  \/_/  \/_//_____//_____//_/\/_/ \/_/ \/_____//_/ \/_/"#;

/// Megaton build tool CLI
#[derive(Debug, Clone, PartialEq, Parser)]
#[clap(bin_name="megaton", before_help=LOGO)]
pub struct Cli {
    /// Top level options
    #[clap(flatten)]
    pub top: TopLevelOptions,

    /// Subcommand
    #[clap(subcommand)]
    pub command: Command,

    /// Common options
    #[clap(flatten)]
    pub options: CommonOptions,
}

/// Top level options
#[derive(Debug, Clone, PartialEq, Args)]
pub struct TopLevelOptions {
    /// Change the directory to run in
    #[clap(short('C'), long, default_value = ".")]
    pub dir: String,

    /// Set path to megaton repo
    ///
    /// Used by the shim script for passing in the path directly
    /// so the tool doesn't need to query it
    #[clap(short('H'), long, hide(true))]
    pub home: Option<String>,
}

impl Cli {
    pub fn apply_print_options(&self) {
        if self.is_verbose_on() {
            print::verbose_on();
        }

        match (&self.command.color, &self.options.color) {
            (Some(ColorOption::Never), _) => {
                print::color_off();
            }
            (Some(ColorOption::Always), _) => {
                // color is already on by default
            }
            (None, Some(ColorOption::Never)) => {
                print::color_off();
            }
            (None, Some(ColorOption::Always)) => {
                // color is already on by default
            }
            _ => print::auto_color(),
        }
    }

    #[inline]
    pub fn is_verbose_on(&self) -> bool {
        self.options.verbose || self.command.verbose
    }

    #[inline]
    pub fn is_trace_on(&self) -> bool {
        self.options.trace || self.command.trace
    }
}

#[derive(Debug, Clone, PartialEq, Subcommand)]
pub enum Command {
    /// Create a new project
    Init(CommonOptions),
    /// Build the current project
    Build(crate::cmd_build::Options),
    /// Clean outputs
    Clean(crate::cmd_clean::Options),
    /// Check the environment and installation status of
    /// megaton, dependent tools and toolchain/libraries
    ///
    /// The paths found will be cached for faster lookup in the future
    Checkenv(CommonOptions),
    /// Install or update the build tool
    Install(crate::cmd_install::Options),
    /// Format the code
    Fmt(CommonOptions),
    /// Rustc options
    Rustc(CommonOptions),
}

impl Command {
    pub fn show_fatal_error_message(&self) -> bool {
        match self {
            Self::Build(options) => {
                // don't show too many error messages for regular build
                options.compdb || options.lib
            }
            _ => true
        }
    }
}

impl std::ops::Deref for Command {
    type Target = CommonOptions;

    fn deref(&self) -> &Self::Target {
        match self {
            Command::Init(x) => x,
            Command::Build(x) => x,
            Command::Clean(x) => x,
            Command::Checkenv(x) => x,
            Command::Install(x) => x,
            Command::Fmt(x) => x,
            Command::Rustc(x) => x,
        }
    }
}

/// Common options for all commands
#[derive(Debug, Clone, PartialEq, Args)]
pub struct CommonOptions {
    /// Enable verbose output
    #[clap(short = 'V', long)]
    pub verbose: bool,

    /// Enable error trace
    #[clap(short = 'T', long)]
    pub trace: bool,

    /// Set output color option
    ///
    /// By default, color is enabled when stderr is terminal
    #[clap(long)]
    pub color: Option<ColorOption>,
}

/// Color options for output
#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum ColorOption {
    Always,
    Never,
}
