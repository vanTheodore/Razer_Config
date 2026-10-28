// Generate a deterministic application icon at build time and embed it in Windows PE
// resources (Explorer/taskbar), without depending on external images or icon fonts.
use std::{error::Error, fs, path::PathBuf};

const SIZE: u32 = 64;

fn color(x: u32, y: u32) -> [u8; 4] {
    let xf = x as f32;
    let yf = y as f32;
    let body = ((xf - 32.0) / 21.0).powi(2) + ((yf - 33.0) / 27.0).powi(2) <= 1.0;
    let wheel = (28.0..=36.0).contains(&xf) && (15.0..=27.0).contains(&yf);
    let seam = (31.0..=33.0).contains(&xf) && (11.0..=35.0).contains(&yf);
    if wheel { [15, 24, 32, 255] }
    else if seam { [88, 174, 162, 255] }
    else if body { [100, 224, 196, 255] }
    else { [0, 0, 0, 0] }
}

fn ico_bytes() -> Vec<u8> {
    let side = SIZE as usize;
    let mask_stride = side.div_ceil(32) * 4;
    let dib_len = 40 + side * side * 4 + side * mask_stride;
    let mut output = Vec::with_capacity(22 + dib_len);
    output.extend_from_slice(&[0, 0, 1, 0, 1, 0]); // ICONDIR with 1 image
    output.extend_from_slice(&[SIZE as u8, SIZE as u8, 0, 0]);
    output.extend_from_slice(&1u16.to_le_bytes()); // planes
    output.extend_from_slice(&32u16.to_le_bytes()); // bpp
    output.extend_from_slice(&(dib_len as u32).to_le_bytes());
    output.extend_from_slice(&22u32.to_le_bytes());
    output.extend_from_slice(&40u32.to_le_bytes()); // BITMAPINFOHEADER
    output.extend_from_slice(&SIZE.to_le_bytes());
    output.extend_from_slice(&(SIZE * 2).to_le_bytes()); // XOR bitmap + AND mask
    output.extend_from_slice(&1u16.to_le_bytes());
    output.extend_from_slice(&32u16.to_le_bytes());
    output.extend_from_slice(&0u32.to_le_bytes()); // BI_RGB
    output.extend_from_slice(&((side * side * 4) as u32).to_le_bytes());
    output.extend_from_slice(&[0u8; 16]); // pels/m and palette
    for y in (0..SIZE).rev() {
        for x in 0..SIZE {
            let [r, g, b, a] = color(x, y);
            output.extend_from_slice(&[b, g, r, a]); // DIB is BGRA, bottom-up
        }
    }
    for y in (0..SIZE).rev() {
        for group in 0..mask_stride {
            let mut bits = 0u8;
            for bit in 0..8 {
                let x = group * 8 + bit;
                if x < side && color(x as u32, y)[3] == 0 {
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
    let out = PathBuf::from(std::env::var("OUT_DIR")?).join("cloncontrol.ico");
    fs::write(&out, ico_bytes())?;
    let mut resource = winresource::WindowsResource::new();
    resource.set_icon(out.to_str().ok_or("Icon path not UTF-8")?);
    resource.set("ProductName", "CLON//CONTROL");
    resource.set("FileDescription", "Instant A704F mouse configurator");
    resource.compile()?;
    Ok(())
}
