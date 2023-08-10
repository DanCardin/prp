use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

use clap::{ArgAction, CommandFactory, Parser, Subcommand};
use clap_complete::generate;

mod x;

use crate::cli::x::ExecutableCommand;
use crate::settings::Settings;
use crate::shell::Shell;
use crate::venv::Venv;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(
        short,
        long,
        action = clap::ArgAction::Set,
        value_parser = clap_complete::Shell::from_str,
    )]
    shell: Option<clap_complete::Shell>,

    #[arg(short, long)]
    name: Option<String>,

    #[arg(short, long)]
    python: Option<PathBuf>,
}

impl Args {
    pub fn parse() -> Self {
        Self::try_parse().unwrap_or_else(|e| {
            let stderr = std::io::stderr();
            let mut handle = stderr.lock();

            let message = format!("{}", e);
            handle.write_all(message.as_ref()).unwrap();
            handle.flush().unwrap();
            std::process::exit(1)
        })
    }
}

#[derive(Subcommand, Debug)]
enum Commands {
    // New
    Activate,
    // Build(RunCommand),
    Exec(ExecCommand),
    Info,
    Prompt,
    Run(RunCommand),
    Shell(ShellCommand),
    Venv(VenvCommand),

    #[command(visible_alias = "x")]
    Executable(ExecutableCommand),
    //
    // Native to pip
    // Install(RunCommand),
    // Download(RunCommand),
    // Uninstall(RunCommand),
}

#[derive(Parser, Debug)]
pub struct VenvCommand {
    #[arg(long = "no-activate", action = ArgAction::SetFalse)]
    activate: bool,

    #[arg(short, long)]
    delete: bool,

    #[arg(long)]
    fix: bool,
}

impl Default for VenvCommand {
    fn default() -> Self {
        Self {
            activate: true,
            delete: false,
            fix: false,
        }
    }
}

#[derive(Parser, Debug)]
pub struct RunCommand {
    command: Option<String>,
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, value_delimiter = None)]
    args: Vec<String>,
}

#[derive(Parser, Debug)]
pub struct ExecCommand {
    command: String,
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, value_delimiter = None)]
    args: Vec<String>,
}

#[derive(Parser, Debug)]
pub struct ShellCommand {
    #[command(subcommand)]
    command: Option<ShellCommands>,
}

#[derive(Subcommand, Debug)]
enum ShellCommands {
    Init,
    Completions,
}

pub fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut cli_command = Args::command();
    let cli_name = cli_command.get_name().to_string();

    let mut settings = Settings::read(&cli_name)?;
    settings.set_python_path(args.python);
    settings.set_venv_name(args.name);

    let mut venv = Venv::from_settings(settings)?;

    let command = args
        .command
        .unwrap_or(Commands::Venv(VenvCommand::default()));

    let clap_shell = get_shell(args.shell);
    let shell_name = format!("{}", clap_shell);
    let shell = Shell::new(&shell_name)?;

    match command {
        Commands::Info => venv.print_info(),
        Commands::Activate => shell.activate(&venv),
        Commands::Run(cmd) => shell.run(&venv, cmd.command.as_deref(), &cmd.args)?,
        Commands::Exec(cmd) => shell.exec(&venv, &cmd.command, &cmd.args)?,
        Commands::Prompt => shell.prompt(&venv)?,
        Commands::Shell(subcmd) => match subcmd.command {
            None => shell.enter(&venv)?,
            Some(ShellCommands::Init) => shell.init(),
            Some(ShellCommands::Completions) => {
                generate(
                    clap_shell,
                    &mut cli_command,
                    cli_name,
                    &mut std::io::stdout(),
                );
            }
        },
        Commands::Venv(cmd) => {
            if cmd.delete {
                venv.delete()?;
            } else {
                venv.create(cmd.fix)?;
                if cmd.activate && venv.settings.auto_activate {
                    shell.activate(&venv);
                }
            }
        } // Commands::Install => shell.run(&venv, "pip"),
        Commands::Executable(cmd) => cmd.run()?,
    }

    Ok(())
}

fn get_shell(shell: Option<clap_complete::Shell>) -> clap_complete::Shell {
    shell.unwrap_or(clap_complete::Shell::from_env().unwrap_or(clap_complete::Shell::Bash))
}
