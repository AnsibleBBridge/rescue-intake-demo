//! 1080x1080 "adopt me" share image drawn on a canvas, in the fact sheet's visual family
//! (periwinkle bands, pale background, round photo with a name ribbon).
use crate::pages::fact_sheet::sil_path;
use rescue_core::model::*;
use rescue_core::posts::fact_lines;
use std::cell::Cell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement, Path2d};

pub const SIZE: u32 = 1080;
const BAND: &str = "#a9b8e3";
const RIBBON: &str = "#6f7fe0";
const FONT: &str = "system-ui, -apple-system, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif";

thread_local! {
    /// Bumped on every render so a slow photo load can't paint over a newer animal.
    static GENERATION: Cell<u32> = const { Cell::new(0) };
}

/// Download filename: lowercase name, spaces -> hyphens, only [a-z0-9-]; empty -> "pet".
pub fn download_name(name: &str) -> String {
    let slug: String = name
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c == ' ' { '-' } else { c })
        .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-')
        .collect();
    format!("{}-adopt-me.png", if slug.is_empty() { "pet" } else { &slug })
}

/// Up to three "Label: value" lines, preferring Breed, Age, Good with.
pub fn pick_facts(a: &Animal) -> Vec<String> {
    let lines = fact_lines(a);
    let mut chosen: Vec<(&str, &String)> = Vec::new();
    for want in ["Breed", "Age", "Good with"] {
        if let Some((k, v)) = lines.iter().find(|(k, _)| *k == want) {
            chosen.push((k, v));
        }
    }
    for (k, v) in &lines {
        if chosen.len() >= 3 {
            break;
        }
        if !chosen.iter().any(|(ck, _)| ck == k) {
            chosen.push((k, v));
        }
    }
    chosen.into_iter().take(3).map(|(k, v)| format!("{k}: {v}")).collect()
}

fn js<T: std::fmt::Debug>(e: T) -> String {
    format!("{e:?}")
}

/// Largest font size (<= start) at which `text` fits `max_w`.
fn fit_font(ctx: &CanvasRenderingContext2d, text: &str, weight: &str, start: f64, max_w: f64) -> Result<f64, String> {
    let mut size = start;
    while size > 14.0 {
        ctx.set_font(&format!("{weight} {size}px {FONT}"));
        if ctx.measure_text(text).map_err(js)?.width() <= max_w {
            break;
        }
        size -= 2.0;
    }
    ctx.set_font(&format!("{weight} {size}px {FONT}"));
    Ok(size)
}

fn centered(ctx: &CanvasRenderingContext2d, text: &str, weight: &str, size: f64, max_w: f64, y: f64) -> Result<(), String> {
    fit_font(ctx, text, weight, size, max_w)?;
    ctx.fill_text(text, SIZE as f64 / 2.0, y).map_err(js)
}

/// Rounded-pill path (arcs, so no reliance on newer roundRect support).
fn pill(ctx: &CanvasRenderingContext2d, x: f64, y: f64, w: f64, h: f64) -> Result<(), String> {
    let r = h / 2.0;
    ctx.begin_path();
    ctx.arc(x + r, y + r, r, std::f64::consts::FRAC_PI_2, 3.0 * std::f64::consts::FRAC_PI_2).map_err(js)?;
    ctx.arc(x + w - r, y + r, r, 3.0 * std::f64::consts::FRAC_PI_2, std::f64::consts::FRAC_PI_2).map_err(js)?;
    ctx.close_path();
    Ok(())
}

/// Paint the whole card; `photo` is a loaded image, or None to draw the species placeholder.
fn draw(ctx: &CanvasRenderingContext2d, a: &Animal, s: &Settings, photo: Option<&HtmlImageElement>) -> Result<(), String> {
    let w = SIZE as f64;
    let (cx, cy, r) = (w / 2.0, 450.0, 250.0);
    ctx.set_text_align("center");
    ctx.set_text_baseline("alphabetic");

    ctx.set_fill_style_str("#eef2fd");
    ctx.fill_rect(0.0, 0.0, w, w);

    // Title band
    ctx.set_fill_style_str(BAND);
    ctx.fill_rect(0.0, 0.0, w, 170.0);
    ctx.set_fill_style_str("#fff");
    centered(ctx, "ADOPT ME", "800", 84.0, 900.0, 105.0)?;
    centered(ctx, s.rescue_name.trim(), "600", 36.0, 900.0, 152.0)?;

    // White ring + clipped photo
    ctx.begin_path();
    ctx.arc(cx, cy, r + 16.0, 0.0, std::f64::consts::TAU).map_err(js)?;
    ctx.set_fill_style_str("#fff");
    ctx.fill();
    ctx.save();
    ctx.begin_path();
    ctx.arc(cx, cy, r, 0.0, std::f64::consts::TAU).map_err(js)?;
    ctx.clip();
    ctx.set_fill_style_str("#cfd8f3");
    ctx.fill_rect(cx - r, cy - r, 2.0 * r, 2.0 * r);
    match photo {
        Some(img) => {
            // Cover-crop to a square; sample SVGs are square with a viewBox only (bogus natural size), so they are drawn whole.
            let (nw, nh) = (img.natural_width() as f64, img.natural_height() as f64);
            let is_svg = img.current_src().ends_with(".svg");
            if nw > 0.0 && nh > 0.0 && !is_svg {
                let side = nw.min(nh);
                ctx.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                    img, (nw - side) / 2.0, (nh - side) / 2.0, side, side, cx - r, cy - r, 2.0 * r, 2.0 * r,
                ).map_err(js)?;
            } else {
                ctx.draw_image_with_html_image_element_and_dw_and_dh(img, cx - r, cy - r, 2.0 * r, 2.0 * r).map_err(js)?;
            }
        }
        None => {
            // Silhouette from the same path data the fact sheet uses (100x100 box, scaled to 60%).
            let path = Path2d::new_with_path_string(sil_path(a.species)).map_err(js)?;
            ctx.save();
            let k = 2.0 * r * 0.6 / 100.0;
            ctx.translate(cx - r * 0.6, cy - r * 0.6).map_err(js)?;
            ctx.scale(k, k).map_err(js)?;
            ctx.set_fill_style_str("#8f9fd4");
            ctx.fill_with_path_2d(&path);
            ctx.restore();
        }
    }
    ctx.restore();

    // Name ribbon overlapping the photo's bottom edge; long names shrink to fit.
    let name = a.name.trim();
    let name = if name.is_empty() { "Unnamed" } else { name };
    let size = fit_font(ctx, name, "700", 64.0, 600.0)?;
    let tw = ctx.measure_text(name).map_err(js)?.width();
    let (rw, rh) = (tw + 90.0, 96.0);
    pill(ctx, cx - rw / 2.0, cy + r - rh / 2.0, rw, rh)?;
    ctx.set_fill_style_str(RIBBON);
    ctx.fill();
    ctx.set_line_width(8.0);
    ctx.set_stroke_style_str("#fff");
    ctx.stroke();
    ctx.set_fill_style_str("#fff");
    ctx.set_font(&format!("700 {size}px {FONT}"));
    ctx.fill_text(name, cx, cy + r + size * 0.35).map_err(js)?;

    // Up to three fact lines
    ctx.set_fill_style_str("#3a4270");
    for (i, line) in pick_facts(a).iter().enumerate() {
        centered(ctx, line, "600", 40.0, 960.0, 810.0 + i as f64 * 56.0)?;
    }

    // Footer band
    ctx.set_fill_style_str(BAND);
    ctx.fill_rect(0.0, 960.0, w, 120.0);
    ctx.set_fill_style_str("#fff");
    // Rescue name, then phone (falling back to email; blank contact parts are skipped).
    let contact = if s.phone.trim().is_empty() { s.email.trim() } else { s.phone.trim() };
    if contact.is_empty() {
        centered(ctx, s.rescue_name.trim(), "700", 44.0, 960.0, 1034.0)?;
    } else {
        centered(ctx, s.rescue_name.trim(), "700", 34.0, 960.0, 1008.0)?;
        centered(ctx, contact, "700", 42.0, 960.0, 1056.0)?;
    }
    Ok(())
}

/// Render the share image. Sync errors are returned; photo-load paths report through `on_error`.
pub fn render_with(canvas: &HtmlCanvasElement, a: &Animal, s: &Settings, on_error: impl Fn(String) + 'static) -> Result<(), String> {
    canvas.set_width(SIZE);
    canvas.set_height(SIZE);
    let ctx: CanvasRenderingContext2d = canvas
        .get_context("2d").map_err(js)?.ok_or("Canvas is not supported in this browser")?
        .dyn_into().map_err(js)?;
    let gen = GENERATION.with(|g| { g.set(g.get().wrapping_add(1)); g.get() });
    let _ = canvas.remove_attribute("data-ready");

    let Some(src) = a.photo.clone() else {
        draw(&ctx, a, s, None)?;
        let _ = canvas.set_attribute("data-ready", "1");
        return Ok(());
    };
    // Same-origin SVG / data URL, so the canvas stays untainted and exportable.
    let img = HtmlImageElement::new().map_err(js)?;
    let (a, s) = (a.clone(), s.clone());
    let (canvas, ctx2, img2) = (canvas.clone(), ctx.clone(), img.clone());
    let finish = move |loaded: bool| {
        if GENERATION.with(|g| g.get()) != gen {
            return;
        }
        match draw(&ctx2, &a, &s, loaded.then_some(&img2)) {
            Ok(()) => { let _ = canvas.set_attribute("data-ready", "1"); }
            Err(e) => on_error(e),
        }
    };
    let finish = std::rc::Rc::new(finish);
    let f1 = finish.clone();
    let onload = Closure::once_into_js(move || f1(true));
    let onerror = Closure::once_into_js(move || finish(false));
    img.set_onload(Some(onload.unchecked_ref()));
    img.set_onerror(Some(onerror.unchecked_ref()));
    img.set_src(&src);
    Ok(())
}

/// Interface form of `render`; late photo failures are logged (the page uses `render_with`).
#[allow(dead_code)]
pub fn render(canvas: &HtmlCanvasElement, a: &Animal, s: &Settings) -> Result<(), String> {
    render_with(canvas, a, s, |e| leptos::logging::error!("share image: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_name_sanitizes() {
        assert_eq!(download_name("Biscuit"), "biscuit-adopt-me.png");
        assert_eq!(download_name("Mr. Whiskers Jr"), "mr-whiskers-jr-adopt-me.png");
        assert_eq!(download_name("  "), "pet-adopt-me.png");
        assert_eq!(download_name("Zoë!?"), "zo-adopt-me.png");
        assert_eq!(download_name("***"), "pet-adopt-me.png");
    }
}
