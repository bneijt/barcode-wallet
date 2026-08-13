use crate::model::Symbology;

/// Longest side, in pixels, that a frame is scaled down to before decoding.
///
/// Callers that capture their own frames (the live camera path) should draw at
/// this size so the decoder never has to rescale, and so that the pixels the
/// decoder sees are the pixels that were captured.
pub const MAX_SIDE: u32 = 640;

/// Result of decoding an image.
pub struct Decoded {
    pub value: String,
    pub symbology: Symbology,
}

/// Decode a barcode from raw image bytes (PNG/JPEG).
///
/// Tries the plain reader first, then the filtered reader, which rescales and
/// re-binarizes the image. The filtered pass rescues tightly printed barcodes
/// whose narrowest bar is about one pixel wide: at that density the binarizer
/// cannot place an edge reliably and the plain reader finds nothing.
pub fn decode_from_bytes(bytes: &[u8]) -> Result<Decoded, String> {
    let img =
        image::load_from_memory(bytes).map_err(|_| "could not read this image file".to_string())?;

    if let Ok(result) = rxing::helpers::detect_in_image_with_hints(
        img.clone(),
        None,
        &mut rxing::DecodeHints::default(),
    ) {
        return decoded_from_result(&result);
    }

    let result = rxing::helpers::detect_in_image_filtered_with_hints(
        img,
        None,
        &mut rxing::DecodeHints::default(),
    )
    .map_err(|_| "no barcode found in image".to_string())?;
    decoded_from_result(&result)
}

/// Decode a barcode from raw RGBA pixels (e.g. a canvas ImageData buffer).
/// Avoids a PNG encode/decode round-trip when capturing live camera frames.
///
/// Large frames are downscaled (longest side capped at [`MAX_SIDE`]) and the
/// image is attempted upright and rotated a quarter turn, since 1D symbologies
/// are not rotated automatically by the decoder and phone camera frames are
/// often sensor-rotated.
///
/// Only two orientations are tried: a 180° turn leaves a 1D barcode horizontal
/// and the decoder already retries every row reversed, so 180° and 270° are
/// redundant with the 0° and 90° attempts respectively.
pub fn decode_from_rgba(width: u32, height: u32, rgba: &[u8]) -> Result<Decoded, String> {
    let img = image::RgbaImage::from_raw(width, height, rgba.to_vec())
        .ok_or_else(|| "invalid image dimensions".to_string())?;
    let mut img = image::DynamicImage::ImageRgba8(img);

    let longest = width.max(height);
    if longest > MAX_SIDE {
        let scale = MAX_SIDE as f32 / longest as f32;
        let nw = (width as f32 * scale).round() as u32;
        let nh = (height as f32 * scale).round() as u32;
        img = img.thumbnail(nw, nh);
    }

    for attempt in 0..2 {
        let candidate = match attempt {
            1 => img.rotate90(),
            _ => img.clone(),
        };
        if let Ok(result) = rxing::helpers::detect_in_image_with_hints(
            candidate,
            None,
            &mut rxing::DecodeHints::default(),
        ) {
            return decoded_from_result(&result);
        }
    }

    Err("no barcode found in image".to_string())
}

fn decoded_from_result(result: &rxing::RXingResult) -> Result<Decoded, String> {
    let text = result.getText().to_string();
    let format = *result.getBarcodeFormat();

    let symbology = match format {
        rxing::BarcodeFormat::CODE_128 => Symbology::Code128,
        rxing::BarcodeFormat::EAN_13 => Symbology::Ean13,
        rxing::BarcodeFormat::UPC_A => Symbology::UpcA,
        rxing::BarcodeFormat::QR_CODE => Symbology::QrCode,
        other => {
            return Err(format!(
                "this barcode type is not supported yet ({})",
                format_name(other)
            ));
        }
    };

    Ok(Decoded {
        value: text,
        symbology,
    })
}

fn format_name(f: rxing::BarcodeFormat) -> &'static str {
    match f {
        rxing::BarcodeFormat::PDF_417 => "PDF417",
        rxing::BarcodeFormat::DATA_MATRIX => "Data Matrix",
        rxing::BarcodeFormat::AZTEC => "Aztec",
        rxing::BarcodeFormat::MAXICODE => "MaxiCode",
        rxing::BarcodeFormat::CODABAR => "Codabar",
        rxing::BarcodeFormat::CODE_39 => "Code 39",
        rxing::BarcodeFormat::CODE_93 => "Code 93",
        rxing::BarcodeFormat::ITF => "ITF",
        rxing::BarcodeFormat::EAN_8 => "EAN-8",
        rxing::BarcodeFormat::UPC_E => "UPC-E",
        _ => "unknown",
    }
}
