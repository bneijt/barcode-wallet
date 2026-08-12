use barcode_wallet::import_export;
use barcode_wallet::model::{Code, Symbology};

fn code(value: &str, name: &str) -> Code {
    Code {
        value: value.to_string(),
        symbology: Symbology::Code128,
        name: name.to_string(),
        color: "#f5d76e".to_string(),
    }
}

#[test]
fn export_wraps_codes_in_envelope() {
    let codes = vec![code("1234567890", "Nero")];
    let json = import_export::export_to_json(&codes);
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid json");
    assert_eq!(parsed["format"], "barcode-wallet.bneijt.nl");
    assert_eq!(parsed["version"], 1);
    assert_eq!(parsed["codes"][0]["symbology"], "Code128");
    assert_eq!(parsed["codes"][0]["value"], "1234567890");
}

#[test]
fn import_adds_new_codes_and_roundtrips() {
    let codes = vec![code("1234567890", "Nero")];
    let json = import_export::export_to_json(&codes);

    let (added, summary) =
        import_export::import_from_json(&json, &[]).expect("import succeeds");
    assert_eq!(added.len(), 1);
    assert_eq!(summary.imported, 1);
    assert_eq!(summary.skipped, 0);
    assert_eq!(summary.rejected, 0);
    assert_eq!(added[0].value, "1234567890");
}

#[test]
fn import_skips_duplicates() {
    let existing = vec![code("1234567890", "Nero")];
    let json = import_export::export_to_json(&[
        code("1234567890", "Nero"),
        code("999000111222", "Tesco"),
    ]);

    let (added, summary) =
        import_export::import_from_json(&json, &existing).expect("import succeeds");
    assert_eq!(added.len(), 1);
    assert_eq!(added[0].value, "999000111222");
    assert_eq!(summary.imported, 1);
    assert_eq!(summary.skipped, 1);
    assert_eq!(summary.rejected, 0);
}

#[test]
fn import_rejects_invalid_records_but_keeps_good_ones() {
    // A hand-authored file with one valid Code128 and one that fails validation.
    let json = r##"{
      "format": "barcode-wallet.bneijt.nl",
      "version": 1,
      "codes": [
        { "value": "1234567890", "symbology": "Code128", "name": "Nero", "color": "#f5d76e" },
        { "value": "", "symbology": "Code128", "name": "Empty", "color": "#fff" }
      ]
    }"##;

    let (added, summary) =
        import_export::import_from_json(json, &[]).expect("import succeeds");
    assert_eq!(added.len(), 1);
    assert_eq!(added[0].value, "1234567890");
    assert_eq!(summary.imported, 1);
    assert_eq!(summary.skipped, 0);
    assert_eq!(summary.rejected, 1);
}

#[test]
fn import_errors_on_bad_json() {
    let result = import_export::import_from_json("not json at all", &[]);
    assert!(result.is_err());
}
