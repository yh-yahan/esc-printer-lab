use crate::parser::command::{QrEcLevel, RasterImage, RasterScale};

const QUIET_ZONE_MODULES: usize = 4;
const DEFAULT_MODULE_SIZE: u8 = 3;

pub fn encode_qr_raster(data: &[u8], module_size: u8, level: QrEcLevel) -> Option<RasterImage> {
    if data.is_empty() {
        return None;
    }

    let ec = match level {
        QrEcLevel::L => qrcode::EcLevel::L,
        QrEcLevel::M => qrcode::EcLevel::M,
        QrEcLevel::Q => qrcode::EcLevel::Q,
        QrEcLevel::H => qrcode::EcLevel::H,
    };

    let code = qrcode::QrCode::with_error_correction_level(data, ec).ok()?;
    let modules = code.width();
    let scale = match module_size {
        1..=16 => module_size as usize,
        _ => DEFAULT_MODULE_SIZE as usize,
    };
    let dim = (modules + QUIET_ZONE_MODULES * 2) * scale;
    if dim == 0 {
        return None;
    }

    let width_bytes = dim.div_ceil(8);
    let mut bits = vec![0u8; width_bytes.saturating_mul(dim)];

    for module_y in 0..modules {
        for module_x in 0..modules {
            if code[(module_x, module_y)] != qrcode::Color::Dark {
                continue;
            }

            let x0 = (module_x + QUIET_ZONE_MODULES) * scale;
            let y0 = (module_y + QUIET_ZONE_MODULES) * scale;
            for dy in 0..scale {
                for dx in 0..scale {
                    set_dot(&mut bits, width_bytes, x0 + dx, y0 + dy);
                }
            }
        }
    }

    Some(RasterImage {
        scale: RasterScale::Normal,
        width_bytes: width_bytes as u16,
        height: dim as u16,
        data: bits,
    })
}

fn set_dot(bits: &mut [u8], width_bytes: usize, x: usize, y: usize) {
    let byte_index = y.saturating_mul(width_bytes) + (x / 8);
    if let Some(byte) = bits.get_mut(byte_index) {
        *byte |= 0x80 >> (x % 8);
    }
}
