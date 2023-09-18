mod cli;
mod posy;
mod python;
mod settings;
mod shell;
mod venv;

mod package_specifier;

fn main() -> anyhow::Result<()> {
    crate::cli::main()
}
