use serde::{Deserialize, Serialize};

pub const EXPORT_FORMAT: &str = "barcode-wallet.bneijt.nl";
pub const EXPORT_VERSION: u32 = 1;

/// The JSON envelope wrapping an Export of the Code collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportFile {
    pub format: String,
    pub version: u32,
    pub codes: Vec<Code>,
}

/// Outcome of an Import.
#[derive(Debug, Clone, Default)]
pub struct ImportSummary {
    pub imported: usize,
    pub skipped: usize,
    pub rejected: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Symbology {
    Code128,
    Ean13,
    UpcA,
    QrCode,
    // Pdf417 is deferred: the encoder crate needs nightly Rust.
    // Pdf417,
}

impl Symbology {
    pub fn display_name(&self) -> &'static str {
        match self {
            Symbology::Code128 => "Code 128",
            Symbology::Ean13 => "EAN-13",
            Symbology::UpcA => "UPC-A",
            Symbology::QrCode => "QR",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Code {
    pub value: String,
    pub symbology: Symbology,
    pub name: String,
    pub color: String,
}

impl Code {
    /// Stable identity: symbology + value. Name and color are mutable metadata.
    pub fn key(&self) -> (Symbology, String) {
        (self.symbology, self.value.clone())
    }

    /// The serde spelling of the symbology (the Rust variant name), used in
    /// export JSON and for duplicate matching.
    pub fn symbology_serde(&self) -> String {
        serde_json::to_string(&self.symbology)
            .unwrap_or_default()
            .trim_matches('"')
            .to_string()
    }

    /// Validate that a value is encodable for its symbology.
    pub fn validate(&self) -> Result<(), String> {
        match self.symbology {
            Symbology::Code128 => {
                if self.value.is_empty() {
                    return Err("value must not be empty".into());
                }
                Ok(())
            }
            Symbology::Ean13 => {
                let digits = self.value.chars().filter(|c| c.is_ascii_digit()).count();
                if !(12..=13).contains(&digits) {
                    return Err("EAN-13 requires 12 digits (or 13 with check digit)".into());
                }
                Ok(())
            }
            Symbology::UpcA => {
                let digits = self.value.chars().filter(|c| c.is_ascii_digit()).count();
                if !(11..=12).contains(&digits) {
                    return Err("UPC-A requires 11 digits (or 12 with check digit)".into());
                }
                Ok(())
            }
            Symbology::QrCode => {
                if self.value.is_empty() {
                    return Err("value must not be empty".into());
                }
                Ok(())
            }
        }
    }
}

pub fn ordinal_word(n: usize) -> String {
    let suffix = match n % 100 {
        11 | 12 | 13 => "th",
        _ => match n % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        },
    };
    format!("{n}{suffix}")
}
