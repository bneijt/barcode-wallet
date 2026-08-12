use crate::model::Symbology;

/// Result of decoding an image.
pub struct Decoded {
    pub value: String,
    pub symbology: Symbology,
}

/// Decode a barcode from raw image bytes (PNG/JPEG).
pub fn decode_from_bytes(bytes: &[u8]) -> Result<Decoded, String> {
    let result = rxing::helpers::detect_in_buffer(bytes, None)
        .map_err(|_| "no barcode found in image".to_string())?;
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
            ))
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
