use crate::model::{Code, Symbology};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

/// Render a Code onto a canvas. Returns a human-readable error on failure.
pub fn render(code: &Code, canvas: &HtmlCanvasElement) -> Result<(), String> {
    let ctx = canvas
        .get_context("2d")
        .map_err(|_| "could not get 2d context".to_string())?
        .and_then(|c| c.dyn_into::<CanvasRenderingContext2d>().ok())
        .ok_or_else(|| "could not get 2d context".to_string())?;

    match code.symbology {
        Symbology::Code128 => render_code128(&code.value, canvas, &ctx),
        Symbology::Ean13 => render_ean13(&code.value, canvas, &ctx),
        Symbology::UpcA => render_upca(&code.value, canvas, &ctx),
        Symbology::QrCode => render_qr(&code.value, canvas, &ctx),
    }
}

/// Draw a 1D barcode from a sequence of module bits (1 = dark, 0 = light).
fn draw_modules(bits: &[u8], canvas: &HtmlCanvasElement, ctx: &CanvasRenderingContext2d) {
    let total = bits.len() as f64;
    const QUIET: f64 = 10.0; // modules of white quiet zone on each side
    let module_w = 4.0; // px per module
    let draw_w = (total + QUIET * 2.0) * module_w;
    let height = 240.0;

    canvas.set_width(draw_w as u32);
    canvas.set_height(height as u32);

    ctx.set_fill_style_str("#ffffff");
    ctx.fill_rect(0.0, 0.0, draw_w, height);

    ctx.set_fill_style_str("#000000");
    let mut x = QUIET * module_w;
    for &b in bits {
        if b == 1 {
            ctx.fill_rect(x, 0.0, module_w, height);
        }
        x += module_w;
    }
}

fn render_code128(value: &str, canvas: &HtmlCanvasElement, ctx: &CanvasRenderingContext2d) -> Result<(), String> {
    // barcoders requires an explicit start character; set B (Ɓ) handles ASCII.
    let data = format!("\u{0181}{value}");
    let code = barcoders::sym::code128::Code128::new(data)
        .map_err(|e| format!("Code 128 encode failed: {e:?}"))?;
    draw_modules(&code.encode(), canvas, ctx);
    Ok(())
}

fn render_ean13(value: &str, canvas: &HtmlCanvasElement, ctx: &CanvasRenderingContext2d) -> Result<(), String> {
    let code = barcoders::sym::ean13::EAN13::new(value)
        .map_err(|e| format!("EAN-13 encode failed: {e:?}"))?;
    draw_modules(&code.encode(), canvas, ctx);
    Ok(())
}

fn render_upca(value: &str, canvas: &HtmlCanvasElement, ctx: &CanvasRenderingContext2d) -> Result<(), String> {
    // UPC-A is EAN-13 with a leading zero.
    let data = format!("0{value}");
    let code = barcoders::sym::ean13::EAN13::new(data)
        .map_err(|e| format!("UPC-A encode failed: {e:?}"))?;
    draw_modules(&code.encode(), canvas, ctx);
    Ok(())
}

fn render_qr(value: &str, canvas: &HtmlCanvasElement, ctx: &CanvasRenderingContext2d) -> Result<(), String> {
    let code = qrcode::QrCode::new(value.as_bytes()).map_err(|e| format!("QR encode failed: {e}"))?;
    let width = code.width();
    let cells = code.to_colors();
    let scale = 10.0;
    let size = width as f64 * scale;
    let border = 40.0;

    canvas.set_width((size + border * 2.0) as u32);
    canvas.set_height((size + border * 2.0) as u32);

    ctx.set_fill_style_str("#ffffff");
    ctx.fill_rect(0.0, 0.0, size + border * 2.0, size + border * 2.0);
    ctx.set_fill_style_str("#000000");

    for y in 0..width {
        for x in 0..width {
            if cells[y * width + x] != qrcode::Color::Light {
                ctx.fill_rect(border + x as f64 * scale, border + y as f64 * scale, scale, scale);
            }
        }
    }
    Ok(())
}
