use corpus::{builder, Corpus, RootLocation};
use indoc::formatdoc;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use symlink::symlink_file;

use crate::package_specifier::PackageSpecifier;
use crate::python::Python;
use crate::settings::{Settings, Strategy};

pub struct VenvPaths {
    pub path: PathBuf,

    pub scripts_path: PathBuf,
    pub include_path: PathBuf,
    pub lib_path: PathBuf,
    pub lib64_path: PathBuf,

    pub python_path: PathBuf,
    pub pip_path: PathBuf,
    pub pyvenv_cfg: PathBuf,
}

impl VenvPaths {
    pub fn new(path: &Path) -> Self {
        let scripts_path = path.join("bin");
        Self {
            path: path.to_path_buf(),
            scripts_path: scripts_path.clone(),
            include_path: path.join("include"),
            lib_path: path.join("lib"),
            lib64_path: path.join("lib64"),

            python_path: scripts_path.join("python"),
            pip_path: scripts_path.join("pip"),
            pyvenv_cfg: path.join("pyvenv.cfg"),
        }
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    pub fn script(&self, name: &str) -> PathBuf {
        self.scripts_path.join(name)
    }

    pub fn site_packages_path(&self, python: &Python) -> PathBuf {
        self.lib_path.join(format!(
            "python{major}.{minor}/site-packages",
            major = python.major,
            minor = python.minor
        ))
    }
    pub fn python_path_major(&self, python: &Python) -> PathBuf {
        self.scripts_path
            .join(format!("python{major}", major = python.major))
    }

    pub fn python_path_minor(&self, python: &Python) -> PathBuf {
        self.scripts_path.join(format!(
            "python{major}.{minor}",
            major = python.major,
            minor = python.minor
        ))
    }

    pub fn python_path_patch(&self, python: &Python) -> PathBuf {
        self.scripts_path.join(format!(
            "python{major}.{minor}.{patch}",
            major = python.major,
            minor = python.minor,
            patch = python.patch,
        ))
    }

    pub fn pyvenv_cfg_content(&self, python: &Python) -> String {
        formatdoc!(
            r#"
                home = {scripts_path}
                include-system-site-packages = false
                version = {major}.{minor}.{patch}
                executable = {python_executable}
                "#,
            major = python.major,
            minor = python.minor,
            patch = python.patch,
            scripts_path = self.scripts_path.to_string_lossy(),
            python_executable = python.exe_path.to_string_lossy(),
        )
    }
}

pub struct Venv {
    pub python_path: PathBuf,
    pub paths: VenvPaths,
    pub name: String,
}

impl Venv {
    pub fn from_current_dir(settings: &Settings) -> anyhow::Result<Venv> {
        let current_dir = std::env::current_dir()?;
        let dir = find_project_root(&current_dir, &settings.project_root);

        let path = match settings.strategy {
            Strategy::Local => dir,
            Strategy::Central => {
                let corpus: Corpus = builder()
                    .relative_to_home()?
                    .with_root(RootLocation::XDGData)
                    .with_name(&settings.name)
                    .build()?;

                corpus.path(dir.as_path())
            }
        };
        Ok(Venv::new(settings, path, &settings.venv_name))
    }

    pub fn from_package_name(settings: &Settings, package_name: &str) -> Venv {
        Self::new(settings, settings.executables_path.clone(), package_name)
    }

    pub fn new(settings: &Settings, root_path: PathBuf, name: &str) -> Self {
        let path = root_path.join(name);
        Self {
            python_path: settings.python_path.clone(),
            paths: VenvPaths::new(&path),
            name: name.to_string(),
        }
    }

    pub fn exists(&self) -> bool {
        self.paths.exists()
    }

    pub fn create(&mut self, fix: bool) -> anyhow::Result<()> {
        if !fix && self.paths.exists() {
            return Ok(());
        }

        let python = Python::detect(&self.python_path)?;

        let site_packages = self.paths.site_packages_path(&python);
        let mut required_paths = vec![
            &self.paths.path,
            &self.paths.scripts_path,
            &self.paths.include_path,
            &site_packages,
        ];

        if cfg!(target_pointer_width = "64")
            && cfg!(target_family = "unix")
            && cfg!(not(target_os = "macos"))
        {
            required_paths.push(&self.paths.lib64_path);
        }

        for path in required_paths {
            if !path.exists() {
                std::fs::create_dir_all(path)?;
            }
        }

        create_symlink(&python.exe_path, &self.paths.python_path)?;
        create_symlink(&python.exe_path, &self.paths.python_path_major(&python))?;
        create_symlink(&python.exe_path, &self.paths.python_path_minor(&python))?;
        create_symlink(&python.exe_path, &self.paths.python_path_patch(&python))?;

        if !self.paths.pyvenv_cfg.exists() {
            let mut pyvenv_file = File::create(&self.paths.pyvenv_cfg)?;
            pyvenv_file.write_all(self.paths.pyvenv_cfg_content(&python).as_ref())?;
        }

        self.ensure_pip()?;

        Ok(())
    }

    fn ensure_pip(&self) -> anyhow::Result<()> {
        Command::new(&self.paths.python_path)
            .args(["-m", "ensurepip", "--upgrade", "--default-pip"])
            .env("VIRTUAL_ENV", &self.paths.path)
            .output()?;

        self.pip(&["install", "--upgrade", "pip"])?;
        Ok(())
    }

    pub fn delete(&mut self) -> anyhow::Result<()> {
        Ok(std::fs::remove_dir_all(&self.paths.path)?)
    }

    pub fn pip(&self, command: &[&str]) -> anyhow::Result<(String, String)> {
        let output: Output = Command::new(self.paths.pip_path.to_string_lossy().as_ref())
            .args(command)
            .env("VIRTUAL_ENV", &self.paths.path)
            .output()?;

        if output.status.success() {
            Ok((
                String::from_utf8(output.stdout)?,
                String::from_utf8(output.stderr)?,
            ))
        } else {
            Err(anyhow::anyhow!("{}", String::from_utf8(output.stderr)?))
        }
    }

    pub fn install(&self, package_spec: PackageSpecifier) -> anyhow::Result<()> {
        self.pip(&["install", &format!("{}", package_spec)])?;
        Ok(())
    }

    pub fn print_info(&self) {
        let exists = if self.paths.path.exists() {
            "exists"
        } else {
            "does not exist"
        };

        indoc::eprintdoc!(
            r#"
            Venv Path: {venv} ({exists})
            Venv Name: {name}
            Activated: {activated}
            "#,
            venv = self.paths.path.to_string_lossy(),
            exists = exists,
            name = self.name,
            activated = std::env::var_os("VIRTUAL_ENV")
                .map(|e| e == self.paths.path)
                .unwrap_or(false)
                .to_string(),
        )
    }
}

fn find_project_root(path: &Path, project_root: &[String]) -> PathBuf {
    for p in path.ancestors() {
        for root in project_root {
            if p.join(root).exists() {
                return p.to_path_buf();
            }
        }
    }
    path.to_path_buf()
}

fn create_symlink(source: &Path, dest: &Path) -> anyhow::Result<()> {
    let real_source = std::fs::canonicalize(source)?;

    if dest.exists() {
        let real_path = std::fs::canonicalize(dest)?;
        if real_path == source {
            return Ok(());
        }

        std::fs::remove_file(dest)?;
    }

    symlink_file(real_source, dest)?;
    Ok(())
}
