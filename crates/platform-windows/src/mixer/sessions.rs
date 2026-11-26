use super::{Action, Session, names};
use crate::volume::VolumeState;
use std::collections::BTreeMap;
use windows::{
    Win32::Media::Audio::{
        AudioSessionStateExpired, IAudioSessionControl, IAudioSessionControl2, IAudioSessionEvents,
        IAudioSessionManager2, ISimpleAudioVolume,
    },
    core::{Interface, Result},
};

struct Entry {
    id: u64,
    control: IAudioSessionControl,
    volume: ISimpleAudioVolume,
    callback: Option<IAudioSessionEvents>,
}

impl Drop for Entry {
    fn drop(&mut self) {
        if let Some(callback) = &self.callback {
            // SAFETY: this entry registered the callback, and its COM control is still live.
            let _ = unsafe { self.control.UnregisterAudioSessionNotification(callback) };
        }
    }
}

#[derive(Default)]
pub(super) struct Sessions {
    entries: BTreeMap<String, Entry>,
    next_id: u64,
}

impl Sessions {
    pub(super) fn clear(&mut self) {
        self.entries.clear();
    }

    pub(super) fn refresh(
        &mut self,
        manager: &IAudioSessionManager2,
        callback: &IAudioSessionEvents,
    ) -> Result<Vec<Session>> {
        // SAFETY: receive a fresh snapshot on the dedicated COM thread.
        let (enumerator, count) = unsafe {
            let enumerator = manager.GetSessionEnumerator()?;
            let count = enumerator.GetCount()?;
            (enumerator, count)
        };
        let mut previous = std::mem::take(&mut self.entries);
        let mut rows = Vec::new();
        for index in 0..count {
            // SAFETY: index is within the enumerator snapshot.
            let Ok(control) = (unsafe { enumerator.GetSession(index) }) else {
                continue;
            };
            let Ok(details) = control.cast::<IAudioSessionControl2>() else {
                continue;
            };
            let Ok(key) = names::session_id(&details) else {
                continue;
            };
            if key.is_empty() || self.entries.contains_key(&key) || !is_live(&control) {
                continue;
            }
            let entry = if let Some(entry) = previous.remove(&key) {
                entry
            } else {
                let Ok(volume) = control.cast::<ISimpleAudioVolume>() else {
                    continue;
                };
                self.next_id += 1;
                // SAFETY: the callback queues a refresh and never reenters WASAPI.
                let registered =
                    unsafe { control.RegisterAudioSessionNotification(callback) }.is_ok();
                Entry {
                    id: self.next_id,
                    control,
                    volume,
                    callback: registered.then(|| callback.clone()),
                }
            };
            if let Ok(volume) = entry.read() {
                rows.push(Session {
                    id: entry.id,
                    name: names::session_name(&details),
                    volume,
                });
                self.entries.insert(key, entry);
            }
        }
        rows.sort_by(|a, b| {
            a.name
                .to_lowercase()
                .cmp(&b.name.to_lowercase())
                .then(a.id.cmp(&b.id))
        });
        for group in rows.chunk_by_mut(|a, b| a.name == b.name) {
            if group.len() > 1 {
                for row in group {
                    row.name = format!("{} · №{}", row.name, row.id);
                }
            }
        }
        Ok(rows)
    }

    pub(super) fn adjust(&self, id: u64, action: Action) -> Result<()> {
        let entry = self
            .entries
            .values()
            .find(|entry| entry.id == id && is_live(&entry.control))
            .ok_or_else(|| {
                windows::core::Error::new(
                    windows::core::HRESULT::from_win32(
                        windows::Win32::Foundation::ERROR_NOT_FOUND.0,
                    ),
                    "Аудиосессия завершилась",
                )
            })?;
        // SAFETY: the ID refers to a live session of the current device. Null context is allowed.
        unsafe {
            match action {
                Action::Set(percent) => entry
                    .volume
                    .SetMasterVolume(f32::from(percent.min(100)) / 100.0, std::ptr::null()),
                Action::Step(delta) => {
                    let level = entry.volume.GetMasterVolume()? + f32::from(delta) / 100.0;
                    entry
                        .volume
                        .SetMasterVolume(level.clamp(0.0, 1.0), std::ptr::null())
                }
                Action::ToggleMute => entry
                    .volume
                    .SetMute(!entry.volume.GetMute()?.as_bool(), std::ptr::null()),
            }
        }
    }
}

fn is_live(control: &IAudioSessionControl) -> bool {
    // SAFETY: the control is live on the initialized COM thread.
    unsafe { control.GetState() }.is_ok_and(|state| state != AudioSessionStateExpired)
}

impl Entry {
    fn read(&self) -> Result<VolumeState> {
        // SAFETY: read-only calls to the volume interface held by this entry.
        let (level, muted) = unsafe { (self.volume.GetMasterVolume()?, self.volume.GetMute()?) };
        Ok(VolumeState {
            percent: (level.clamp(0.0, 1.0) * 100.0).round() as u8,
            muted: muted.as_bool(),
        })
    }
}
