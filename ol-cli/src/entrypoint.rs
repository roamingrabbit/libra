//! Toplevel entrypoint command.

use abscissa_core::{
    Command, command::Usage, Config, Configurable, FrameworkError, 
    Options, Runnable    
};
use libra_types::account_address::AccountAddress;
use std::path::PathBuf;

/// Toplevel entrypoint command.
///
/// Handles obtaining toplevel help as well as verbosity settings.
#[derive(Debug, Options)]
pub struct EntryPoint<Cmd>
where
    Cmd: Command + Runnable,
{
    /// Path to the configuration file
    #[options(short = "c", help = "path to configuration file")]
    pub config: Option<PathBuf>,

    /// Obtain help about the current command
    #[options(short = "h", help = "print help message")]
    pub help: bool,

    /// Increase verbosity setting
    #[options(short = "v", help = "be verbose")]
    pub verbose: bool,

    /// Subcommand to execute.
    ///
    /// The `command` option will delegate option parsing to the command type,
    /// starting at the first free argument.
    #[options(command)]
    pub command: Option<Cmd>,

    /// --- Customizing EntryPoint --- ///

    /// Account Address
    #[options(short = "a", help = "account address")]
    pub account: Option<AccountAddress>,
}

impl<Cmd> EntryPoint<Cmd>
where
    Cmd: Command + Runnable,
{
    /// Borrow the underlying command type or print usage info and exit
    fn command(&self) -> &Cmd {
        self.command
            .as_ref()
            .unwrap_or_else(|| Cmd::print_usage_and_exit(&[]))
    }
}

impl<Cmd> Runnable for EntryPoint<Cmd>
where
    Cmd: Command + Runnable,
{
    fn run(&self) {
        self.command().run()
    }
}

impl<Cmd> Command for EntryPoint<Cmd>
where
    Cmd: Command + Runnable,
{
    /// Name of this program as a string
    fn name() -> &'static str {
        Cmd::name()
    }

    /// Description of this program
    fn description() -> &'static str {
        Cmd::description()
    }

    /// Version of this program
    fn version() -> &'static str {
        Cmd::version()
    }

    /// Authors of this program
    fn authors() -> &'static str {
        Cmd::authors()
    }

    /// Get usage information for a particular subcommand (if available)
    fn subcommand_usage(command: &str) -> Option<Usage> {
        Cmd::subcommand_usage(command)
    }
}

impl<Cfg, Cmd> Configurable<Cfg> for EntryPoint<Cmd>
where
    Cmd: Command + Configurable<Cfg> + Runnable,
    Cfg: Config,
{
    /// Path to the command's configuration file
    fn config_path(&self) -> Option<PathBuf> {
        match &self.config {
            // Use explicit `-c`/`--config` argument if passed
            Some(cfg) => Some(cfg.clone()),

            // Otherwise defer to the toplevel command's config path logic
            None => self.command.as_ref().and_then(|cmd| cmd.config_path()),
        }
    }

    /// Process the configuration after it has been loaded, potentially
    /// modifying it or returning an error if options are incompatible
    fn process_config(&self, config: Cfg) -> Result<Cfg, FrameworkError> {
        match &self.command {
            Some(cmd) => cmd.process_config(config),
            None => Ok(config),
        }
    }
}