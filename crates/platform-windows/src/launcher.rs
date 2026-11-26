//! Explicit executable paths only; launching never invokes a command interpreter.
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

/// Resolve a chosen local .exe file. Revalidate before every launch.
pub fn validate(path: &Path) -> Result<PathBuf, String> {
    if !path.is_absolute() {
        return Err("Выберите полный путь к приложению .exe".into());
    }
    if !is_local(path) {
        return Err("Выберите приложение на локальном диске".into());
    }
    let target = path
        .canonicalize()
        .map_err(|_| "Приложение не найдено. Выберите файл заново.")?;
    if !is_local(&target) {
        return Err("Выберите приложение на локальном диске".into());
    }
    if !target.is_file()
        || !target
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"))
    {
        return Err("Выберите исполняемый файл .exe".into());
    }
    Ok(target)
}

fn is_local(path: &Path) -> bool {
    let value = path.to_string_lossy().to_uppercase();
    !value.starts_with(r"\\?\UNC\") && (!value.starts_with(r"\\") || value.starts_with(r"\\?\"))
}

/// Start the selected executable without arguments or shell expansion.
pub fn launch(path: &Path) -> Result<(), String> {
    let target = validate(path)?;
    let parent = target
        .parent()
        .ok_or("Не удалось определить папку приложения")?;
    Command::new(&target)
        .current_dir(parent)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "Не удалось запустить приложение. Проверьте файл и права доступа.")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_real_exe_and_rejects_relative_missing_and_non_executable_files() {
        let exe = std::env::current_exe().expect("test executable");
        assert!(validate(&exe).is_ok());
        assert!(validate(Path::new("cmd.exe")).is_err());
        assert!(validate(Path::new(r"\\server\share\app.exe")).is_err());
        assert!(validate(&exe.with_extension("missing.exe")).is_err());
        let directory = std::env::temp_dir();
        assert!(validate(&directory).is_err());
        let file = directory.join(format!("islandvolt-launcher-{}.txt", std::process::id()));
        std::fs::write(&file, "test").expect("fixture");
        assert!(validate(&file).is_err());
        std::fs::remove_file(file).expect("cleanup");
    }
}
