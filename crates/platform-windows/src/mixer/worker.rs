use super::{Message, Snapshot, callbacks, names, sessions::Sessions};
use crate::volume::notifications::device_callback;
use async_channel::{Receiver, Sender};
use windows::{
    Win32::{
        Media::Audio::{
            IAudioSessionManager2, IAudioSessionNotification, IMMDevice, IMMDeviceEnumerator,
            MMDeviceEnumerator, eConsole, eRender,
        },
        System::Com::{
            CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
        },
    },
    core::Result,
};

pub(super) fn start(
    events: Receiver<Message>,
    commands: Sender<Message>,
    updates: Sender<Snapshot>,
) {
    std::thread::spawn(move || {
        // SAFETY: the native interfaces are created and released on this MTA thread.
        if unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_err() {
            let _ = updates.force_send(unavailable("Не удалось запустить микшер"));
            return;
        }
        let result = run(events, commands, &updates);
        // SAFETY: run has dropped all COM interfaces and this balances CoInitializeEx.
        unsafe { CoUninitialize() };
        if result.is_err() {
            let _ = updates.force_send(unavailable("Служба звука Windows недоступна"));
        }
    });
}

fn run(
    events: Receiver<Message>,
    commands: Sender<Message>,
    updates: &Sender<Snapshot>,
) -> Result<()> {
    // SAFETY: the worker has initialized COM in MTA.
    let devices: IMMDeviceEnumerator =
        unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)? };
    let signal = commands.clone();
    let device_events = device_callback(move || {
        let _ = signal.try_send(Message::Refresh);
    });
    let changed = callbacks::changed(commands.clone());
    let created = callbacks::created(commands);
    // SAFETY: unregister the paired callback before releasing the device enumerator.
    unsafe { devices.RegisterEndpointNotificationCallback(&device_events)? };
    let mut current: Option<Endpoint> = None;
    let mut sessions = Sessions::default();
    let mut next = Some(Message::Refresh);
    while let Some(message) = next {
        if updates.is_closed() {
            break;
        }
        // SAFETY: Core Audio is used on this COM thread only.
        let device = unsafe { devices.GetDefaultAudioEndpoint(eRender, eConsole) }
            .and_then(|device| Ok((names::device_id(&device)?, device)));
        let snapshot = match device {
            Ok((key, device)) => {
                if current.as_ref().is_none_or(|endpoint| endpoint.key != key) {
                    sessions.clear();
                    current = Endpoint::bind(key, &device, created.clone()).ok();
                }
                match current.as_ref() {
                    Some(endpoint) => {
                        let mut warning = None;
                        if let Message::Adjust(id, action) = message
                            && sessions.adjust(id, action).is_err()
                        {
                            warning = Some(
                                "Не удалось изменить громкость. Сессия могла завершиться.".into(),
                            );
                        }
                        match sessions.refresh(&endpoint.manager, &changed) {
                            Ok(sessions) => Snapshot {
                                output: Some(endpoint.name.clone()),
                                sessions,
                                warning,
                            },
                            Err(_) => Snapshot {
                                output: Some(endpoint.name.clone()),
                                ..unavailable("Не удалось прочитать аудиосессии. Обновите список.")
                            },
                        }
                    }
                    None => unavailable("Не удалось открыть микшер устройства. Обновите список."),
                }
            }
            Err(_) => {
                sessions.clear();
                current = None;
                unavailable("Устройство вывода не найдено")
            }
        };
        if updates.force_send(snapshot).is_err() {
            break;
        }
        next = events.recv_blocking().ok();
    }
    sessions.clear();
    drop(current);
    // SAFETY: paired with the successful registration above.
    let _ = unsafe { devices.UnregisterEndpointNotificationCallback(&device_events) };
    Ok(())
}

fn unavailable(message: &str) -> Snapshot {
    Snapshot {
        warning: Some(message.into()),
        ..Default::default()
    }
}

struct Endpoint {
    key: String,
    name: String,
    manager: IAudioSessionManager2,
    created: IAudioSessionNotification,
}

impl Endpoint {
    fn bind(key: String, device: &IMMDevice, created: IAudioSessionNotification) -> Result<Self> {
        // SAFETY: device is valid on this initialized thread. GetCount primes notifications.
        let manager = unsafe {
            let manager = device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None)?;
            manager.RegisterSessionNotification(&created)?;
            manager
        };
        Ok(Self {
            key,
            name: names::device_name(device)
                .unwrap_or_else(|| "Устройство Windows по умолчанию".into()),
            manager,
            created,
        })
    }
}

impl Drop for Endpoint {
    fn drop(&mut self) {
        // SAFETY: the callback was registered on this manager and both are still live.
        let _ = unsafe { self.manager.UnregisterSessionNotification(&self.created) };
    }
}
