use regex::Regex;

#[derive(Debug)]
pub struct FecContext {
    pub f99_text_start: Regex,     // Regex for detecting F99 text start
    pub f99_text_end: Regex,       // Regex for detecting F99 text end
    pub version: Option<String>,   // Parsed version (if any)
    pub version_length: usize,     // Length of the version string
    pub silent: bool,              // Suppress output messages
    pub warn: bool,                // Show warning messages
    pub use_ascii28: bool,         // Whether to use ASCII28 delimiters
    pub summary: bool,             // Whether this is a summary parse
    pub form_type: Option<String>, // Current form type
    pub num_fields: usize,         // Number of fields in the form
    pub include_filing_id: bool,   // Include filing ID in CSV output
    pub fec_id: String,            // Filing ID or file name
}

impl PartialEq for FecContext {
    fn eq(&self, other: &Self) -> bool {
        self.f99_text_start.as_str() == other.f99_text_start.as_str() &&
        self.f99_text_end.as_str() == other.f99_text_end.as_str() &&
        self.version == other.version &&
        self.version_length == other.version_length &&
        self.silent == other.silent &&
        self.warn == other.warn &&
        self.use_ascii28 == other.use_ascii28 &&
        self.summary == other.summary &&
        self.form_type == other.form_type &&
        self.num_fields == other.num_fields &&
        self.include_filing_id == other.include_filing_id &&
        self.fec_id == other.fec_id
    }
}

impl FecContext {
    pub fn new(
        fec_id: String,
        include_filing_id: bool,
        silent: bool,
        warn: bool,
    ) -> Self {
        FecContext {
            f99_text_start: Regex::new(r"(?i)^\s*\[BEGIN ?TEXT\]\s*$").unwrap(),
            f99_text_end: Regex::new(r"(?i)^\s*\[END ?TEXT\]\s*$").unwrap(),
            version: None,
            version_length: 0,
            silent,
            warn,
            use_ascii28: false,
            summary: false,
            form_type: None,
            num_fields: 0,
            include_filing_id,
            fec_id,
        }
    }
}