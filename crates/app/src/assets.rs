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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_icons_are_embedded_and_available() {
        for path in [
            "icons/play.svg",
            "icons/pause.svg",
            "icons/previous.svg",
            "icons/next.svg",
        ] {
            let icon = Icons
                .load(path)
                .expect("icon lookup")
                .expect("embedded icon");
            assert!(icon.starts_with(b"<svg"), "invalid SVG: {path}");
        }
        assert!(Icons.load("icons/unknown.svg").expect("lookup").is_none());
    }
}
