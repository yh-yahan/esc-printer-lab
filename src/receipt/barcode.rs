use crate::parser::command::{BarcodeSymbology, RasterImage, RasterScale};

const DEFAULT_MODULE_WIDTH: u8 = 3;
const DEFAULT_BAR_HEIGHT: u8 = 162;
const QUIET_MODULES: usize = 10;

pub fn module_width(n: u8) -> u8 {
    match n {
        2..=6 => n,
        _ => DEFAULT_MODULE_WIDTH,
    }
}

pub fn bar_height(n: u8) -> u8 {
    if n == 0 {
        DEFAULT_BAR_HEIGHT
    } else {
        n
    }
}

pub fn hri_text(symbology: BarcodeSymbology, data: &[u8]) -> Option<String> {
    match symbology {
        BarcodeSymbology::UpcA => Some(upc_a_digits(data)?.iter().map(|d| char::from(b'0' + d)).collect()),
        BarcodeSymbology::UpcE => Some(upc_e_digits(data)?.iter().map(|d| char::from(b'0' + d)).collect()),
        BarcodeSymbology::Ean13 => Some(ean13_digits(data)?.iter().map(|d| char::from(b'0' + d)).collect()),
        BarcodeSymbology::Ean8 => Some(ean8_digits(data)?.iter().map(|d| char::from(b'0' + d)).collect()),
        BarcodeSymbology::Code39 | BarcodeSymbology::Itf | BarcodeSymbology::Codabar | BarcodeSymbology::Code93 => {
            let text = String::from_utf8_lossy(data).trim().to_string();
            if text.is_empty() {
                None
            } else {
                Some(text)
            }
        }
        BarcodeSymbology::Code128 => code128_hri(data),
    }
}

pub fn encode_barcode_raster(
    symbology: BarcodeSymbology,
    data: &[u8],
    width: u8,
    height: u8,
) -> Option<RasterImage> {
    let modules = encode_modules(symbology, data)?;
    if modules.is_empty() {
        return None;
    }

    let scale = module_width(width) as usize;
    let bar_h = bar_height(height) as usize;
    let quiet = QUIET_MODULES * scale;
    let content_w = modules.len() * scale;
    let dim_w = quiet * 2 + content_w;
    if dim_w == 0 || bar_h == 0 {
        return None;
    }

    let width_bytes = dim_w.div_ceil(8);
    let mut bits = vec![0u8; width_bytes.saturating_mul(bar_h)];

    for (i, &on) in modules.iter().enumerate() {
        if !on {
            continue;
        }
        let x0 = quiet + i * scale;
        for y in 0..bar_h {
            for dx in 0..scale {
                set_dot(&mut bits, width_bytes, x0 + dx, y);
            }
        }
    }

    Some(RasterImage {
        scale: RasterScale::Normal,
        width_bytes: width_bytes as u16,
        height: bar_h as u16,
        data: bits,
    })
}

fn encode_modules(symbology: BarcodeSymbology, data: &[u8]) -> Option<Vec<bool>> {
    match symbology {
        BarcodeSymbology::UpcA => encode_upc_a(data),
        BarcodeSymbology::UpcE => encode_upc_e(data),
        BarcodeSymbology::Ean13 => encode_ean13(data),
        BarcodeSymbology::Ean8 => encode_ean8(data),
        BarcodeSymbology::Code39 => encode_code39(data),
        BarcodeSymbology::Itf => encode_itf(data),
        BarcodeSymbology::Codabar => encode_codabar(data),
        BarcodeSymbology::Code93 => encode_code93(data),
        BarcodeSymbology::Code128 => encode_code128(data),
    }
}

fn set_dot(bits: &mut [u8], width_bytes: usize, x: usize, y: usize) {
    let byte_index = y.saturating_mul(width_bytes) + (x / 8);
    if let Some(byte) = bits.get_mut(byte_index) {
        *byte |= 0x80 >> (x % 8);
    }
}

fn push_pattern(out: &mut Vec<bool>, pattern: &str) {
    for ch in pattern.chars() {
        out.push(ch == '1');
    }
}

fn push_widths(out: &mut Vec<bool>, widths: &[u8], start_bar: bool) {
    let mut bar = start_bar;
    for &w in widths {
        for _ in 0..w {
            out.push(bar);
        }
        bar = !bar;
    }
}

fn digits_only(data: &[u8]) -> Vec<u8> {
    data.iter()
        .copied()
        .filter(|b| b.is_ascii_digit())
        .map(|b| b - b'0')
        .collect()
}

fn ean_checksum(digits: &[u8]) -> u8 {
    let mut sum = 0u32;
    for (i, &d) in digits.iter().rev().enumerate() {
        sum += d as u32 * if i % 2 == 0 { 3 } else { 1 };
    }
    ((10 - (sum % 10)) % 10) as u8
}

const EAN_L: [&str; 10] = [
    "0001101", "0011001", "0010011", "0111101", "0100011",
    "0110001", "0101111", "0111011", "0110111", "0001011",
];
const EAN_G: [&str; 10] = [
    "0100111", "0110011", "0011011", "0100001", "0011101",
    "0111001", "0000101", "0010001", "0001001", "0010111",
];
const EAN_R: [&str; 10] = [
    "1110010", "1100110", "1101100", "1000010", "1011100",
    "1001110", "1010000", "1000100", "1001000", "1110100",
];
const EAN13_PARITY: [&str; 10] = [
    "LLLLLL", "LLGLGG", "LLGGLG", "LLGGGL", "LGLLGG",
    "LGGLLG", "LGGGLL", "LGLGLG", "LGLGGL", "LGGLGL",
];

fn ean13_digits(data: &[u8]) -> Option<[u8; 13]> {
    let d = digits_only(data);
    match d.len() {
        12 => {
            let mut out = [0u8; 13];
            out[..12].copy_from_slice(&d);
            out[12] = ean_checksum(&d);
            Some(out)
        }
        13 => {
            let mut out = [0u8; 13];
            out.copy_from_slice(&d);
            Some(out)
        }
        _ => None,
    }
}

fn ean8_digits(data: &[u8]) -> Option<[u8; 8]> {
    let d = digits_only(data);
    match d.len() {
        7 => {
            let mut out = [0u8; 8];
            out[..7].copy_from_slice(&d);
            out[7] = ean_checksum(&d);
            Some(out)
        }
        8 => {
            let mut out = [0u8; 8];
            out.copy_from_slice(&d);
            Some(out)
        }
        _ => None,
    }
}

fn upc_a_digits(data: &[u8]) -> Option<[u8; 12]> {
    let d = digits_only(data);
    match d.len() {
        11 => {
            let mut out = [0u8; 12];
            out[..11].copy_from_slice(&d);
            out[11] = upc_a_checksum(&d);
            Some(out)
        }
        12 => {
            let mut out = [0u8; 12];
            out.copy_from_slice(&d);
            Some(out)
        }
        _ => None,
    }
}

fn upc_a_checksum(digits11: &[u8]) -> u8 {
    let mut ean = Vec::with_capacity(12);
    ean.push(0);
    ean.extend_from_slice(digits11);
    ean_checksum(&ean)
}

fn encode_ean13(data: &[u8]) -> Option<Vec<bool>> {
    let digits = ean13_digits(data)?;
    let mut out = Vec::new();
    push_pattern(&mut out, "101");
    let parity = EAN13_PARITY[digits[0] as usize];
    for (i, ch) in parity.chars().enumerate() {
        let digit = digits[i + 1] as usize;
        push_pattern(
            &mut out,
            if ch == 'L' { EAN_L[digit] } else { EAN_G[digit] },
        );
    }
    push_pattern(&mut out, "01010");
    for digit in digits[7..].iter() {
        push_pattern(&mut out, EAN_R[*digit as usize]);
    }
    push_pattern(&mut out, "101");
    Some(out)
}

fn encode_ean8(data: &[u8]) -> Option<Vec<bool>> {
    let digits = ean8_digits(data)?;
    let mut out = Vec::new();
    push_pattern(&mut out, "101");
    for digit in digits[..4].iter() {
        push_pattern(&mut out, EAN_L[*digit as usize]);
    }
    push_pattern(&mut out, "01010");
    for digit in digits[4..].iter() {
        push_pattern(&mut out, EAN_R[*digit as usize]);
    }
    push_pattern(&mut out, "101");
    Some(out)
}

fn encode_upc_a(data: &[u8]) -> Option<Vec<bool>> {
    let digits = upc_a_digits(data)?;
    let mut ean = Vec::with_capacity(13);
    ean.push(0);
    ean.extend_from_slice(&digits);
    encode_ean13(&ean.into_iter().map(|d| b'0' + d).collect::<Vec<_>>())
}

fn upc_e_digits(data: &[u8]) -> Option<[u8; 8]> {
    let d = digits_only(data);
    match d.len() {
        6 => {
            let mut out = [0u8; 8];
            out[0] = 0;
            out[1..7].copy_from_slice(&d);
            let upc_a = expand_upc_e(&out[1..7], 0)?;
            out[7] = upc_a[11];
            Some(out)
        }
        7 => {
            let ns = d[0];
            if ns > 1 {
                return None;
            }
            let mut out = [0u8; 8];
            out[0] = ns;
            out[1..7].copy_from_slice(&d[1..]);
            let upc_a = expand_upc_e(&out[1..7], ns)?;
            out[7] = upc_a[11];
            Some(out)
        }
        8 => {
            let ns = d[0];
            if ns > 1 {
                return None;
            }
            let mut out = [0u8; 8];
            out.copy_from_slice(&d);
            Some(out)
        }
        _ => None,
    }
}

fn expand_upc_e(body: &[u8], ns: u8) -> Option<[u8; 12]> {
    if body.len() != 6 || ns > 1 {
        return None;
    }
    let mut mfr = [0u8; 5];
    let mut prod = [0u8; 5];
    match body[5] {
        0..=2 => {
            mfr[0] = body[0];
            mfr[1] = body[1];
            mfr[2] = body[5];
            prod[2] = body[2];
            prod[3] = body[3];
            prod[4] = body[4];
        }
        3 => {
            mfr[0] = body[0];
            mfr[1] = body[1];
            mfr[2] = body[2];
            prod[3] = body[3];
            prod[4] = body[4];
        }
        4 => {
            mfr[0] = body[0];
            mfr[1] = body[1];
            mfr[2] = body[2];
            mfr[3] = body[3];
            prod[4] = body[4];
        }
        5..=9 => {
            mfr[0] = body[0];
            mfr[1] = body[1];
            mfr[2] = body[2];
            mfr[3] = body[3];
            mfr[4] = body[4];
            prod[4] = body[5];
        }
        _ => return None,
    }

    let mut digits = [0u8; 11];
    digits[0] = ns;
    digits[1..6].copy_from_slice(&mfr);
    digits[6..11].copy_from_slice(&prod);
    let mut out = [0u8; 12];
    out[..11].copy_from_slice(&digits);
    out[11] = ean_checksum(&digits);
    Some(out)
}

const UPCE_PARITY_EVEN: [&str; 10] = [
    "EEEOOO", "EEOEOO", "EEOOEO", "EEOOOE", "EOEEOO",
    "EOOEEO", "EOOOEE", "EOEOEO", "EOEOOE", "EOOEOE",
];
const UPCE_PARITY_ODD: [&str; 10] = [
    "OOOEEE", "OOEOEE", "OOEEOE", "OOEEEO", "OEOOEE",
    "OEEOOE", "OEEEOO", "OEOEOE", "OEOEEO", "OEEOEO",
];

fn encode_upc_e(data: &[u8]) -> Option<Vec<bool>> {
    let digits = upc_e_digits(data)?;
    let parity_row = if digits[0] == 0 {
        UPCE_PARITY_EVEN[digits[7] as usize]
    } else {
        UPCE_PARITY_ODD[digits[7] as usize]
    };
    let mut out = Vec::new();
    push_pattern(&mut out, "101");
    for (i, kind) in parity_row.chars().enumerate() {
        let digit = digits[i + 1] as usize;
        push_pattern(
            &mut out,
            if kind == 'O' { EAN_L[digit] } else { EAN_G[digit] },
        );
    }
    push_pattern(&mut out, "010101");
    Some(out)
}

const CODE39: [(u8, &str); 44] = [
    (b'0', "nnnwwnwnn"),
    (b'1', "wnnwnnnnw"),
    (b'2', "nnwwnnnnw"),
    (b'3', "wnwwnnnnn"),
    (b'4', "nnnwwnnnw"),
    (b'5', "wnnwwnnnn"),
    (b'6', "nnwwwnnnn"),
    (b'7', "nnnwnnwnw"),
    (b'8', "wnnwnnwnn"),
    (b'9', "nnwwnnwnn"),
    (b'A', "wnnnnwnnw"),
    (b'B', "nnwnnwnnw"),
    (b'C', "wnwnnwnnn"),
    (b'D', "nnnnwwnnw"),
    (b'E', "wnnnwwnnn"),
    (b'F', "nnwnwwnnn"),
    (b'G', "nnnnnwwnw"),
    (b'H', "wnnnnwwnn"),
    (b'I', "nnwnnwwnn"),
    (b'J', "nnnnwwwnn"),
    (b'K', "wnnnnnnww"),
    (b'L', "nnwnnnnww"),
    (b'M', "wnwnnnnwn"),
    (b'N', "nnnnwnnww"),
    (b'O', "wnnnwnnwn"),
    (b'P', "nnwnwnnwn"),
    (b'Q', "nnnnnnwww"),
    (b'R', "wnnnnnwwn"),
    (b'S', "nnwnnnwwn"),
    (b'T', "nnnnwnwwn"),
    (b'U', "wwnnnnnnw"),
    (b'V', "nwwnnnnnw"),
    (b'W', "wwwnnnnnn"),
    (b'X', "nwnnwnnnw"),
    (b'Y', "wwnnwnnnn"),
    (b'Z', "nwwnwnnnn"),
    (b'-', "nwnnnnwnw"),
    (b'.', "wwnnnnwnn"),
    (b' ', "nwwnnnwnn"),
    (b'$', "nwnwnwnnn"),
    (b'/', "nwnwnnnwn"),
    (b'+', "nwnnnwnwn"),
    (b'%', "nnnwnwnwn"),
    (b'*', "nwnnwnwnn"),
];

fn code39_pattern(ch: u8) -> Option<&'static str> {
    CODE39.iter().find(|(c, _)| *c == ch).map(|(_, p)| *p)
}

fn encode_code39(data: &[u8]) -> Option<Vec<bool>> {
    let mut chars = Vec::new();
    for &b in data {
        let ch = if b.is_ascii_lowercase() {
            b.to_ascii_uppercase()
        } else {
            b
        };
        if ch == b'*' {
            continue;
        }
        code39_pattern(ch)?;
        chars.push(ch);
    }
    if chars.is_empty() {
        return None;
    }

    let mut out = Vec::new();
    push_code39_char(&mut out, b'*');
    for ch in chars {
        out.push(false);
        push_code39_char(&mut out, ch);
    }
    out.push(false);
    push_code39_char(&mut out, b'*');
    Some(out)
}

fn push_code39_char(out: &mut Vec<bool>, ch: u8) {
    let pattern = code39_pattern(ch).unwrap();
    let mut bar = true;
    for p in pattern.chars() {
        let w = if p == 'w' { 3 } else { 1 };
        for _ in 0..w {
            out.push(bar);
        }
        bar = !bar;
    }
}

const ITF_PATTERNS: [[u8; 5]; 10] = [
    [1, 1, 3, 3, 1],
    [3, 1, 1, 1, 3],
    [1, 3, 1, 1, 3],
    [3, 3, 1, 1, 1],
    [1, 1, 3, 1, 3],
    [3, 1, 3, 1, 1],
    [1, 3, 3, 1, 1],
    [1, 1, 1, 3, 3],
    [3, 1, 1, 3, 1],
    [1, 3, 1, 3, 1],
];

fn encode_itf(data: &[u8]) -> Option<Vec<bool>> {
    let mut digits = digits_only(data);
    if digits.is_empty() {
        return None;
    }
    if digits.len() % 2 == 1 {
        digits.insert(0, 0);
    }

    let mut out = Vec::new();
    push_widths(&mut out, &[1, 1, 1, 1], true);
    for pair in digits.chunks(2) {
        let a = ITF_PATTERNS[pair[0] as usize];
        let b = ITF_PATTERNS[pair[1] as usize];
        for i in 0..5 {
            for _ in 0..a[i] {
                out.push(true);
            }
            for _ in 0..b[i] {
                out.push(false);
            }
        }
    }
    push_widths(&mut out, &[3, 1, 1], true);
    Some(out)
}

const CODABAR: [(u8, [u8; 7]); 20] = [
    (b'0', [1, 1, 1, 1, 1, 3, 3]),
    (b'1', [1, 1, 1, 1, 3, 3, 1]),
    (b'2', [1, 1, 1, 3, 1, 1, 3]),
    (b'3', [3, 3, 1, 1, 1, 1, 1]),
    (b'4', [1, 1, 3, 1, 1, 3, 1]),
    (b'5', [3, 1, 1, 1, 1, 3, 1]),
    (b'6', [1, 3, 1, 1, 1, 1, 3]),
    (b'7', [1, 3, 1, 1, 3, 1, 1]),
    (b'8', [1, 3, 3, 1, 1, 1, 1]),
    (b'9', [3, 1, 1, 3, 1, 1, 1]),
    (b'-', [1, 1, 1, 3, 3, 1, 1]),
    (b'$', [1, 1, 3, 3, 1, 1, 1]),
    (b':', [3, 1, 1, 1, 3, 1, 3]),
    (b'/', [3, 1, 3, 1, 1, 1, 3]),
    (b'.', [3, 1, 3, 1, 3, 1, 1]),
    (b'+', [1, 1, 3, 1, 3, 1, 3]),
    (b'A', [1, 1, 3, 3, 1, 3, 1]),
    (b'B', [1, 3, 1, 3, 1, 1, 3]),
    (b'C', [1, 1, 1, 3, 1, 3, 3]),
    (b'D', [1, 1, 3, 1, 3, 3, 1]),
];

fn encode_codabar(data: &[u8]) -> Option<Vec<bool>> {
    let mut chars = Vec::new();
    for &b in data {
        let ch = b.to_ascii_uppercase();
        if !CODABAR.iter().any(|(c, _)| *c == ch) {
            return None;
        }
        chars.push(ch);
    }
    if chars.len() < 2 {
        return None;
    }

    let mut out = Vec::new();
    for (i, ch) in chars.iter().enumerate() {
        if i > 0 {
            out.push(false);
        }
        let widths = CODABAR.iter().find(|(c, _)| c == ch).map(|(_, w)| w)?;
        push_widths(&mut out, widths, true);
    }
    Some(out)
}

const CODE93_CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ-. $/+%";

// 9-module patterns, start/stop is extra.
const CODE93_PATTERNS: [&str; 47] = [
    "100010100", "101001000", "101000100", "101000010", "100101000",
    "100100100", "100100010", "101010000", "100010010", "100001010",
    "110101000", "110100100", "110100010", "110010100", "110010010",
    "110001010", "101101000", "101100100", "101100010", "100110100",
    "100011010", "101011000", "101001100", "101000110", "100101100",
    "100010110", "110110100", "110110010", "110101100", "110100110",
    "110010110", "110011010", "101101100", "101100110", "100110110",
    "100111010", "100101110", "111010100", "111010010", "111001010",
    "101101110", "101110110", "110101110", "100100110", "111011010",
    "111010110", "100110010",
];
const CODE93_START_STOP: &str = "101011110";
const CODE93_TERMINATOR: &str = "1";

fn code93_index(ch: u8) -> Option<usize> {
    CODE93_CHARS.iter().position(|&c| c == ch)
}

fn encode_code93(data: &[u8]) -> Option<Vec<bool>> {
    let mut values = Vec::new();
    for &b in data {
        let ch = if (b as char).is_ascii_lowercase() {
            b.to_ascii_uppercase()
        } else {
            b
        };
        values.push(code93_index(ch)?);
    }
    if values.is_empty() {
        return None;
    }

    let c = code93_checksum(&values, 20);
    values.push(c);
    let k = code93_checksum(&values, 15);
    values.push(k);

    let mut out = Vec::new();
    push_pattern(&mut out, CODE93_START_STOP);
    for v in values {
        push_pattern(&mut out, CODE93_PATTERNS[v]);
    }
    push_pattern(&mut out, CODE93_START_STOP);
    push_pattern(&mut out, CODE93_TERMINATOR);
    Some(out)
}

fn code93_checksum(values: &[usize], max_weight: usize) -> usize {
    let mut sum = 0usize;
    let mut weight = 1usize;
    for &v in values.iter().rev() {
        sum += v * weight;
        weight += 1;
        if weight > max_weight {
            weight = 1;
        }
    }
    sum % 47
}

const CODE128_PATTERNS: [&str; 107] = [
    "11011001100", "11001101100", "11001100110", "10010011000", "10010001100",
    "10001001100", "10011001000", "10011000100", "10001100100", "11001001000",
    "11001000100", "11000100100", "10110011100", "10011011100", "10011001110",
    "10111001100", "10011101100", "10011100110", "11001110010", "11001011100",
    "11001001110", "11011100100", "11001110100", "11101101110", "11101001100",
    "11100101100", "11100100110", "11101100100", "11100110100", "11100110010",
    "11011011000", "11011000110", "11000110110", "10100011000", "10001011000",
    "10001000110", "10110001000", "10001101000", "10001100010", "11010001000",
    "11000101000", "11000100010", "10110111000", "10110001110", "10001101110",
    "10111011000", "10111000110", "10001110110", "11101110110", "11010001110",
    "11000101110", "11011101000", "11011100010", "11011101110", "11101011000",
    "11101000110", "11100010110", "11101101000", "11101100010", "11100011010",
    "11101111010", "11001000010", "11110001010", "10100110000", "10100001100",
    "10010110000", "10010000110", "10000101100", "10000100110", "10110010000",
    "10110000100", "10011010000", "10011000010", "10000110100", "10000110010",
    "11000010010", "11001010000", "11110111010", "11000010100", "10001111010",
    "10100111100", "10010111100", "10010011110", "10111100100", "10011110100",
    "10011110010", "11110100100", "11110010100", "11110010010", "11011011110",
    "11011110110", "11110110110", "10101111000", "10100011110", "10001011110",
    "10111101000", "10111100010", "11110101000", "11110100010", "10111011110",
    "10111101110", "11101011110", "11110101110", "11010001100", "11010000110",
    "11010011100", "11000111010",
];
const CODE128_STOP: &str = "1100011101011";

const CODE128_START_A: u8 = 103;
const CODE128_START_B: u8 = 104;
const CODE128_START_C: u8 = 105;
const CODE128_CODE_A: u8 = 101;
const CODE128_CODE_B: u8 = 100;
const CODE128_CODE_C: u8 = 99;
const CODE128_FNC1: u8 = 102;
const CODE128_FNC2_A_B: u8 = 97;
const CODE128_FNC3_A_B: u8 = 96;
const CODE128_SHIFT: u8 = 98;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Code128Set {
    A,
    B,
    C,
}

fn encode_code128(data: &[u8]) -> Option<Vec<bool>> {
    let symbols = code128_symbols(data)?;
    if symbols.is_empty() {
        return None;
    }

    let mut checksum = symbols[0] as u32;
    for (i, &sym) in symbols.iter().enumerate().skip(1) {
        checksum += sym as u32 * i as u32;
    }
    let check = (checksum % 103) as u8;

    let mut out = Vec::new();
    for &sym in symbols.iter() {
        push_pattern(&mut out, CODE128_PATTERNS[sym as usize]);
    }
    push_pattern(&mut out, CODE128_PATTERNS[check as usize]);
    push_pattern(&mut out, CODE128_STOP);
    Some(out)
}

fn code128_symbols(data: &[u8]) -> Option<Vec<u8>> {
    if data.is_empty() {
        return None;
    }

    let mut i = 0usize;
    let mut set = Code128Set::B;
    let mut symbols = Vec::new();
    let mut started = false;

    while i < data.len() {
        if data[i] == b'{' && i + 1 < data.len() {
            match data[i + 1] {
                b'{' => {
                    if !started {
                        started = true;
                        symbols.push(CODE128_START_B);
                        set = Code128Set::B;
                    }
                    symbols.push(code128_char_value(b'{', set)?);
                    i += 2;
                    continue;
                }
                b'A' => {
                    if !started {
                        started = true;
                        symbols.push(CODE128_START_A);
                    } else if set != Code128Set::A {
                        symbols.push(CODE128_CODE_A);
                    }
                    set = Code128Set::A;
                    i += 2;
                    continue;
                }
                b'B' => {
                    if !started {
                        started = true;
                        symbols.push(CODE128_START_B);
                    } else if set != Code128Set::B {
                        symbols.push(CODE128_CODE_B);
                    }
                    set = Code128Set::B;
                    i += 2;
                    continue;
                }
                b'C' => {
                    if !started {
                        started = true;
                        symbols.push(CODE128_START_C);
                    } else if set != Code128Set::C {
                        symbols.push(CODE128_CODE_C);
                    }
                    set = Code128Set::C;
                    i += 2;
                    continue;
                }
                b'1' => {
                    if !started {
                        started = true;
                        symbols.push(CODE128_START_B);
                        set = Code128Set::B;
                    }
                    symbols.push(CODE128_FNC1);
                    i += 2;
                    continue;
                }
                b'2' => {
                    if !started {
                        started = true;
                        symbols.push(CODE128_START_B);
                        set = Code128Set::B;
                    }
                    if matches!(set, Code128Set::C) {
                        return None;
                    }
                    symbols.push(CODE128_FNC2_A_B);
                    i += 2;
                    continue;
                }
                b'3' => {
                    if !started {
                        started = true;
                        symbols.push(CODE128_START_B);
                        set = Code128Set::B;
                    }
                    if matches!(set, Code128Set::C) {
                        return None;
                    }
                    symbols.push(CODE128_FNC3_A_B);
                    i += 2;
                    continue;
                }
                b'S' => {
                    if !started {
                        started = true;
                        symbols.push(CODE128_START_B);
                        set = Code128Set::B;
                    }
                    if matches!(set, Code128Set::C) || i + 2 >= data.len() {
                        return None;
                    }
                    symbols.push(CODE128_SHIFT);
                    let shifted = if set == Code128Set::A {
                        Code128Set::B
                    } else {
                        Code128Set::A
                    };
                    symbols.push(code128_char_value(data[i + 2], shifted)?);
                    i += 3;
                    continue;
                }
                _ => {}
            }
        }

        if !started {
            started = true;
            symbols.push(CODE128_START_B);
            set = Code128Set::B;
        }

        if set == Code128Set::C {
            if i + 1 >= data.len() || !data[i].is_ascii_digit() || !data[i + 1].is_ascii_digit() {
                return None;
            }
            symbols.push((data[i] - b'0') * 10 + (data[i + 1] - b'0'));
            i += 2;
        } else {
            symbols.push(code128_char_value(data[i], set)?);
            i += 1;
        }
    }

    if symbols.len() < 2 {
        return None;
    }
    Some(symbols)
}

fn code128_char_value(ch: u8, set: Code128Set) -> Option<u8> {
    match set {
        Code128Set::B => match ch {
            0x20..=0x7F => Some(ch - 0x20),
            _ => None,
        },
        Code128Set::A => match ch {
            0x20..=0x5F => Some(ch - 0x20),
            0x00..=0x1F => Some(ch + 64),
            _ => None,
        },
        Code128Set::C => None,
    }
}

fn code128_hri(data: &[u8]) -> Option<String> {
    if data.is_empty() {
        return None;
    }
    let mut out = String::new();
    let mut i = 0usize;
    let mut set = Code128Set::B;
    while i < data.len() {
        if data[i] == b'{' && i + 1 < data.len() {
            match data[i + 1] {
                b'A' => {
                    set = Code128Set::A;
                    i += 2;
                    continue;
                }
                b'B' => {
                    set = Code128Set::B;
                    i += 2;
                    continue;
                }
                b'C' => {
                    set = Code128Set::C;
                    i += 2;
                    continue;
                }
                b'1' | b'2' | b'3' | b'S' => {
                    i += 2;
                    continue;
                }
                b'{' => {
                    out.push('{');
                    i += 2;
                    continue;
                }
                _ => {}
            }
        }
        if set == Code128Set::C {
            if i + 1 < data.len() && data[i].is_ascii_digit() && data[i + 1].is_ascii_digit() {
                out.push(data[i] as char);
                out.push(data[i + 1] as char);
                i += 2;
                continue;
            }
        }
        if data[i].is_ascii_graphic() || data[i] == b' ' {
            out.push(data[i] as char);
        }
        i += 1;
    }
    if out.is_empty() { None } else { Some(out) }
}
