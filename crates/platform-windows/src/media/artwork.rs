use image::{ImageFormat, ImageReader, Limits, imageops::FilterType};
use std::{io::Cursor, sync::mpsc, time::Duration};
use windows::{
    Media::Control::GlobalSystemMediaTransportControlsSessionMediaProperties as Properties,
    Storage::Streams::{DataReader, IRandomAccessStreamReference},
};
use windows_future::{AsyncOperationCompletedHandler, IAsyncOperation};

const MAX_BYTES: u64 = 4 * 1024 * 1024;
const COVER_SIDE: u32 = 128;

/// A small square cover in RGBA format, sent to the renderer without COM handles.
#[derive(Debug, PartialEq, Eq)]
pub struct Artwork {
    pub side: u32,
    pub rgba: Vec<u8>,
}

pub(super) fn read(properties: &Properties) -> Option<Artwork> {
    read_reference(&properties.Thumbnail().ok()?)
}

fn read_reference(reference: &IRandomAccessStreamReference) -> Option<Artwork> {
    let operation = reference.OpenReadAsync().ok()?;
    let stream = wait(&operation)?;
    let size = stream.Size().ok()?;
    if !(1..=MAX_BYTES).contains(&size) {
        return None;
    }
    let reader = DataReader::CreateDataReader(&stream.GetInputStreamAt(0).ok()?).ok()?;
    let loaded = wait(&reader.LoadAsync(size as u32).ok()?)?;
    if u64::from(loaded) != size {
        return None;
    }
    let mut bytes = vec![0; loaded as usize];
    reader.ReadBytes(&mut bytes).ok()?;
    decode(&bytes)
}

fn wait<T: windows::core::RuntimeType>(operation: &IAsyncOperation<T>) -> Option<T> {
    let (tx, rx) = mpsc::sync_channel(1);
    operation
        .SetCompleted(&AsyncOperationCompletedHandler::new(move |_, _| {
            let _ = tx.try_send(());
            Ok(())
        }))
        .ok()?;
    if rx.recv_timeout(Duration::from_millis(350)).is_err() {
        let _ = operation.Cancel();
        return None;
    }
    operation.GetResults().ok()
}

fn decode(bytes: &[u8]) -> Option<Artwork> {
    let format = image::guess_format(bytes).ok()?;
    if !matches!(
        format,
        ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::WebP
    ) {
        return None;
    }
    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
    let mut limits = Limits::default();
    limits.max_image_width = Some(4096);
    limits.max_image_height = Some(4096);
    limits.max_alloc = Some(64 * 1024 * 1024);
    reader.limits(limits);
    let image = reader.decode().ok()?;
    Some(Artwork {
        side: COVER_SIDE,
        rgba: image
            .resize_to_fill(COVER_SIDE, COVER_SIDE, FilterType::Triangle)
            .into_rgba8()
            .into_raw(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageEncoder, codecs::png::PngEncoder};
    use windows::Storage::Streams::{
        DataWriter, InMemoryRandomAccessStream, RandomAccessStreamReference,
    };

    #[test]
    fn reads_a_thumbnail_through_native_winrt_streams() -> windows::core::Result<()> {
        let stream = InMemoryRandomAccessStream::new()?;
        let writer = DataWriter::CreateDataWriter(&stream.GetOutputStreamAt(0)?)?;
        let mut bytes = Vec::new();
        PngEncoder::new(&mut bytes)
            .write_image(&[17, 29, 54, 255], 1, 1, image::ExtendedColorType::Rgba8)
            .expect("PNG fixture");
        writer.WriteBytes(&bytes)?;
        assert_eq!(wait(&writer.StoreAsync()?), Some(bytes.len() as u32));
        let reference = RandomAccessStreamReference::CreateFromStream(&stream)?;
        let cover = read_reference((&reference).into()).expect("WinRT thumbnail");
        assert_eq!(&cover.rgba[..4], &[17, 29, 54, 255]);
        Ok(())
    }

    #[test]
    fn valid_thumbnail_is_scaled_and_corrupt_or_oversized_sources_are_rejected() {
        let mut bytes = Vec::new();
        PngEncoder::new(&mut bytes)
            .write_image(
                &[17, 29, 54, 255].repeat(24),
                6,
                4,
                image::ExtendedColorType::Rgba8,
            )
            .expect("PNG fixture");
        let cover = decode(&bytes).expect("decoded cover");
        assert_eq!(cover.side, COVER_SIDE);
        assert_eq!(cover.rgba.len(), (COVER_SIDE * COVER_SIDE * 4) as usize);
        assert_eq!(&cover.rgba[..4], &[17, 29, 54, 255]);
        assert!(decode(&bytes[..20]).is_none());
        let mut wide = Vec::new();
        PngEncoder::new(&mut wide)
            .write_image(&[0; 4097], 4097, 1, image::ExtendedColorType::L8)
            .expect("oversized fixture");
        assert!(decode(&wide).is_none());
    }
}
