//! A Rust module replicating `encoding.c` logic using Hoehrmann's UTF-8 state machine.
//!
//! - Checks each byte to see if ASCII28 is present.
//! - Tracks ASCII-only vs. not (optional).
//! - If the data is invalid UTF-8, fallback to ISO-8859-1 conversion.
//!
//! This matches the original C approach from `encoding.c`, but in safe, idiomatic Rust.

/// The Hoehrmann state machine's "ACCEPT" and "REJECT" states.
const UTF8_ACCEPT: u32 = 0;
const UTF8_REJECT: u32 = 1;

/// The Hoehrmann `utf8d` table, replicated from the C code (256 + 6*16 = 352 elements).
/// In the C code, it's a big static array named `utf8d[]`.
static UTF8D: [u8; 400] = [
    // 0..255 for "type" mapping
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, // 0x00..0x1F
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, // 0x20..0x3F
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, // 0x40..0x5F
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, // 0x60..0x7F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
    9, // 0x80..0x9F
    7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
    7, // 0xA0..0xBF
    8, 8, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, // 0xC0..0xDF
    0xa, 0x3, 0x3, 0x3, 0x3, 0x3, 0x3, 0x3, 0x3, 0x3, 0x3, 0x3, 0x3, 0x4, 0x3,
    0x3, // 0xE0..0xEF
    0xb, 0x6, 0x6, 0x6, 0x5, 0x8, 0x8, 0x8, 0x8, 0x8, 0x8, 0x8, 0x8, 0x8, 0x8,
    0x8, // 0xF0..0xFF
    // 256.. (the "transition" table, 6 rows * 16 columns = 96)
    0x0, 0x1, 0x2, 0x3, 0x5, 0x8, 0x7, 0x1, 0x1, 0x1, 0x4, 0x6, 0x1, 0x1, 0x1, 0x1, // s0..s0
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 1, 1, 1,
    1, // s1..s2
    1, 2, 1, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1,
    1, // s3..s4
    1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 1, 3, 1, 1, 1, 1, 1,
    1, // s5..s6
    1, 3, 1, 1, 1, 1, 1, 3, 1, 3, 1, 1, 1, 1, 1, 1, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, // s7..s8
];

/// A structure to hold line information, mimicking `LINE_INFO` from C.
#[derive(Debug)]
pub struct LineInfo {
    /// Whether ASCII 28 (the "file separator") was found.
    pub ascii28: bool,
    /// Whether the line was entirely ASCII (<128). This is optional usage.
    pub ascii_only: bool,
    /// Whether the line was valid UTF-8 (according to the Hoehrmann state machine).
    pub valid_utf8: bool,
    /// The total number of bytes encountered (excluding the null terminator in C).
    pub length: usize,
}

impl Default for LineInfo {
    fn default() -> Self {
        Self {
            ascii28: false,
            ascii_only: true,
            valid_utf8: true,
            length: 0,
        }
    }
}

/// Collect line info by iterating bytes and applying the Hoehrmann UTF-8 state machine.
///
/// - `data`: raw bytes from the line
/// - returns: a `LineInfo` with flags for ascii28, ascii_only, valid_utf8, and the length
fn collect_line_info(data: &[u8]) -> LineInfo {
    let mut info = LineInfo::default();
    let mut state: u32 = UTF8_ACCEPT; // start in accept state

    for &byte in data {
        info.length += 1;

        if byte == 28 {
            info.ascii28 = true;
        }
        if byte > 127 {
            info.ascii_only = false;
        }

        let t = UTF8D[byte as usize];
        // compute next state
        state = UTF8D[256 + (state * 16 + t as u32) as usize] as u32;
        if state == UTF8_REJECT {
            info.valid_utf8 = false;
            // We won't break early, because the original code
            // just keeps reading the whole line. The final result is "invalid"
        }
    }

    info
}

/// Convert ISO-8859-1 bytes to UTF-8, storing the result in a new Vec<u8>.
/// This matches the logic from `iso_8859_1_to_utf_8`.
fn iso_8859_1_to_utf8(data: &[u8]) -> Vec<u8> {
    // Worst case size: 2 * data.len()
    let mut output = Vec::with_capacity(data.len() * 2);

    for &b in data {
        if b < 128 {
            output.push(b);
        } else {
            // "0xc2 + (b > 0xbf)" => if b > 0xBF, we use 0xc3, else 0xc2
            let first = 0xc2 + ((b > 0xbf) as u8);
            let second = (b & 0x3f) + 0x80;
            output.push(first);
            output.push(second);
        }
    }
    output
}

/// Decode a line from raw bytes, returning a `(decoded_string, ascii28_found)`.
///
/// - We first apply `collect_line_info` to detect ASCII28, check validity, etc.
/// - If it is invalid UTF-8, we fallback to ISO-8859-1 → UTF-8.
/// - We return the final `String` plus a boolean if ASCII28 was found.
pub fn decode_line(data: &[u8]) -> (String, bool) {
    // 1. Collect line info
    let info = collect_line_info(data);

    // 2. If not valid UTF-8, fallback to ISO-8859-1
    if !info.valid_utf8 {
        let converted = iso_8859_1_to_utf8(data);
        // Safe to unwrap because it's guaranteed valid now
        let s = String::from_utf8(converted).unwrap();
        return (s, info.ascii28);
    }

    // 3. If valid, we can interpret the original data as UTF-8 safely.
    //    The original code would just do "copyString(in, out)", i.e. no transformation.
    if let Ok(s) = std::str::from_utf8(data) {
        (s.to_string(), info.ascii28)
    } else {
        // If we can't interpret as UTF-8 (unlikely if valid_utf8 == true, but just in case)
        // fallback as well
        let converted = iso_8859_1_to_utf8(data);
        let s = String::from_utf8(converted).unwrap();
        (s, info.ascii28)
    }
}
