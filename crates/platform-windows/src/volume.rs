//! Windows endpoint volume, accessed only on a dedicated COM thread.

pub(crate) mod notifications;

use async_channel::{Receiver, Sender};
use notifications::{device_callback, volume_callback};
use windows::Win32::{
    Media::Audio::{
        Endpoints::IAudioEndpointVolume, IMMDeviceEnumerator, MMDeviceEnumerator, eConsole, eRender,
    },
    System::Com::{
        CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VolumeState {
    pub percent: u8,
    pub muted: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum Action {
    Increase,
    Decrease,
    ToggleMute,
    Set(u8),
}

#[derive(Clone, Copy)]
pub(super) enum Message {
    Refresh,
    Rebind,
    Control(Action),
}

pub struct VolumeMonitor {
    sender: Sender<Message>,
}

impl VolumeMonitor {
    pub fn control(&self, action: Action) -> Result<(), &'static str> {
        self.sender
            .try_send(Message::Control(action))
            .map_err(|_| "Не удалось отправить команду громкости")
    }
}

impl Drop for VolumeMonitor {
    fn drop(&mut self) {
        self.sender.close();
    }
}

pub fn watch() -> (VolumeMonitor, Receiver<Result<VolumeState, String>>) {
    let (sender, events) = async_channel::bounded(16);
    let (updates, receiver) = async_channel::bounded(16);
    let notify = sender.clone();
    std::thread::spawn(move || {
        // SAFETY: initialization and cleanup happen on the same dedicated thread.
        let initialized = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_ok();
        if initialized {
            if let Err(error) = run(&events, &notify, &updates) {
                let _ = updates.try_send(Err(error.to_string()));
            }
            // SAFETY: balances the successful initialization on this thread.
            unsafe { CoUninitialize() };
        } else {
            let _ = updates.try_send(Err("Не удалось инициализировать COM".into()));
        }
    });
    (VolumeMonitor { sender }, receiver)
}

fn run(
    events: &Receiver<Message>,
    notify: &Sender<Message>,
    updates: &Sender<Result<VolumeState, String>>,
) -> windows::core::Result<()> {
    // SAFETY: all COM interfaces below belong to this initialized thread.
    let enumerator: IMMDeviceEnumerator =
        unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)? };
    let device_signal = notify.clone();
    let devices = device_callback(move || {
        let _ = device_signal.try_send(Message::Rebind);
    });
    let callback = volume_callback(notify.clone());
    unsafe { enumerator.RegisterEndpointNotificationCallback(&devices)? };
    let mut endpoint: Option<IAudioEndpointVolume> = None;
    let _ = notify.try_send(Message::Rebind);
    while let Ok(message) = events.recv_blocking() {
        if matches!(message, Message::Rebind) {
            if let Some(old) = endpoint.take() {
                // SAFETY: the callback was registered on this endpoint.
                let _ = unsafe { old.UnregisterControlChangeNotify(&callback) };
            }
            // SAFETY: obtained interfaces are retained and used on this thread.
            endpoint = unsafe {
                enumerator
                    .GetDefaultAudioEndpoint(eRender, eConsole)
                    .and_then(|device| device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None))
                    .and_then(|volume| {
                        volume.RegisterControlChangeNotify(&callback)?;
                        Ok(volume)
                    })
            }
            .ok();
        }
        let result = endpoint
            .as_ref()
            .ok_or_else(|| "Нет доступного устройства вывода".to_string())
            .and_then(|device| apply_and_read(device, message));
        if updates.try_send(result).is_err() && updates.is_closed() {
            break;
        }
    }
    if let Some(old) = endpoint {
        // SAFETY: callback was registered on this endpoint.
        let _ = unsafe { old.UnregisterControlChangeNotify(&callback) };
    }
    // SAFETY: unregister the callback before releasing the enumerator.
    let _ = unsafe { enumerator.UnregisterEndpointNotificationCallback(&devices) };
    Ok(())
}

fn apply_and_read(device: &IAudioEndpointVolume, message: Message) -> Result<VolumeState, String> {
    // SAFETY: endpoint is live on its COM thread; null event context is allowed.
    unsafe {
        let current = device
            .GetMasterVolumeLevelScalar()
            .map_err(|error| error.to_string())?;
        let muted = device
            .GetMute()
            .map_err(|error| error.to_string())?
            .as_bool();
        match message {
            Message::Control(Action::Increase) => {
                device.SetMasterVolumeLevelScalar((current + 0.05).min(1.0), std::ptr::null())
            }
            Message::Control(Action::Decrease) => {
                device.SetMasterVolumeLevelScalar((current - 0.05).max(0.0), std::ptr::null())
            }
            Message::Control(Action::ToggleMute) => device.SetMute(!muted, std::ptr::null()),
            Message::Control(Action::Set(value)) => device
                .SetMasterVolumeLevelScalar(f32::from(value.min(100)) / 100.0, std::ptr::null()),
            _ => Ok(()),
        }
        .map_err(|error| error.to_string())?;
        let level = device
            .GetMasterVolumeLevelScalar()
            .map_err(|error| error.to_string())?;
        let muted = device
            .GetMute()
            .map_err(|error| error.to_string())?
            .as_bool();
        Ok(VolumeState {
            percent: (level.clamp(0.0, 1.0) * 100.0).round() as u8,
            muted,
        })
    }
}
