use islandvolt_platform_windows::mixer::{self, Action, Snapshot};
use std::time::{Duration, Instant};
use windows::{
    Win32::{
        Media::Audio::{
            AUDCLNT_SHAREMODE_SHARED, IAudioClient, IAudioSessionControl, IMMDeviceEnumerator,
            ISimpleAudioVolume, MMDeviceEnumerator, eConsole, eRender,
        },
        System::Com::{
            CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateGuid, CoCreateInstance, CoInitializeEx,
            CoTaskMemFree, CoUninitialize,
        },
    },
    core::{Result, w},
};

const TEST_NAME: &str = "IslandVolt mixer integration fixture";

struct Apartment;

impl Apartment {
    fn start() -> Result<Self> {
        // SAFETY: this test owns the thread's matching COM initialization guard.
        unsafe { CoInitializeEx(None, COINIT_MULTITHREADED).ok()? };
        Ok(Self)
    }
}

impl Drop for Apartment {
    fn drop(&mut self) {
        // SAFETY: balances the successful initialization on the same test thread.
        unsafe { CoUninitialize() };
    }
}

fn fixture() -> Result<(IAudioClient, ISimpleAudioVolume)> {
    // SAFETY: COM is initialized by the caller. A unique, stopped stream never plays sound
    // or reuses a real application's volume settings. The native format is released.
    unsafe {
        let devices: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
        let device = devices.GetDefaultAudioEndpoint(eRender, eConsole)?;
        let audio = device.Activate::<IAudioClient>(CLSCTX_ALL, None)?;
        let id = CoCreateGuid()?;
        let format = audio.GetMixFormat()?;
        let result = audio.Initialize(AUDCLNT_SHAREMODE_SHARED, 0, 0, 0, format, Some(&id));
        CoTaskMemFree(Some(format.cast()));
        result?;
        let session = audio.GetService::<IAudioSessionControl>()?;
        session.SetDisplayName(w!("IslandVolt mixer integration fixture"), std::ptr::null())?;
        let volume = audio.GetService::<ISimpleAudioVolume>()?;
        Ok((audio, volume))
    }
}

#[test]
fn observes_and_controls_only_a_new_native_audio_session() -> anyhow::Result<()> {
    let _apartment = Apartment::start()?;
    let (mixer, events) = mixer::open();
    wait_for(&events, |snapshot| snapshot.output.as_ref().cloned())?;
    let (_stream, volume) = fixture()?;
    let id = wait_for(&events, |snapshot| {
        snapshot
            .sessions
            .iter()
            .find(|row| row.name == TEST_NAME)
            .map(|row| row.id)
    })?;
    mixer
        .adjust(id, Action::Set(37))
        .map_err(anyhow::Error::msg)?;
    wait_for(&events, |snapshot| {
        snapshot
            .sessions
            .iter()
            .any(|row| row.id == id && row.volume.percent == 37)
            .then_some(())
    })?;
    // SAFETY: volume belongs to the stopped fixture stream on this COM thread.
    assert!((unsafe { volume.GetMasterVolume()? } - 0.37).abs() < 0.01);
    for _ in 0..2 {
        mixer
            .adjust(id, Action::Step(5))
            .map_err(anyhow::Error::msg)?;
    }
    wait_for(&events, |snapshot| {
        snapshot
            .sessions
            .iter()
            .any(|row| row.id == id && row.volume.percent == 47)
            .then_some(())
    })?;

    mixer
        .adjust(id, Action::ToggleMute)
        .map_err(anyhow::Error::msg)?;
    wait_for(&events, |snapshot| {
        snapshot
            .sessions
            .iter()
            .any(|row| row.id == id && row.volume.muted)
            .then_some(())
    })?;
    // SAFETY: this simulates an external change only on the fixture stream.
    unsafe { volume.SetMute(false, std::ptr::null())? };
    wait_for(&events, |snapshot| {
        snapshot
            .sessions
            .iter()
            .any(|row| row.id == id && !row.volume.muted)
            .then_some(())
    })?;
    mixer
        .adjust(u64::MAX, Action::Set(0))
        .map_err(anyhow::Error::msg)?;
    wait_for(&events, |snapshot| snapshot.warning.as_ref().map(|_| ()))?;
    // SAFETY: the invalid command must not change the fixture's volume.
    assert!((unsafe { volume.GetMasterVolume()? } - 0.47).abs() < 0.01);
    Ok(())
}

fn wait_for<T>(
    events: &async_channel::Receiver<Snapshot>,
    accept: impl Fn(&Snapshot) -> Option<T>,
) -> anyhow::Result<T> {
    let deadline = Instant::now() + Duration::from_secs(4);
    while Instant::now() < deadline {
        match events.try_recv() {
            Ok(snapshot) => {
                if let Some(value) = accept(&snapshot) {
                    return Ok(value);
                }
            }
            Err(async_channel::TryRecvError::Closed) => anyhow::bail!("Mixer worker closed"),
            Err(async_channel::TryRecvError::Empty) => {
                std::thread::sleep(Duration::from_millis(15))
            }
        }
    }
    anyhow::bail!("Timed out waiting for an audio session snapshot")
}
