mod cli;
mod python;
mod settings;
mod shell;
mod venv;

fn main() -> anyhow::Result<()> {
    crate::cli::main()
}
