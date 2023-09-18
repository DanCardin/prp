use clap::{Parser, Subcommand};

use crate::package_specifier::PackageSpecifier;
use crate::settings::Settings;
use crate::venv::Venv;

#[derive(Parser, Debug)]
pub struct ExecutableCommand {
    #[command(subcommand)]
    command: ExecutableCommands,
}

impl ExecutableCommand {
    pub fn run(&self, settings: &Settings) -> anyhow::Result<()> {
        match &self.command {
            ExecutableCommands::Install(cmd) => {
                let spec = PackageSpecifier::parse(cmd.package.as_ref())?;
                let mut venv = Venv::from_package_name(settings, &spec.name());
                venv.create(cmd.force)?;
                venv.install(spec)?
                // TODO:
                //  * warn if binary path is not on PATH
                //  * warn if no apps are exposed by installation
                //  *  - delete venv
                //  * optional cli command target(s)
            }
            ExecutableCommands::Update(_cmd) => {}
            ExecutableCommands::Uninstall(_cmd) => {}
            ExecutableCommands::Run(_cmd) => {}
            ExecutableCommands::Exec(_cmd) => {}
            ExecutableCommands::Info(cmd) => {
                let spec = PackageSpecifier::parse(cmd.package.as_ref())?;
                let venv = Venv::from_package_name(settings, &spec.name());
                venv.print_info();
            }
        }
        Ok(())
    }
}

#[derive(Subcommand, Debug)]
pub enum ExecutableCommands {
    Install(ExecutableInstall),
    Update(ExecutableUpdate),
    Uninstall(ExecutableUninstall),
    Run(ExecutableRun),
    Exec(ExecutableExec),
    Info(ExecutableInfo),
}

#[derive(Parser, Debug)]
pub struct ExecutableInstall {
    package: String,

    #[arg(short, long)]
    force: bool,
}

#[derive(Parser, Debug)]
pub struct ExecutableUpdate {}

#[derive(Parser, Debug)]
pub struct ExecutableUninstall {}

#[derive(Parser, Debug)]
pub struct ExecutableRun {
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, value_delimiter = None)]
    args: Vec<String>,
}

#[derive(Parser, Debug)]
pub struct ExecutableExec {
    command: Option<String>,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true, value_delimiter = None)]
    args: Vec<String>,
}

#[derive(Parser, Debug)]
pub struct ExecutableInfo {
    package: String,
}
