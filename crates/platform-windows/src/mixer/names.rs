use windows::{
    Win32::{
        Devices::FunctionDiscovery::PKEY_Device_FriendlyName,
        Foundation::{CloseHandle, S_OK},
        Media::Audio::{IAudioSessionControl2, IMMDevice},
        System::{
            Com::{
                CoTaskMemFree, STGM_READ,
                StructuredStorage::{PropVariantClear, PropVariantToStringAlloc},
            },
            Threading::{
                OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
                QueryFullProcessImageNameW,
            },
        },
    },
    core::PWSTR,
};

pub(super) fn device_id(device: &IMMDevice) -> windows::core::Result<String> {
    // SAFETY: GetId returns a caller-owned string from a live COM endpoint.
    unsafe { take_com_string(device.GetId()?) }
}

pub(super) fn session_id(control: &IAudioSessionControl2) -> windows::core::Result<String> {
    // SAFETY: the returned string belongs to the caller and is freed by take_com_string.
    unsafe { take_com_string(control.GetSessionInstanceIdentifier()?) }
}

pub(super) fn device_name(device: &IMMDevice) -> Option<String> {
    // SAFETY: the property store is opened read-only. Both the variant and string are released.
    unsafe {
        let store = device.OpenPropertyStore(STGM_READ).ok()?;
        let mut value = store.GetValue(&PKEY_Device_FriendlyName).ok()?;
        let name = PropVariantToStringAlloc(&value);
        let _ = PropVariantClear(&mut value);
        take_com_string(name.ok()?)
            .ok()
            .filter(|name| !name.trim().is_empty())
    }
}

pub(super) fn session_name(control: &IAudioSessionControl2) -> String {
    // SAFETY: querying a live COM session; GetDisplayName transfers ownership of its string.
    let (display, pid) = unsafe {
        if control.IsSystemSoundsSession() == S_OK {
            return "Системные звуки".into();
        }
        let display = control
            .GetDisplayName()
            .ok()
            .and_then(|name| take_com_string(name).ok());
        (display, control.GetProcessId().ok())
    };
    if let Some(display) = display.filter(|name| !name.trim().is_empty() && !name.starts_with('@'))
    {
        return display.chars().take(128).collect();
    }
    pid.and_then(process_name)
        .or_else(|| pid.map(|pid| format!("Приложение {pid}")))
        .unwrap_or_else(|| "Аудиосессия".into())
}

unsafe fn take_com_string(value: PWSTR) -> windows::core::Result<String> {
    if value.is_null() {
        return Ok(String::new());
    }
    // SAFETY: caller supplies a non-dangling NUL-terminated CoTaskMem string.
    unsafe {
        let text = value.to_string();
        CoTaskMemFree(Some(value.0.cast()));
        Ok(text?)
    }
}

fn process_name(pid: u32) -> Option<String> {
    let mut path = [0u16; 2048];
    let mut length = path.len() as u32;
    // SAFETY: query only a live process handle and close it on both success and error.
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let result = QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            PWSTR(path.as_mut_ptr()),
            &mut length,
        );
        let _ = CloseHandle(process);
        result.ok()?;
    }
    let path = String::from_utf16_lossy(path.get(..length as usize)?);
    std::path::Path::new(&path)
        .file_stem()?
        .to_str()
        .map(str::to_owned)
}
