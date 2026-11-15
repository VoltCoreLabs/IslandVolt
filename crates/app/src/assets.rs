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
            "icons/speaker.svg" => Some(include_bytes!(
                "../../../assets/icons/phosphor/speaker-high.svg"
            )),
            "icons/muted.svg" => Some(include_bytes!(
                "../../../assets/icons/phosphor/speaker-slash.svg"
            )),
            "icons/minus.svg" => Some(include_bytes!("../../../assets/icons/phosphor/minus.svg")),
            "icons/plus.svg" => Some(include_bytes!("../../../assets/icons/phosphor/plus.svg")),
            "icons/back.svg" => Some(include_bytes!(
                "../../../assets/icons/phosphor/arrow-left.svg"
            )),
            "pets/volt-cat/idle.svg" => {
                Some(include_bytes!("../../../assets/pets/volt-cat/idle.svg"))
            }
            "pets/volt-cat/blink.svg" => {
                Some(include_bytes!("../../../assets/pets/volt-cat/blink.svg"))
            }
            "pets/volt-cat/happy.svg" => {
                Some(include_bytes!("../../../assets/pets/volt-cat/happy.svg"))
            }
            "pets/volt-cat/happy-up.svg" => {
                Some(include_bytes!("../../../assets/pets/volt-cat/happy-up.svg"))
            }
            "pets/volt-cat/sleep.svg" => {
                Some(include_bytes!("../../../assets/pets/volt-cat/sleep.svg"))
            }
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
    fn builtin_pet_frames_exist_rasterize_and_show_different_states() {
        use islandvolt_features::pets::{Animator, Package};
        let package =
            Package::from_json(include_str!("../../../assets/pets/volt-cat/manifest.json"))
                .expect("built-in package");
        let mut pet = Animator::new(package, false);
        let mut frames = std::collections::BTreeMap::new();
        pet.react();
        for _ in 0..64 {
            let path = pet.frame().asset.clone();
            let data = Icons
                .load(&path)
                .expect("asset lookup")
                .expect("embedded frame");
            let tree = resvg::usvg::Tree::from_data(&data, &resvg::usvg::Options::default())
                .expect("valid SVG");
            let mut pixmap = resvg::tiny_skia::Pixmap::new(48, 48).expect("frame buffer");
            resvg::render(
                &tree,
                resvg::tiny_skia::Transform::identity(),
                &mut pixmap.as_mut(),
            );
            assert!(
                pixmap
                    .data()
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .any(|pixel| pixel[3] != 0)
            );
            frames.insert(path, pixmap.take());
            pet.advance();
        }
        assert_eq!(frames.len(), 5, "every manifest frame must be exercised");
        assert_ne!(
            frames["pets/volt-cat/idle.svg"],
            frames["pets/volt-cat/blink.svg"]
        );
        assert_ne!(
            frames["pets/volt-cat/happy.svg"],
            frames["pets/volt-cat/happy-up.svg"]
        );
        assert_ne!(
            frames["pets/volt-cat/idle.svg"],
            frames["pets/volt-cat/sleep.svg"]
        );
    }

    #[test]
    fn media_icons_are_embedded_and_available() {
        for path in [
            "icons/play.svg",
            "icons/pause.svg",
            "icons/previous.svg",
            "icons/next.svg",
            "icons/speaker.svg",
            "icons/muted.svg",
            "icons/minus.svg",
            "icons/plus.svg",
            "icons/back.svg",
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
