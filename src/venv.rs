use corpus::{builder, Corpus, RootLocation};
use indoc::formatdoc;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use symlink::symlink_file;

use crate::python::Python;
use crate::settings::{Settings, Strategy};

pub struct Venv {
    pub settings: Settings,
    pub path: PathBuf,
    pub name: String,
    pub python: Option<Python>,
}

impl Venv {
    pub fn from_settings(settings: Settings) -> anyhow::Result<Venv> {
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
        let venv_name = settings.venv_name.clone();
        Ok(Venv::new(settings, path, venv_name))
    }
    pub fn new(settings: Settings, root_path: PathBuf, name: String) -> Self {
        let path = root_path.join(&name);
        Self {
            settings,
            path,
            name,
            python: None,
        }
    }

    fn require_python(&mut self) -> anyhow::Result<()> {
        let python = Python::detect(&self.settings.python_path)?;
        self.python = Some(python);
        Ok(())
    }

    fn get_python(&self) -> &Python {
        self.python.as_ref().unwrap()
    }

    fn exists(&self) -> bool {
        self.path.exists()
    }

    pub fn scripts_path(&self) -> PathBuf {
        self.path.join("bin")
    }

    pub fn include_path(&self) -> PathBuf {
        self.path.join("include")
    }

    pub fn lib_path(&self) -> PathBuf {
        self.path.join("lib")
    }

    pub fn site_packages_path(&self, python: &Python) -> PathBuf {
        self.lib_path().join(format!(
            "python{major}.{minor}/site-packages",
            major = python.major,
            minor = python.minor
        ))
    }

    pub fn create(&mut self, fix: bool) -> anyhow::Result<()> {
        if !fix && self.exists() {
            return Ok(());
        }

        self.require_python()?;
        let python = self.get_python();
        let scripts_path = self.scripts_path();

        if !self.path.exists() {
            std::fs::create_dir_all(&self.path)?;
        }

        if !scripts_path.exists() {
            std::fs::create_dir_all(&scripts_path)?;
        }
        std::fs::create_dir_all(self.include_path())?;
        std::fs::create_dir_all(self.site_packages_path(python))?;

        if cfg!(target_pointer_width = "64")
            && cfg!(target_family = "unix")
            && cfg!(not(target_os = "macos"))
        {
            let lib64_path = self.path.join("lib64");
            if !lib64_path.exists() {
                std::os::unix::fs::symlink(self.lib_path(), lib64_path)?;
            }
        }
        symlink_file(&python.exe_path, scripts_path.join("python"))?;
        symlink_file(
            &python.exe_path,
            scripts_path.join(format!(
                "python{major}.{minor}",
                major = python.major,
                minor = python.minor
            )),
        )?;
        symlink_file(
            &python.exe_path,
            scripts_path.join(format!(
                "python{major}.{minor}.{patch}",
                major = python.major,
                minor = python.minor,
                patch = python.patch,
            )),
        )?;

        let pyvenv_cfg = self.path.join("pyvenv.cfg");
        if !pyvenv_cfg.exists() {
            let pyvenv_content = formatdoc!(
                r#"
                home = {scripts_path}
                include-system-site-packages = false
                version = {major}.{minor}.{patch}
                executable = {python_executable}
                "#,
                major = python.major,
                minor = python.minor,
                patch = python.patch,
                scripts_path = scripts_path.to_string_lossy(),
                python_executable = python.exe_path.to_string_lossy(),
            );

            let mut pyvenv_file = File::create(pyvenv_cfg)?;
            pyvenv_file.write_all(pyvenv_content.as_ref())?;
        }

        python.ensure_pip()?;

        Ok(())
    }

    pub fn delete(&mut self) -> anyhow::Result<()> {
        Ok(std::fs::remove_dir_all(&self.path)?)
    }

    pub fn print_info(&self) {
        let exists = if self.path.exists() {
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
            venv = self.path.to_string_lossy(),
            exists = exists,
            name = self.name,
            activated = std::env::var_os("VIRTUAL_ENV")
                .map(|e| e == self.path)
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
