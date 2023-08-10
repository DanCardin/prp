use regex::Regex;
use std::path::{Path, PathBuf};
use std::process::Command;
use which::which;

pub struct Python {
    pub exe_path: PathBuf,
    pub major: String,
    pub minor: String,
    pub patch: String,
}

impl Python {
    pub fn detect(python_path: &Path) -> anyhow::Result<Self> {
        let exe_path = if python_path.is_absolute() {
            python_path.to_path_buf()
        } else {
            which(python_path)?
        };
        let (major, minor, patch) = Self::compute_version(&exe_path)?;

        Ok(Self {
            exe_path,
            major,
            minor,
            patch,
        })
    }

    pub fn ensure_pip(&self) -> anyhow::Result<()> {
        Command::new("python")
            .args(["-m", "ensurepip", "--upgrade", "--default-pip"])
            .output()?;
        Ok(())
    }

    fn compute_version(path: &Path) -> anyhow::Result<(String, String, String)> {
        let output = Command::new(path).args(["-V"]).output()?;
        let output = String::from_utf8(output.stdout)?;

        let re = Regex::new(r"Python ([0-9]+)\.([0-9]+)\.([0-9]+)")?;
        if let Some(capture) = re.captures(&output) {
            let major = capture.get(1).expect("").as_str().to_string();
            let minor = capture.get(2).expect("").as_str().to_string();
            let patch = capture.get(3).expect("").as_str().to_string();
            Ok((major, minor, patch))
        } else {
            anyhow::bail!("");
        }
    }
}
