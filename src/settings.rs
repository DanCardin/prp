use etcetera::base_strategy::{BaseStrategy, Xdg};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use toml_edit::Document;

pub struct Settings {
    pub name: String,
    pub config_file: PathBuf,

    pub venv_name: String,
    pub strategy: Strategy,
    pub project_root: Vec<String>,
    pub auto_activate: bool,
    pub python_path: PathBuf,

    pub executables_path: PathBuf,
}

impl Settings {
    pub fn read(name: &str) -> anyhow::Result<Self> {
        let strategy = Xdg::new()?;
        let config_dir = strategy.config_dir().join(name);
        let data_dir = strategy.data_dir().join(name);

        let config_file = config_dir.with_extension("toml");

        let content = read_file(&config_file);
        let document = &get_document(content);

        let venv_name = document
            .get("venv-name")
            .and_then(|v| v.as_str())
            .unwrap_or(".venv")
            .to_string();

        let strategy = Strategy::from(document.get("strategy").and_then(|v| v.as_str()));
        let project_root = document
            .get("project-root")
            .and_then(|t| t.as_array())
            .map(|t| {
                t.iter()
                    .map(|v| v.as_str().unwrap_or("").to_string())
                    .collect()
            })
            .unwrap_or_else(|| {
                ["pyproject.toml", "setup.py", "setup.cfg", ".gitignore"]
                    .map(String::from)
                    .into_iter()
                    .collect()
            });

        let auto_activate = document
            .get("auto-activate")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let python_path = Path::new(
            document
                .get("python-path")
                .and_then(|v| v.as_str())
                .unwrap_or("python"),
        )
        .to_path_buf();

        let executables_path = document
            .get("executables-path")
            .and_then(|v| v.as_str())
            .map(|v| Path::new(v).to_path_buf())
            .unwrap_or(data_dir.join("venvs"));

        Ok(Self {
            name: name.to_string(),
            config_file,
            venv_name,
            strategy,
            project_root,
            auto_activate,
            python_path,
            executables_path,
        })
    }

    pub fn set_python_path(&mut self, maybe_path: Option<PathBuf>) {
        if let Some(path) = maybe_path {
            self.python_path = path;
        }
    }

    pub fn set_venv_name(&mut self, maybe_name: Option<String>) {
        if let Some(name) = maybe_name {
            self.venv_name = name;
        }
    }
}

pub enum Strategy {
    Local,
    Central,
}

impl From<Option<&str>> for Strategy {
    fn from(value: Option<&str>) -> Self {
        match value {
            Some("local") => Self::Local,
            Some("central") => Self::Central,
            _ => Self::Local,
        }
    }
}

fn read_file(path: &Path) -> String {
    if let Ok(file) = File::open(path) {
        let mut reader = BufReader::new(file);

        let mut contents = String::new();
        reader.read_to_string(&mut contents).unwrap_or(0);
        contents
    } else {
        String::new()
    }
}

fn get_document(contents: String) -> Document {
    contents.parse::<Document>().unwrap_or(Document::new())
}
