use super::Message;
use async_channel::Sender;
use windows::{
    Win32::Media::Audio::{
        AudioSessionDisconnectReason, AudioSessionState, IAudioSessionControl, IAudioSessionEvents,
        IAudioSessionEvents_Impl, IAudioSessionNotification, IAudioSessionNotification_Impl,
    },
    core::{BOOL, GUID, PCWSTR, Ref, Result, implement},
};

#[implement(IAudioSessionNotification)]
struct Created(Sender<Message>);

impl IAudioSessionNotification_Impl for Created_Impl {
    fn OnSessionCreated(&self, _: Ref<'_, IAudioSessionControl>) -> Result<()> {
        let _ = self.0.try_send(Message::Refresh);
        Ok(())
    }
}

pub(super) fn created(signal: Sender<Message>) -> IAudioSessionNotification {
    Created(signal).into()
}

#[implement(IAudioSessionEvents)]
struct Changed(Sender<Message>);

impl Changed_Impl {
    fn refresh(&self) -> Result<()> {
        let _ = self.0.try_send(Message::Refresh);
        Ok(())
    }
}

impl IAudioSessionEvents_Impl for Changed_Impl {
    fn OnDisplayNameChanged(&self, _: &PCWSTR, _: *const GUID) -> Result<()> {
        self.refresh()
    }
    fn OnIconPathChanged(&self, _: &PCWSTR, _: *const GUID) -> Result<()> {
        Ok(())
    }
    fn OnSimpleVolumeChanged(&self, _: f32, _: BOOL, _: *const GUID) -> Result<()> {
        self.refresh()
    }
    fn OnChannelVolumeChanged(&self, _: u32, _: *const f32, _: u32, _: *const GUID) -> Result<()> {
        Ok(())
    }
    fn OnGroupingParamChanged(&self, _: *const GUID, _: *const GUID) -> Result<()> {
        Ok(())
    }
    fn OnStateChanged(&self, _: AudioSessionState) -> Result<()> {
        self.refresh()
    }
    fn OnSessionDisconnected(&self, _: AudioSessionDisconnectReason) -> Result<()> {
        self.refresh()
    }
}

pub(super) fn changed(signal: Sender<Message>) -> IAudioSessionEvents {
    Changed(signal).into()
}
