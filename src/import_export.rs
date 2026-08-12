use crate::model::{Code, ExportFile, ImportSummary, EXPORT_FORMAT, EXPORT_VERSION};

/// Serialize the Code collection into the export JSON envelope.
pub fn export_to_json(codes: &[Code]) -> String {
    let export = ExportFile {
        format: EXPORT_FORMAT.to_string(),
        version: EXPORT_VERSION,
        codes: codes.to_vec(),
    };
    serde_json::to_string_pretty(&export).unwrap_or_else(|_| "{}".to_string())
}

/// Parse an export file and apply it against an existing set of Codes.
///
/// Records are validated individually: valid new Codes are added, records whose
/// `(symbology, value)` already exists in `existing` are skipped as duplicates,
/// and records that fail to parse or validate are rejected. A summary of the
/// counts is returned.
pub fn import_from_json(
    text: &str,
    existing: &[Code],
) -> Result<(Vec<Code>, ImportSummary), String> {
    let export: ExportFile =
        serde_json::from_str(text).map_err(|e| format!("not a valid export file: {e}"))?;

    let mut existing_keys: Vec<(String, String)> = existing
        .iter()
        .map(|c| (c.symbology_serde(), c.value.clone()))
        .collect();

    let mut summary = ImportSummary::default();
    let mut added = Vec::new();

    for code in export.codes {
        if code.validate().is_err() {
            summary.rejected += 1;
            continue;
        }
        let key = (code.symbology_serde(), code.value.clone());
        if existing_keys.contains(&key) {
            summary.skipped += 1;
            continue;
        }
        existing_keys.push(key);
        summary.imported += 1;
        added.push(code);
    }

    Ok((added, summary))
}
