use std::fs;
use std::io::Cursor;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, ImageFormat, RgbImage, RgbaImage};
use img_parts::{Bytes, ImageEXIF};
use libheif_rs::{ColorSpace, HeifContext, LibHeif, RgbChroma};
use log::debug;

use crate::cli::Format;

/// Sizes Pillow writes when saving an ICO file, largest first.
const ICO_SIZES: [u32; 7] = [256, 128, 64, 48, 32, 24, 16];

pub struct HeifImage {
    pub image: DynamicImage,
    pub exif: Option<Vec<u8>>,
}

/// Decodes the primary image of a HEIF file, together with its EXIF metadata.
pub fn read_heif(lib_heif: &LibHeif, path: &Path) -> Result<HeifImage> {
    let path_str = path
        .to_str()
        .ok_or_else(|| anyhow!("Input file '{}' is not valid UTF-8", path.display()))?;
    let context = HeifContext::read_from_file(path_str)
        .with_context(|| format!("cannot identify image file '{}'", path.display()))?;
    let handle = context
        .primary_image_handle()
        .with_context(|| format!("cannot read primary image of '{}'", path.display()))?;

    let has_alpha = handle.has_alpha_channel();
    let chroma = if has_alpha {
        RgbChroma::Rgba
    } else {
        RgbChroma::Rgb
    };
    debug!(
        "Decoding {} as {}",
        path.display(),
        if has_alpha { "RGBA" } else { "RGB" }
    );
    let decoded = lib_heif
        .decode(&handle, ColorSpace::Rgb(chroma), None)
        .with_context(|| format!("cannot decode '{}'", path.display()))?;

    let planes = decoded.planes();
    let plane = planes
        .interleaved
        .ok_or_else(|| anyhow!("'{}' has no interleaved plane", path.display()))?;

    let width = plane.width;
    let height = plane.height;
    let channels = if has_alpha { 4 } else { 3 } as usize;
    let row_len = width as usize * channels;
    let mut pixels = Vec::with_capacity(row_len * height as usize);
    for row in 0..height as usize {
        let start = row * plane.stride;
        pixels.extend_from_slice(&plane.data[start..start + row_len]);
    }

    let image = if has_alpha {
        DynamicImage::ImageRgba8(
            RgbaImage::from_raw(width, height, pixels)
                .ok_or_else(|| anyhow!("unexpected pixel buffer size"))?,
        )
    } else {
        DynamicImage::ImageRgb8(
            RgbImage::from_raw(width, height, pixels)
                .ok_or_else(|| anyhow!("unexpected pixel buffer size"))?,
        )
    };

    Ok(HeifImage {
        image,
        exif: exif_of(&handle),
    })
}

/// Extracts the EXIF payload, dropping the four leading bytes that hold the
/// offset to the start of the TIFF header.
fn exif_of(handle: &libheif_rs::ImageHandle) -> Option<Vec<u8>> {
    handle
        .all_metadata()
        .into_iter()
        .find(|metadata| metadata.item_type.to_string() == "Exif")
        .and_then(|metadata| {
            let raw = metadata.raw_data;
            if raw.len() < 4 {
                return None;
            }
            let offset = u32::from_be_bytes([raw[0], raw[1], raw[2], raw[3]]) as usize;
            let start = 4usize.checked_add(offset)?;
            if start >= raw.len() {
                return None;
            }
            Some(raw[start..].to_vec())
        })
}

/// Encodes and writes the image, embedding the EXIF metadata when requested and
/// supported by the target format.
pub fn write_image(
    heif_image: &HeifImage,
    output: &Path,
    format: Format,
    quality: u8,
    keep_exif: bool,
) -> Result<()> {
    let encoded = encode(&heif_image.image, format, quality)?;
    let encoded = match (keep_exif, heif_image.exif.as_ref()) {
        (true, Some(exif)) => embed_exif(encoded, format, exif)?,
        _ => encoded,
    };
    fs::write(output, encoded).with_context(|| format!("cannot write '{}'", output.display()))?;
    Ok(())
}

fn encode(image: &DynamicImage, format: Format, quality: u8) -> Result<Vec<u8>> {
    match format {
        Format::Jpg => {
            let mut buffer = Vec::new();
            let rgb = DynamicImage::ImageRgb8(image.to_rgb8());
            JpegEncoder::new_with_quality(&mut buffer, quality.clamp(1, 100))
                .encode_image(&rgb)
                .context("cannot encode JPEG image")?;
            Ok(buffer)
        }
        Format::Ico => encode_ico(image),
        other => {
            let mut buffer = Cursor::new(Vec::new());
            image
                .write_to(&mut buffer, image_format(other))
                .with_context(|| format!("cannot encode {other} image"))?;
            Ok(buffer.into_inner())
        }
    }
}

/// Writes a multi resolution ICO file, using the same size list as Pillow.
fn encode_ico(image: &DynamicImage) -> Result<Vec<u8>> {
    let (width, height) = (image.width(), image.height());
    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);
    for size in ICO_SIZES
        .iter()
        .copied()
        .filter(|s| *s <= width && *s <= height)
    {
        let resized = image.resize_exact(size, size, image::imageops::FilterType::Lanczos3);
        let icon_image = ico::IconImage::from_rgba_data(size, size, resized.to_rgba8().into_raw());
        icon_dir
            .add_entry(ico::IconDirEntry::encode(&icon_image).context("cannot encode ICO entry")?);
    }
    if icon_dir.entries().is_empty() {
        let icon_image = ico::IconImage::from_rgba_data(width, height, image.to_rgba8().into_raw());
        icon_dir
            .add_entry(ico::IconDirEntry::encode(&icon_image).context("cannot encode ICO entry")?);
    }
    let mut buffer = Cursor::new(Vec::new());
    icon_dir
        .write(&mut buffer)
        .context("cannot write ICO image")?;
    Ok(buffer.into_inner())
}

fn image_format(format: Format) -> ImageFormat {
    match format {
        Format::Jpg => ImageFormat::Jpeg,
        Format::Png => ImageFormat::Png,
        Format::Webp => ImageFormat::WebP,
        Format::Gif => ImageFormat::Gif,
        Format::Tiff => ImageFormat::Tiff,
        Format::Bmp => ImageFormat::Bmp,
        Format::Ico => ImageFormat::Ico,
    }
}

/// Adds the EXIF metadata to the encoded image. GIF, BMP, ICO and TIFF are left
/// untouched, they carry no EXIF container we can splice the metadata into.
fn embed_exif(encoded: Vec<u8>, format: Format, exif: &[u8]) -> Result<Vec<u8>> {
    let bytes = Bytes::from(encoded);
    let exif = Bytes::copy_from_slice(exif);
    let mut output = Vec::new();
    match format {
        Format::Jpg => {
            let mut jpeg = img_parts::jpeg::Jpeg::from_bytes(bytes.clone())
                .context("cannot parse encoded JPEG image")?;
            jpeg.set_exif(Some(exif));
            jpeg.encoder()
                .write_to(&mut output)
                .context("cannot write EXIF metadata")?;
        }
        Format::Png => {
            let mut png = img_parts::png::Png::from_bytes(bytes.clone())
                .context("cannot parse encoded PNG image")?;
            png.set_exif(Some(exif));
            png.encoder()
                .write_to(&mut output)
                .context("cannot write EXIF metadata")?;
        }
        Format::Webp => {
            let mut webp = img_parts::webp::WebP::from_bytes(bytes.clone())
                .context("cannot parse encoded WebP image")?;
            webp.set_exif(Some(exif));
            webp.encoder()
                .write_to(&mut output)
                .context("cannot write EXIF metadata")?;
        }
        Format::Gif | Format::Tiff | Format::Bmp | Format::Ico => return Ok(bytes.to_vec()),
    }
    Ok(output)
}
