// CLON//CONTROL identity glyph for Windows (Explorer and taskbar).
// Self-contained deterministic icon: no icon fonts, network downloads or build assets.
use std::{error::Error, fs, path::PathBuf};

const SIZE: u32 = 64;
const AA: u32 = 4;

fn inside_rounded(x: f32, y: f32) -> bool {
    let dx = (x - 32.0).abs() - 23.0;
    let dy = (y - 32.0).abs() - 23.0;
    let ox = dx.max(0.0);
    let oy = dy.max(0.0);
    ox * ox + oy * oy < 12.0 * 12.0
}

fn line_dist(x: f32, y: f32, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let vx = bx - ax;
    let vy = by - ay;
    let t = (((x - ax) * vx + (y - ay) * vy) / (vx * vx + vy * vy)).clamp(0.0, 1.0);
    ((x - ax - t * vx).powi(2) + (y - ay - t * vy).powi(2)).sqrt()
}

fn sample(x: f32, y: f32) -> [f32; 4] {
    if !inside_rounded(x, y) { return [0.0, 0.0, 0.0, 0.0]; }
    let bg = [12.0, 24.0, 37.0, 255.0];
    let mouse = ((x - 31.0) / 17.0).powi(2) + ((y - 29.5) / 23.0).powi(2);
    let mouse_outline = (mouse.sqrt() - 1.0).abs() < 0.055 && y < 49.5;
    let split = line_dist(x, y, 31.0, 8.0, 31.0, 27.0) < 1.35;
    let wheel = line_dist(x, y, 31.0, 12.0, 31.0, 20.0) < 2.1;
    // Two azure cut-lines define the CLON//CONTROL slash mark.
    let stripe1 = line_dist(x, y, 18.0, 51.0, 49.0, 22.0) < 3.7;
    let stripe2 = line_dist(x, y, 30.0, 52.0, 50.0, 33.0) < 3.4;
    let pale = [231.0, 242.0, 251.0, 255.0];
    let blue = [24.0, 169.0, 252.0, 255.0];
    if stripe1 || stripe2 || wheel { blue }
    else if mouse_outline || split { pale }
    else { bg }
}

fn rgba_pixels() -> Vec<u8> {
    let mut pixels = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let mut sum = [0.0_f32; 4];
            for sy in 0..AA {
                for sx in 0..AA {
                    let v = sample(x as f32 + (sx as f32 + 0.5) / AA as f32,
                        y as f32 + (sy as f32 + 0.5) / AA as f32);
                    for i in 0..4 { sum[i] += v[i]; }
                }
            }
            for v in sum { pixels.push((v / (AA * AA) as f32).round() as u8); }
        }
    }
    pixels
}

fn ico_bytes(rgba: &[u8]) -> Vec<u8> {
    let side = SIZE as usize;
    let mask_stride = side.div_ceil(32) * 4;
    let dib_len = 40 + side * side * 4 + side * mask_stride;
    let mut output = Vec::with_capacity(22 + dib_len);
    output.extend_from_slice(&[0, 0, 1, 0, 1, 0]);
    output.extend_from_slice(&[SIZE as u8, SIZE as u8, 0, 0]);
    output.extend_from_slice(&1u16.to_le_bytes());
    output.extend_from_slice(&32u16.to_le_bytes());
    output.extend_from_slice(&(dib_len as u32).to_le_bytes());
    output.extend_from_slice(&22u32.to_le_bytes());
    output.extend_from_slice(&40u32.to_le_bytes());
    output.extend_from_slice(&SIZE.to_le_bytes());
    output.extend_from_slice(&(SIZE * 2).to_le_bytes());
    output.extend_from_slice(&1u16.to_le_bytes());
    output.extend_from_slice(&32u16.to_le_bytes());
    output.extend_from_slice(&0u32.to_le_bytes());
    output.extend_from_slice(&((side * side * 4) as u32).to_le_bytes());
    output.extend_from_slice(&[0u8; 16]);
    for y in (0..SIZE).rev() {
        for x in 0..SIZE {
            let i = ((y * SIZE + x) * 4) as usize;
            output.extend_from_slice(&[rgba[i + 2], rgba[i + 1], rgba[i], rgba[i + 3]]);
        }
    }
    for y in (0..SIZE).rev() {
        for group in 0..mask_stride {
            let mut bits = 0u8;
            for bit in 0..8 {
                let x = group * 8 + bit;
                if x < side && rgba[((y as usize * side + x) * 4) + 3] == 0 {
                    bits |= 1 << (7 - bit);
                }
            }
            output.push(bits);
        }
    }
    debug_assert_eq!(output.len(), 22 + dib_len);
    output
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return Ok(());
    }
    let outdir = PathBuf::from(std::env::var("OUT_DIR")?);
    let rgba = rgba_pixels();
    fs::write(outdir.join("cloncontrol.rgba"), &rgba)?;
    let ico = outdir.join("cloncontrol.ico");
    fs::write(&ico, ico_bytes(&rgba))?;
    let mut resource = winresource::WindowsResource::new();
    resource.set_icon(ico.to_str().ok_or("Icon path not UTF-8")?);
    resource.set("ProductName", "CLON//CONTROL");
    resource.set("FileDescription", "Configurador de DPI para Instant A704F");
    resource.compile()?;
    Ok(())
}
