/// TODO:
///  * warn if binary path is not on PATH
///  * warn if no apps are exposed by installation
///  *  - delete venv
///  * optional cli command target(s)
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct ExecutableCommand {
    #[command(subcommand)]
    command: ExecutableCommands,
}

impl ExecutableCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        match &self.command {
            ExecutableCommands::Install(_cmd) => {}
            ExecutableCommands::Update(_cmd) => {}
            ExecutableCommands::Uninstall(_cmd) => {}
            ExecutableCommands::Run(_cmd) => {}
            ExecutableCommands::Exec(_cmd) => {}
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
}

#[derive(Parser, Debug)]
pub struct ExecutableInstall {
    package: String,

    #[arg(short, long)]
    extras: Vec<String>,

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
