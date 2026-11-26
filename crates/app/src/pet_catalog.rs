//! Embedded pet catalog and color frame preparation.
use gpui::RenderImage;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum PetKind {
    Fox,
    Rabbit,
    #[default]
    #[serde(other)]
    Cat,
}

type Frames = HashMap<&'static str, Arc<RenderImage>>;

pub struct PetCatalog {
    frames: Frames,
}

macro_rules! package {
    ($id:literal) => {
        (
            include_str!(concat!("../../../assets/pets/", $id, "/manifest.json")),
            [
                frame!($id, "idle"),
                frame!($id, "blink"),
                frame!($id, "happy"),
                frame!($id, "happy-up"),
                frame!($id, "sleep"),
            ],
        )
    };
}

macro_rules! frame {
    ($id:literal, $frame:literal) => {
        (
            concat!("pets/", $id, "/", $frame, ".svg"),
            include_bytes!(concat!("../../../assets/pets/", $id, "/", $frame, ".svg")).as_slice(),
        )
    };
}

impl PetKind {
    pub const ALL: [Self; 3] = [Self::Cat, Self::Fox, Self::Rabbit];

    pub fn next(self) -> Self {
        match self {
            Self::Cat => Self::Fox,
            Self::Fox => Self::Rabbit,
            Self::Rabbit => Self::Cat,
        }
    }

    fn assets(self) -> (&'static str, [(&'static str, &'static [u8]); 5]) {
        match self {
            Self::Cat => package!("volt-cat"),
            Self::Fox => package!("ember-fox"),
            Self::Rabbit => package!("mint-rabbit"),
        }
    }

    pub fn manifest(self) -> &'static str {
        self.assets().0
    }
}

impl PetCatalog {
    pub fn load() -> Self {
        let mut frames = HashMap::new();
        for kind in PetKind::ALL {
            for (path, bytes) in kind.assets().1 {
                match rasterize(bytes) {
                    Ok(frame) => {
                        frames.insert(path, frame);
                    }
                    Err(error) => eprintln!("Не удалось подготовить кадр {path}: {error}"),
                }
            }
        }
        Self { frames }
    }

    pub fn frame(&self, path: &str) -> Option<Arc<RenderImage>> {
        self.frames.get(path).cloned()
    }
}

fn rasterize(bytes: &[u8]) -> anyhow::Result<Arc<RenderImage>> {
    let tree = resvg::usvg::Tree::from_data(bytes, &resvg::usvg::Options::default())?;
    let size = tree.size().to_int_size();
    let mut pixmap = resvg::tiny_skia::Pixmap::new(size.width(), size.height())
        .ok_or_else(|| anyhow::anyhow!("Недопустимый размер кадра"))?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::identity(),
        &mut pixmap.as_mut(),
    );

    // GPUI expects straight alpha with B and R swapped in the texture buffer.
    let mut pixels = Vec::with_capacity(pixmap.data().len());
    for color in pixmap.pixels().iter().map(|pixel| pixel.demultiply()) {
        pixels.extend([color.blue(), color.green(), color.red(), color.alpha()]);
    }
    let buffer = image::RgbaImage::from_raw(size.width(), size.height(), pixels)
        .ok_or_else(|| anyhow::anyhow!("Недопустимый буфер кадра"))?;
    let frames: smallvec::SmallVec<[_; 1]> = smallvec::smallvec![image::Frame::new(buffer)];
    Ok(Arc::new(RenderImage::new(frames)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use islandvolt_features::pets::{Animator, Package};

    #[test]
    fn all_animators_reach_five_distinct_visible_color_frames() {
        let catalog = PetCatalog::load();
        for kind in PetKind::ALL {
            let mut pet = Animator::new(
                Package::from_json(kind.manifest()).expect("manifest"),
                false,
            );
            let mut paths = std::collections::BTreeSet::new();
            pet.react();
            for _ in 0..64 {
                paths.insert(pet.frame().asset.clone());
                pet.advance();
            }
            assert_eq!(paths.len(), 5);
            let mut distinct = std::collections::BTreeSet::new();
            for path in paths {
                let frame = catalog.frame(&path).expect("embedded frame");
                let pixels = frame.as_bytes(0).expect("frame contents");
                assert!(
                    pixels
                        .as_chunks::<4>()
                        .0
                        .iter()
                        .any(|[b, _, r, a]| *a == 255 && r != b)
                );
                distinct.insert(pixels.to_vec());
            }
            assert_eq!(distinct.len(), 5);
        }
    }
}
