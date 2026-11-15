//! User preferences; filesystem work runs before GPUI or on a dedicated worker.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Copy, Default, Serialize, Deserialize)]
pub(crate) struct Preferences {
    #[serde(default)]
    pub wide: bool,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub pet_hidden: bool,
}

pub struct Settings {
    sender: async_channel::Sender<Preferences>,
    current: std::sync::Mutex<Preferences>,
}

/// Flushed after the UI event loop has returned, without blocking the UI thread.
pub struct Shutdown {
    sender: async_channel::Sender<Preferences>,
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
    pub fn open() -> (Self, Preferences, Shutdown) {
        let path = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .map(|root| root.join("IslandVolt").join("settings.json"));
        Self::start(path)
    }

    fn start(path: Option<PathBuf>) -> (Self, Preferences, Shutdown) {
        let preferences = path
            .as_ref()
            .and_then(|path| std::fs::read(path).ok())
            .and_then(|bytes| serde_json::from_slice::<Preferences>(&bytes).ok())
            .unwrap_or_default();
        let (sender, receiver) = async_channel::bounded(1);
        let worker = std::thread::spawn(move || {
            while let Ok(preferences) = receiver.recv_blocking() {
                let Some(path) = path.as_ref() else {
                    eprintln!("Настройки не сохранены: папка AppData недоступна");
                    continue;
                };
                if let Err(error) = write(path, preferences) {
                    eprintln!("Не удалось сохранить настройки: {error}");
                }
            }
        });
        let shutdown = Shutdown {
            sender: sender.clone(),
            worker,
        };
        (
            Self {
                sender,
                current: std::sync::Mutex::new(preferences),
            },
            preferences,
            shutdown,
        )
    }

    pub fn save_width(&self, wide: bool) {
        if let Ok(mut preferences) = self.current.lock() {
            preferences.wide = wide;
            let _ = self.sender.force_send(*preferences);
        }
    }

    pub fn save_pinned(&self, pinned: bool) {
        if let Ok(mut preferences) = self.current.lock() {
            preferences.pinned = pinned;
            let _ = self.sender.force_send(*preferences);
        }
    }

    pub fn save_pet_hidden(&self, hidden: bool) {
        if let Ok(mut preferences) = self.current.lock() {
            preferences.pet_hidden = hidden;
            let _ = self.sender.force_send(*preferences);
        }
    }
}

impl Drop for Settings {
    fn drop(&mut self) {
        self.sender.close();
    }
}

fn write(path: &std::path::Path, preferences: Preferences) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension("json.tmp");
    std::fs::write(&temporary, serde_json::to_vec_pretty(&preferences)?)?;
    std::fs::rename(temporary, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn old_preferences_keep_pet_visible_without_resetting_other_choices() {
        let saved: Preferences = serde_json::from_str(r#"{"wide":true,"pinned":true}"#)
            .expect("existing preference format");
        assert!(saved.wide);
        assert!(saved.pinned);
        assert!(!saved.pet_hidden);
    }
    #[test]
    fn shutdown_flushes_latest_preference() {
        let folder =
            std::env::temp_dir().join(format!("islandvolt-shutdown-{}", std::process::id()));
        let path = folder.join("settings.json");
        let (settings, _, shutdown) = Settings::start(Some(path.clone()));
        settings.save_width(false);
        settings.save_width(true);
        settings.save_pinned(true);
        settings.save_pet_hidden(true);
        shutdown.finish();
        let saved: Preferences =
            serde_json::from_slice(&std::fs::read(&path).expect("saved settings")).expect("parse");
        assert!(saved.wide);
        assert!(saved.pinned);
        assert!(saved.pet_hidden);
        std::fs::remove_file(path).expect("remove test file");
        std::fs::remove_dir(folder).expect("remove test folder");
    }
    #[test]
    fn preference_can_be_replaced_and_read_back() {
        let folder =
            std::env::temp_dir().join(format!("islandvolt-settings-{}", std::process::id()));
        let path = folder.join("settings.json");
        write(
            &path,
            Preferences {
                wide: true,
                ..Default::default()
            },
        )
        .expect("save");
        write(&path, Preferences::default()).expect("replace");
        let preferences: Preferences =
            serde_json::from_slice(&std::fs::read(&path).expect("read")).expect("parse");
        assert!(!preferences.wide);
        std::fs::remove_file(&path).expect("remove test file");
        std::fs::remove_dir(&folder).expect("remove test directory");
    }
}
