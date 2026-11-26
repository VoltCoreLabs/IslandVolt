use crate::theme::Palette;
use gpui::{IntoElement, RenderImage, div, img, prelude::*, px, svg};
use islandvolt_platform_windows::media::Artwork;
use std::sync::Arc;

pub fn prepare(artwork: &Artwork) -> Option<Arc<RenderImage>> {
    let mut pixels = artwork.rgba.clone();
    for pixel in pixels.as_chunks_mut::<4>().0 {
        pixel.swap(0, 2);
    }
    let image = image::RgbaImage::from_raw(artwork.side, artwork.side, pixels)?;
    let frames: smallvec::SmallVec<[_; 1]> = smallvec::smallvec![image::Frame::new(image)];
    Some(Arc::new(RenderImage::new(frames)))
}

pub fn thumbnail(
    artwork: Option<&Arc<RenderImage>>,
    side: f32,
    palette: Palette,
) -> impl IntoElement {
    div()
        .size(px(side))
        .flex_shrink_0()
        .rounded(px(side * 0.2))
        .overflow_hidden()
        .flex()
        .items_center()
        .justify_center()
        .bg(palette.surface)
        .map(|thumbnail| match artwork {
            Some(image) => thumbnail.child(img(image.clone()).size_full()),
            None => thumbnail.child(
                svg()
                    .path("icons/waveform.svg")
                    .size(px(side * 0.5))
                    .text_color(palette.accent),
            ),
        })
}
