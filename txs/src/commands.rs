//! TxsApp Subcommands
//!
//! This is where you specify the subcommands of your application.
//!
//! The default application comes with two subcommands:
//!
//! - `start`: launches the application
//! - `version`: print application version
//!
//! See the `impl Configurable` below for how to specify the path to the
//! application's configuration file.

mod create_account_cmd;
mod create_validator_cmd;
mod oracle_upgrade_cmd;
mod version_cmd;
mod autopay_batch_cmd;

use abscissa_core::{Command, Configurable, Help, Options, Runnable};
use ol_cli::commands::CONFIG_FILE;
use crate::config::AppConfig;
use dirs;
use libra_global_constants::NODE_HOME;
use self::{
    create_account_cmd::CreateAccountCmd,
    create_validator_cmd::CreateValidatorCmd,
    oracle_upgrade_cmd::OracleUpgradeCmd,
    version_cmd::VersionCmd,
    autopay_batch_cmd::AutopayBatchCmd,    
};
use std::path::PathBuf;


/// TxsApp Subcommands
#[derive(Command, Debug, Options, Runnable)]
pub enum TxsCmd {

    /// The `create-account` subcommand
    #[options(help = "submit tx to create a user account from account.json file")]
    CreateAccount(CreateAccountCmd),

    /// The `create-validator` subcommand
    #[options(help = "submit tx to create a validator from account.json file")]
    CreateValidator(CreateValidatorCmd),

    /// The `oracle-upgrade` subcommand
    #[options(help = "submit an oracle transaction to upgrade stdlib")]
    OracleUpgrade(OracleUpgradeCmd),     

    /// The `autopay-new` subcommand
    #[options(help = "batch autopay transactions from json file")]
    AutopayBatch(AutopayBatchCmd),   

    // --- End of STDLIB SCRIPT COMMANDS ---

    /// The `help` subcommand
    #[options(help = "get usage information")]
    Help(Help<Self>),

    /// The `version` subcommand
    #[options(help = "display version information")]
    Version(VersionCmd),   

}

/// This trait allows you to define how application configuration is loaded.
impl Configurable<AppConfig> for TxsCmd {
    /// Location of the configuration file
    fn config_path(&self) -> Option<PathBuf> {
        // Check if the config file exists, and if it does not, ignore it.
        // If you'd like for a missing configuration file to be a hard error
        // instead, always return `Some(CONFIG_FILE)` here.

        let mut config_path = dirs::home_dir().unwrap();
        config_path.push(NODE_HOME);
        config_path.push(CONFIG_FILE);

        Some(config_path)
    }
}
