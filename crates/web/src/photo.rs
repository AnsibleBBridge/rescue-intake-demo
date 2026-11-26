//! Photo downscaling: phone photos are several MB, localStorage is ~5 MB total,
//! so every upload is shrunk to a JPEG data URL before it is stored.
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageBitmap};

/// User-facing text when the browser can't decode the file.
pub const DECODE_ERROR: &str = "Couldn't read that photo — try a JPEG or PNG";

/// Fit (w, h) inside a max x max box, never upscaling. Pure so it's host-testable.
pub fn fit(w: u32, h: u32, max: u32) -> (u32, u32) {
    let longest = w.max(h);
    if longest <= max || longest == 0 {
        return (w.max(1), h.max(1));
    }
    let scale = max as f64 / longest as f64;
    (((w as f64 * scale).round() as u32).max(1), ((h as f64 * scale).round() as u32).max(1))
}

/// Any failure (bad bytes, wrong cast) maps to the one user-facing message.
fn de<T>(_: T) -> String {
    DECODE_ERROR.to_string()
}

/// Decode `file`, shrink so the longest side is at most `max`, return a JPEG data URL.
pub async fn downscale(file: web_sys::File, max: u32, quality: f64) -> Result<String, String> {
    let win = web_sys::window().ok_or_else(|| DECODE_ERROR.to_string())?;
    // createImageBitmap honours EXIF rotation and rejects non-images.
    let promise = win.create_image_bitmap_with_blob(&file).map_err(de)?;
    let bmp: ImageBitmap = JsFuture::from(promise).await.map_err(de)?.dyn_into().map_err(de)?;
    let (tw, th) = fit(bmp.width(), bmp.height(), max);

    let canvas: HtmlCanvasElement = win
        .document()
        .ok_or_else(|| DECODE_ERROR.to_string())?
        .create_element("canvas")
        .map_err(de)?
        .dyn_into()
        .map_err(de)?;
    canvas.set_width(tw);
    canvas.set_height(th);
    let ctx: CanvasRenderingContext2d = canvas
        .get_context("2d")
        .map_err(de)?
        .ok_or_else(|| DECODE_ERROR.to_string())?
        .dyn_into()
        .map_err(de)?;
    // White backdrop so transparent PNGs don't turn black in JPEG.
    ctx.set_fill_style_str("#ffffff");
    ctx.fill_rect(0.0, 0.0, tw as f64, th as f64);
    ctx.draw_image_with_image_bitmap_and_dw_and_dh(&bmp, 0.0, 0.0, tw as f64, th as f64).map_err(de)?;
    bmp.close();
    canvas
        .to_data_url_with_type_and_encoder_options("image/jpeg", &JsValue::from_f64(quality))
        .map_err(de)
}

#[cfg(test)]
mod tests {
    use super::fit;

    #[test]
    fn fit_shrinks_longest_side_and_keeps_aspect() {
        assert_eq!(fit(4000, 3000, 1200), (1200, 900));
        assert_eq!(fit(3000, 4000, 1200), (900, 1200));
    }

    #[test]
    fn fit_never_upscales() {
        assert_eq!(fit(800, 600, 1200), (800, 600));
    }
}
