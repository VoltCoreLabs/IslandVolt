use super::Message;
use async_channel::Sender;
use windows::{
    Win32::{
        Foundation::PROPERTYKEY,
        Media::Audio::{
            AUDIO_VOLUME_NOTIFICATION_DATA, DEVICE_STATE, EDataFlow, ERole,
            Endpoints::{IAudioEndpointVolumeCallback, IAudioEndpointVolumeCallback_Impl},
            IMMNotificationClient, IMMNotificationClient_Impl, eConsole, eRender,
        },
    },
    core::{PCWSTR, implement},
};

#[implement(IAudioEndpointVolumeCallback)]
struct VolumeCallback(Sender<Message>);

impl IAudioEndpointVolumeCallback_Impl for VolumeCallback_Impl {
    fn OnNotify(&self, _data: *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> windows::core::Result<()> {
        let _ = self.0.try_send(Message::Refresh);
        Ok(())
    }
}

pub(super) fn volume_callback(sender: Sender<Message>) -> IAudioEndpointVolumeCallback {
    VolumeCallback(sender).into()
}

#[implement(IMMNotificationClient)]
struct DeviceCallback(Box<dyn Fn() + Send + Sync>);

impl IMMNotificationClient_Impl for DeviceCallback_Impl {
    fn OnDeviceStateChanged(&self, _: &PCWSTR, _: DEVICE_STATE) -> windows::core::Result<()> {
        Ok(())
    }
    fn OnDeviceAdded(&self, _: &PCWSTR) -> windows::core::Result<()> {
        Ok(())
    }
    fn OnDeviceRemoved(&self, _: &PCWSTR) -> windows::core::Result<()> {
        Ok(())
    }
    fn OnPropertyValueChanged(&self, _: &PCWSTR, _: &PROPERTYKEY) -> windows::core::Result<()> {
        Ok(())
    }
    fn OnDefaultDeviceChanged(
        &self,
        flow: EDataFlow,
        role: ERole,
        _: &PCWSTR,
    ) -> windows::core::Result<()> {
        if flow == eRender && role == eConsole {
            (self.0)();
        }
        Ok(())
    }
}

pub(crate) fn device_callback(notify: impl Fn() + Send + Sync + 'static) -> IMMNotificationClient {
    DeviceCallback(Box::new(notify)).into()
}
