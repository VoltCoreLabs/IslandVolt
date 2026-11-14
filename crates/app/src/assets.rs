use std::borrow::Cow;

use gpui::{AssetSource, SharedString};

pub struct Icons;

impl AssetSource for Icons {
    fn load(&self, path: &str) -> anyhow::Result<Option<Cow<'static, [u8]>>> {
        let bytes: Option<&'static [u8]> = match path {
            "icons/play.svg" => Some(include_bytes!("../../../assets/icons/phosphor/play.svg")),
            "icons/pause.svg" => Some(include_bytes!("../../../assets/icons/phosphor/pause.svg")),
            "icons/previous.svg" => Some(include_bytes!(
                "../../../assets/icons/phosphor/skip-back.svg"
            )),
            "icons/next.svg" => Some(include_bytes!(
                "../../../assets/icons/phosphor/skip-forward.svg"
            )),
            _ => None,
        };
        Ok(bytes.map(Cow::Borrowed))
    }

    fn list(&self, _path: &str) -> anyhow::Result<Vec<SharedString>> {
        Ok(vec![])
    }
}
