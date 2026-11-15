//! User preferences; filesystem work runs before GPUI or on a dedicated worker.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Default, Serialize, Deserialize)]
struct Preferences {
    #[serde(default)]
    wide: bool,
}

pub struct Settings {
    sender: async_channel::Sender<bool>,
}

/// Flushed after the UI event loop has returned, without blocking the UI thread.
pub struct Shutdown {
    sender: async_channel::Sender<bool>,
    worker: std::thread::JoinHandle<()>,
}

impl Shutdown {
    pub fn finish(self) {
        self.sender.close();
        if self.worker.join().is_err() {
            eprintln!("Не удалось завершить сохранение настроек");
        }
    }
}

impl Settings {
    pub fn open() -> (Self, bool, Shutdown) {
        let path = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .map(|root| root.join("IslandVolt").join("settings.json"));
        Self::start(path)
    }

    fn start(path: Option<PathBuf>) -> (Self, bool, Shutdown) {
        let wide = path
            .as_ref()
            .and_then(|path| std::fs::read(path).ok())
            .and_then(|bytes| serde_json::from_slice::<Preferences>(&bytes).ok())
            .unwrap_or_default()
            .wide;
        let (sender, receiver) = async_channel::bounded(1);
        let worker = std::thread::spawn(move || {
            while let Ok(wide) = receiver.recv_blocking() {
                let Some(path) = path.as_ref() else {
                    eprintln!("Настройки не сохранены: папка AppData недоступна");
                    continue;
                };
                if let Err(error) = write(path, wide) {
                    eprintln!("Не удалось сохранить настройки: {error}");
                }
            }
        });
        let shutdown = Shutdown {
            sender: sender.clone(),
            worker,
        };
        (Self { sender }, wide, shutdown)
    }

    pub fn save_width(&self, wide: bool) {
        // Only the latest preference matters while an older value is being saved.
        let _ = self.sender.force_send(wide);
    }
}

impl Drop for Settings {
    fn drop(&mut self) {
        self.sender.close();
    }
}

fn write(path: &std::path::Path, wide: bool) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension("json.tmp");
    std::fs::write(
        &temporary,
        serde_json::to_vec_pretty(&Preferences { wide })?,
    )?;
    std::fs::rename(temporary, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shutdown_flushes_latest_preference() {
        let folder =
            std::env::temp_dir().join(format!("islandvolt-shutdown-{}", std::process::id()));
        let path = folder.join("settings.json");
        let (settings, _, shutdown) = Settings::start(Some(path.clone()));
        settings.save_width(false);
        settings.save_width(true);
        shutdown.finish();
        let saved: Preferences =
            serde_json::from_slice(&std::fs::read(&path).expect("saved settings")).expect("parse");
        assert!(saved.wide);
        std::fs::remove_file(path).expect("remove test file");
        std::fs::remove_dir(folder).expect("remove test folder");
    }
    #[test]
    fn preference_can_be_replaced_and_read_back() {
        let folder =
            std::env::temp_dir().join(format!("islandvolt-settings-{}", std::process::id()));
        let path = folder.join("settings.json");
        write(&path, true).expect("save");
        write(&path, false).expect("replace");
        let preferences: Preferences =
            serde_json::from_slice(&std::fs::read(&path).expect("read")).expect("parse");
        assert!(!preferences.wide);
        std::fs::remove_file(&path).expect("remove test file");
        std::fs::remove_dir(&folder).expect("remove test directory");
    }
}
