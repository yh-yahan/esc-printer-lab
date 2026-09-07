use std::io::Write;

use flate2::write::ZlibEncoder;
use flate2::Compression;
use image::RgbaImage;

use super::ExportError;

pub fn encode(image: &RgbaImage, paper_width_mm: f32) -> Result<Vec<u8>, ExportError> {
    let width = image.width().max(1);
    let height = image.height().max(1);
    let mut rgb = Vec::with_capacity(width as usize * height as usize * 3);
    for pixel in image.pixels() {
        rgb.push(pixel[0]);
        rgb.push(pixel[1]);
        rgb.push(pixel[2]);
    }

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&rgb)?;
    let compressed = encoder.finish()?;

    let width_pt = paper_width_mm as f64 / 25.4 * 72.0;
    let height_pt = width_pt * (height as f64 / width as f64);

    let mut out = Vec::new();
    out.extend_from_slice(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n");

    let mut offsets = Vec::new();
    let mut write_obj = |out: &mut Vec<u8>, body: &[u8]| {
        offsets.push(out.len());
        out.extend_from_slice(body);
        if !body.ends_with(b"\n") {
            out.push(b'\n');
        }
    };

    write_obj(
        &mut out,
        b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n",
    );
    write_obj(
        &mut out,
        b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n",
    );

    let page = format!(
        "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {:.3} {:.3}] /Contents 4 0 R /Resources << /XObject << /Im0 5 0 R >> >> >>\nendobj\n",
        width_pt, height_pt
    );
    write_obj(&mut out, page.as_bytes());

    let content = format!("q {:.3} 0 0 {:.3} 0 0 cm /Im0 Do Q\n", width_pt, height_pt);
    let contents_obj = format!(
        "4 0 obj\n<< /Length {} >>\nstream\n{}endstream\nendobj\n",
        content.len(),
        content
    );
    write_obj(&mut out, contents_obj.as_bytes());

    offsets.push(out.len());
    let header = format!(
        "5 0 obj\n<< /Type /XObject /Subtype /Image /Width {width} /Height {height} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /FlateDecode /Length {} >>\nstream\n",
        compressed.len()
    );
    out.extend_from_slice(header.as_bytes());
    out.extend_from_slice(&compressed);
    out.extend_from_slice(b"\nendstream\nendobj\n");

    let xref_pos = out.len();
    let mut xref = format!("xref\n0 {}\n0000000000 65535 f \n", offsets.len() + 1);
    for offset in &offsets {
        xref.push_str(&format!("{offset:010} 00000 n \n"));
    }
    out.extend_from_slice(xref.as_bytes());

    let trailer = format!(
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_pos}\n%%EOF\n",
        offsets.len() + 1
    );
    out.extend_from_slice(trailer.as_bytes());
    Ok(out)
}
