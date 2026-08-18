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

/// A small, realistic wallet: one code of every supported symbology.
fn test_wallet() -> Vec<Code> {
    vec![
        Code {
            value: "1234567890".to_string(),
            symbology: Symbology::Code128,
            name: "Nero Cafe".to_string(),
            color: "#f5d76e".to_string(),
        },
        Code {
            value: "4006381333931".to_string(),
            symbology: Symbology::Ean13,
            name: "Member card".to_string(),
            color: "#70a1ff".to_string(),
        },
        Code {
            value: "036000291452".to_string(),
            symbology: Symbology::UpcA,
            name: "Tesco Clubcard".to_string(),
            color: "#2ed573".to_string(),
        },
        Code {
            value: "https://example.com/reward/abc123".to_string(),
            symbology: Symbology::QrCode,
            name: "Rewards QR".to_string(),
            color: "#a29bfe".to_string(),
        },
    ]
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

/// System test for the full export/import flow: a small wallet is exported to
/// JSON, imported into an empty store, verified field-for-field, then imported
/// again to confirm duplicates are skipped.
#[test]
fn export_import_roundtrip_full_wallet() {
    let wallet = test_wallet();
    assert_eq!(wallet.len(), 4);

    // 1. Export the wallet.
    let json = import_export::export_to_json(&wallet);

    // The envelope must contain every barcode, formatted as the app writes it.
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("export is valid JSON");
    assert_eq!(parsed["format"], "barcode-wallet.bneijt.nl");
    assert_eq!(parsed["version"], 1);
    assert_eq!(
        parsed["codes"].as_array().expect("codes array").len(),
        wallet.len(),
        "export must contain every code"
    );

    // 2. Import into an empty store.
    let (added, summary) = import_export::import_from_json(&json, &[]).expect("import succeeds");
    assert_eq!(added.len(), wallet.len(), "all codes are new");
    assert_eq!(summary.imported, 4);
    assert_eq!(summary.skipped, 0);
    assert_eq!(summary.rejected, 0);

    // 3. Every imported code equals the original, field for field.
    for original in &wallet {
        assert!(
            added.contains(original),
            "imported set must contain {original:?}"
        );
    }
    assert_eq!(
        added, wallet,
        "import must round-trip the wallet without losing data"
    );

    // 4. Import the same export again: everything is now a duplicate.
    let (added2, summary2) = import_export::import_from_json(&json, &wallet).expect("import ok");
    assert!(added2.is_empty(), "nothing may be added twice");
    assert_eq!(summary2.imported, 0);
    assert_eq!(summary2.skipped, 4);
    assert_eq!(summary2.rejected, 0);
}

/// Importing into a partially-populated store merges: new codes are added while
/// codes that already exist are skipped, matching the on-device behaviour.
#[test]
fn export_import_merges_into_existing_wallet() {
    let wallet = test_wallet();
    let json = import_export::export_to_json(&wallet);

    let already_have = vec![wallet[0].clone()];
    let (added, summary) =
        import_export::import_from_json(&json, &already_have).expect("import succeeds");

    assert_eq!(added.len(), 3, "only the codes we don't have are added");
    assert!(!added.contains(&wallet[0]), "existing code must be skipped");
    assert_eq!(summary.imported, 3);
    assert_eq!(summary.skipped, 1);
    assert_eq!(summary.rejected, 0);

    // The merged wallet contains everything, with no duplicates.
    let mut merged = already_have.clone();
    merged.extend(added);
    assert_eq!(merged.len(), wallet.len());
    for original in &wallet {
        assert!(merged.contains(original), "merged must contain {original:?}");
    }
}
