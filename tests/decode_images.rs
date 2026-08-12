use std::path::Path;

/// Expected decodes for the example images under `tmp/`.
/// Add more images here as we grow the set of barcodes we want to support.
const CASES: &[(&str, barcode_wallet::model::Symbology, &str)] = &[
    (
        "tests/resources/c128_1234567890.png",
        barcode_wallet::model::Symbology::Code128,
        "1234567890",
    ),
    (
        "tests/resources/AI01SSCUS.png",
        barcode_wallet::model::Symbology::Code128,
        "0101234567891231",
    ),
    (
        "tests/resources/BootsPrint.png",
        barcode_wallet::model::Symbology::Code128,
        "10123456789012",
    ),
    (
        "tests/resources/GS1-128Freeform.png",
        barcode_wallet::model::Symbology::Code128,
        "101234568020198798787",
    ),
    (
        "tests/resources/MSC128A.png",
        barcode_wallet::model::Symbology::Code128,
        "1234567890",
    ),
];

#[test]
fn decode_example_images() {
    for (path, expected_symbology, expected_value) in CASES {
        let path = Path::new(path);
        assert!(path.exists(), "missing example image: {}", path.display());

        let bytes = std::fs::read(path).expect("read image");
        let decoded = barcode_wallet::decode::decode_from_bytes(&bytes)
            .unwrap_or_else(|e| panic!("{}: failed to decode: {e}", path.display()));

        assert_eq!(
            decoded.symbology,
            *expected_symbology,
            "{}: symbology mismatch",
            path.display()
        );
        assert_eq!(
            decoded.value,
            *expected_value,
            "{}: value mismatch",
            path.display()
        );
    }
}
