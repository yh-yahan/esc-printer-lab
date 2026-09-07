mod pdf;
pub mod raster;

use std::io;
use std::path::Path;

use image::RgbaImage;

use crate::printer::PrinterProfile;
use crate::receipt::receipt::Receipt;

use self::raster::render_receipt_rgba;

const EXPORT_PX_PER_DOT: f32 = 2.0;

#[derive(Debug)]
pub enum ExportError {
    Io(io::Error),
    Encode(String),
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "{err}"),
            Self::Encode(err) => write!(f, "{err}"),
        }
    }
}

impl From<io::Error> for ExportError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

pub fn default_file_name(extension: &str) -> String {
    format!("receipt.{extension}")
}

pub fn render_receipt(receipt: &Receipt, profile: PrinterProfile) -> RgbaImage {
    render_receipt_rgba(receipt, profile, EXPORT_PX_PER_DOT)
}

pub fn save_png(path: &Path, image: &RgbaImage) -> Result<(), ExportError> {
    image
        .save_with_format(path, image::ImageFormat::Png)
        .map_err(|err| ExportError::Encode(err.to_string()))
}

pub fn save_pdf(path: &Path, image: &RgbaImage, profile: PrinterProfile) -> Result<(), ExportError> {
    std::fs::write(path, pdf::encode(image, profile.paper_width_mm)?)?;
    Ok(())
}

pub fn pick_save_path(title: &str, extension: &str, filter_name: &str) -> Option<std::path::PathBuf> {
    rfd::FileDialog::new()
        .set_title(title)
        .set_file_name(default_file_name(extension))
        .add_filter(filter_name, &[extension])
        .save_file()
}
