use barcode_wallet::model::Symbology;

/// Build an RGBA image of a Code128 barcode from its module bits.
fn code128_rgba(value: &str) -> (u32, u32, Vec<u8>) {
    let code = barcoders::sym::code128::Code128::new(format!("\u{0181}{value}")).unwrap();
    let bits = code.encode();
    let module_w = 6u32;
    let quiet = 10u32;
    let width = (bits.len() as u32) * module_w + quiet * 2;
    let height = 100u32;
    let mut rgba = vec![255u8; (width * height * 4) as usize];
    for (i, &b) in bits.iter().enumerate() {
        if b == 1 {
            let x0 = (quiet + i as u32 * module_w) as usize;
            for y in 0..height as usize {
                for x in x0..x0 + module_w as usize {
                    let p = (y * width as usize + x) * 4;
                    rgba[p] = 0;
                    rgba[p + 1] = 0;
                    rgba[p + 2] = 0;
                }
            }
        }
    }
    (width, height, rgba)
}

#[test]
fn decode_code128_from_raw_rgba() {
    let (w, h, rgba) = code128_rgba("1234567890");
    let decoded = barcode_wallet::decode::decode_from_rgba(w, h, &rgba)
        .expect("should decode from raw rgba");
    assert_eq!(decoded.symbology, Symbology::Code128);
    assert_eq!(decoded.value, "1234567890");
}

#[test]
fn decode_from_rgba_rejects_blank_image() {
    let w = 100u32;
    let h = 100u32;
    let rgba = vec![255u8; (w * h * 4) as usize];
    let result = barcode_wallet::decode::decode_from_rgba(w, h, &rgba);
    assert!(result.is_err(), "blank image should fail to decode");
}

/// Transpose an RGBA buffer (rotate 90°), simulating a sensor-rotated camera frame.
fn rotate90_rgba(width: u32, height: u32, rgba: &[u8]) -> (u32, u32, Vec<u8>) {
    let mut out = vec![255u8; rgba.len()];
    for y in 0..height {
        for x in 0..width {
            let src = ((y * width + x) * 4) as usize;
            let dst_x = height - 1 - y;
            let dst_y = x;
            let dst = ((dst_y * height + dst_x) * 4) as usize;
            out[dst..dst + 4].copy_from_slice(&rgba[src..src + 4]);
        }
    }
    (height, width, out)
}

#[test]
fn decode_code128_from_rotated_rgba() {
    let (w, h, rgba) = code128_rgba("1234567890");
    let (rw, rh, rrgba) = rotate90_rgba(w, h, &rgba);
    let decoded = barcode_wallet::decode::decode_from_rgba(rw, rh, &rrgba)
        .expect("should decode a 90-degree-rotated barcode");
    assert_eq!(decoded.symbology, Symbology::Code128);
    assert_eq!(decoded.value, "1234567890");
}
