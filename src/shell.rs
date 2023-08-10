use anyhow::Context;
use std::ffi::OsString;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::venv::Venv;

pub struct Shell {
    exe_name: String,
    exe_path: PathBuf,
    pub kind: String,
}

impl Shell {
    pub fn new(kind: &str) -> anyhow::Result<Shell> {
        let exe_path = std::env::current_exe()?;
        let exe_name = exe_path
            .file_name()
            .context(format!("{exe_path:?} is has no file name"))?;

        Ok(Self {
            exe_path: exe_path.clone(),
            exe_name: exe_name.to_string_lossy().to_string(),
            kind: kind.to_string(),
        })
    }

    pub fn init(&self) {
        indoc::printdoc!(
            r#"
            function {exe_name} {{
              eval "$(command {exe_path} "$@")"
            }}"#,
            exe_name = self.exe_name,
            exe_path = self.exe_path.to_string_lossy(),
        );
    }

    pub fn activate(&self, venv: &Venv) {
        if venv.path.exists() {
            println!("export VIRTUAL_ENV='{}'", venv.path.to_string_lossy());
            println!(
                "export PATH='{}'",
                Shell::extend_path(&venv.scripts_path()).to_string_lossy()
            );
        }
    }

    pub fn enter(&self, venv: &Venv) -> anyhow::Result<()> {
        let path = Self::extend_path(&venv.scripts_path());
        Err(anyhow::Error::from(
            Command::new(&self.kind)
                .env("VIRTUAL_ENV", &venv.path)
                .env("PATH", path)
                .exec(),
        ))
    }

    pub fn extend_path(path: &Path) -> OsString {
        let mut paths = vec![];
        if let Some(path_var) = std::env::var_os("PATH") {
            if !path_var
                .to_string_lossy()
                .contains(&*path.to_string_lossy())
            {
                paths.push(path.to_path_buf());
            }
            paths.extend(std::env::split_paths(&path_var));
        } else {
            paths.push(path.to_path_buf());
        };

        let path = std::env::join_paths(paths).ok();
        path.unwrap_or("".into())
    }

    pub fn run(&self, venv: &Venv, command: Option<&str>, args: &[String]) -> anyhow::Result<()> {
        let scripts_path = venv.scripts_path();
        match command {
            Some(command) => {
                let bin = scripts_path.join(command);
                if !&bin.exists() {
                    anyhow::bail!("No such command: {command}");
                }

                let stderr = os_pipe::dup_stderr()?;
                let mut child = Command::new(bin.to_string_lossy().as_ref())
                    .args(args)
                    .env("VIRTUAL_ENV", &venv.path)
                    .stdout(stderr)
                    .spawn()?;

                child.wait()?;
            }
            None => {
                let scripts = scripts_path
                    .read_dir()?
                    .filter_map(Result::ok)
                    .map(|f| f.file_name().to_string_lossy().to_string())
                    .collect::<Vec<String>>()
                    .join(", ");

                eprintln!("Available scripts: {scripts}");
            }
        }
        Ok(())
    }

    pub fn exec(&self, venv: &Venv, command: &str, args: &[String]) -> anyhow::Result<()> {
        let args = [&["-c".to_string(), command.to_string()], args].concat();

        let stderr = os_pipe::dup_stderr()?;
        let mut child = Command::new(&self.kind)
            .args(args)
            .env("VIRTUAL_ENV", &venv.path)
            .stdout(stderr)
            .spawn()?;

        child.wait()?;

        Ok(())
    }

    pub fn prompt(&self, venv: &Venv) -> anyhow::Result<()> {
        let python = venv.scripts_path().join("python");

        Err(anyhow::Error::from(
            Command::new(python).env("VIRTUAL_ENV", &venv.path).exec(),
        ))
    }
}
