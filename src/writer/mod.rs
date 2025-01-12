//! A Rust module that replicates the functionality of `writer.c` and `writer.h`,
//! now with optional CSV crate integration.
//
//! This includes:
//! - A buffered file writer (`BufferFile`) with flush logic.
//! - A `WriterContext` that can manage multiple files (by name), custom callbacks, etc.
//! - Methods for writing strings, characters, doubles, and flushing/closing resources.
//! - An optional `write_csv_record` method using the `csv` crate to properly escape fields.

use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

// NEW: import the csv crate
use csv::WriterBuilder;

use anyhow::{anyhow, Result};

/// The default CSV extension, as in the original code.
pub const CSV_EXTENSION: &str = ".csv";

/// An optional custom write callback, akin to the old `CustomWriteFunction`.
/// In Rust, we store it as a boxed closure returning `Result<()>`.
pub type CustomWriteFn = dyn Fn(&str, &str, &[u8]) -> Result<()> + Send + Sync;

/// An optional custom line callback, akin to the old `CustomLineFunction`.
pub type CustomLineFn = dyn Fn(&str, &str, &str) -> Result<()> + Send + Sync;

/// A buffered file that replicates `BUFFER_FILE`.
/// - We store `buffer` as a `Vec<u8>` rather than a raw pointer.
/// - We track `position` within this vector.
pub struct BufferFile {
    buffer: Vec<u8>,
    position: usize,
    capacity: usize,
}

impl BufferFile {
    /// Create a new buffer with a given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            position: 0,
            capacity,
        }
    }

    /// Write raw bytes into this buffer, flushing if we exceed capacity.
    /// Returns the leftover bytes if buffer is full.
    fn write_bytes(&mut self, data: &[u8]) -> Vec<u8> {
        let space_left = self.capacity - self.position;
        if data.len() <= space_left {
            // Fits in buffer
            self.buffer.extend_from_slice(data);
            self.position += data.len();
            Vec::new()
        } else {
            // Part of data fits, the rest is overflow
            let (first, second) = data.split_at(space_left);
            self.buffer.extend_from_slice(first);
            self.position += first.len();
            second.to_vec()
        }
    }

    /// Check if buffer is empty.
    fn is_empty(&self) -> bool {
        self.position == 0
    }

    /// Reset buffer position to 0 after flushing.
    fn clear(&mut self) {
        self.buffer.clear();
        self.position = 0;
    }
}

/// Represents an entry in the open files map, containing the buffer and file handle.
struct FileEntry {
    buffer_file: BufferFile,
    file: Option<File>, // Actual file handle if writing to disk
}

impl FileEntry {
    fn new(buffer_capacity: usize, file: Option<File>) -> Self {
        Self {
            buffer_file: BufferFile::new(buffer_capacity),
            file,
        }
    }
}

/// The main writer context, replicating `WRITE_CONTEXT`.
pub struct WriterContext {
    /// The directory path where output files go (if writing to files).
    pub output_directory: String,
    /// The "filing ID" used in building full paths.
    pub filing_id: String,
    /// Whether we actually write to disk or not (like `writeToFile`).
    pub write_to_disk: bool,
    /// The buffer size for each file (akin to `bufferSize`).
    pub buffer_size: usize,

    /// A map of `(filename, extension)` => FileEntry (which holds `BufferFile` + `File`).
    open_files: HashMap<(String, String), FileEntry>,

    /// The "last" file we wrote to, used for optimization.
    last_file_key: Option<(String, String)>,

    /// A local buffer mode (if `local` in the original code is set).
    local_mode: bool,
    local_buffer: String,
    local_buffer_pos: usize,

    /// The custom line function, if any (like `customLineFunction`).
    custom_line_fn: Option<Box<CustomLineFn>>,
    /// A buffer that accumulates the current line. Once we call `end_line`, we pass it to `custom_line_fn`.
    custom_line_buffer: String,

    /// The custom write function, if any (like `customWriteFunction`).
    custom_write_fn: Option<Box<CustomWriteFn>>,
}

impl WriterContext {
    /// Create a new `WriterContext`, replacing `newWriteContext`.
    /// - `output_directory`: e.g. "output/"
    /// - `filing_id`: e.g. "12345"
    /// - `write_to_disk`: whether we actually write to files
    /// - `buffer_size`: each file buffer capacity
    /// - `custom_write_fn`: optional closure for custom writes
    /// - `custom_line_fn`: optional closure for custom lines
    pub fn new(
        output_directory: String,
        filing_id: String,
        write_to_disk: bool,
        buffer_size: usize,
        custom_write_fn: Option<Box<CustomWriteFn>>,
        custom_line_fn: Option<Box<CustomLineFn>>,
    ) -> Self {
        Self {
            output_directory,
            filing_id,
            write_to_disk,
            buffer_size,
            open_files: HashMap::new(),
            last_file_key: None,
            local_mode: false,
            local_buffer: String::new(),
            local_buffer_pos: 0,
            custom_line_fn,
            custom_line_buffer: String::new(),
            custom_write_fn,
        }
    }

    /// Enable local buffer mode.
    pub fn start_local_buffer_mode(&mut self) {
        self.local_mode = true;
        self.local_buffer.clear();
        self.local_buffer_pos = 0;
    }

    /// Disable local buffer mode and retrieve the buffer content.
    pub fn finish_local_buffer_mode(&mut self) -> String {
        self.local_mode = false;
        let content = self.local_buffer.clone();
        self.local_buffer.clear();
        self.local_buffer_pos = 0;
        content
    }

    /// End the current line and call the custom line function if set.
    /// `types` is a string describing the field types for this line.
    pub fn end_line(&mut self, types: &str) -> Result<()> {
        if let Some(ref line_fn) = self.custom_line_fn {
            line_fn(
                self.last_file_key
                    .as_ref()
                    .map(|(f, _)| f.as_str())
                    .unwrap_or(""),
                &self.custom_line_buffer,
                types,
            )?;
        }
        self.custom_line_buffer.clear();
        Ok(())
    }

    /// Retrieve an existing or create a new `FileEntry`.
    fn get_file_entry(
        &mut self,
        filename: &str,
        extension: &str,
    ) -> Result<(&mut FileEntry, bool)> {
        if let Some(ref key) = self.last_file_key {
            if key.0 == filename && key.1 == extension {
                return Ok((
                    self.open_files
                        .get_mut(key)
                        .map(|fe| fe)
                        .ok_or_else(|| anyhow!("File entry not found in open_files!"))?,
                    false,
                ));
            }
        }

        let key = (filename.to_string(), extension.to_string());
        if self.open_files.contains_key(&key) {
            self.last_file_key = Some(key.clone());
            return Ok((
                self.open_files
                    .get_mut(&key)
                    .map(|fe| fe)
                    .ok_or_else(|| anyhow!("File entry not found in open_files!"))?,
                false,
            ));
        }

        let file = if self.write_to_disk {
            let dir_path = Path::new(&self.output_directory).join(&self.filing_id);
            std::fs::create_dir_all(&dir_path)?;
            let normalized_filename = filename.replace('/', "-");
            let fullpath = dir_path
                .join(&normalized_filename)
                .with_extension(extension.trim_start_matches('.'));
            Some(
                OpenOptions::new()
                    .write(true)
                    .create(true)
                    .append(true) // Changed from truncate(true) to append(true) to avoid overwriting
                    .open(fullpath)?,
            )
        } else {
            None
        };

        let entry = FileEntry::new(self.buffer_size, file);
        self.open_files.insert(key.clone(), entry);
        self.last_file_key = Some(key.clone());
        Ok((
            self.open_files
                .get_mut(&key)
                .ok_or_else(|| anyhow!("Failed to insert new FileEntry"))?,
            true,
        ))
    }

    /// Internal flush logic that writes the buffer out to disk or to the custom write fn.
    fn flush_buffer(&mut self, filename: &str, extension: &str) -> Result<()> {
        // Attempt to get the file entry
        let (buffer, file_option) = {
            let (entry, _) = self.get_file_entry(filename, extension)?;

            if entry.buffer_file.is_empty() {
                return Ok(()); // Nothing to flush
            }

            // Clone the buffer content to write it
            let buffer_contents = entry.buffer_file.buffer.clone();

            // Clear the buffer after cloning
            entry.buffer_file.clear();

            // Get a cloned file handle (if writing to disk)
            let file_clone = entry.file.as_ref().map(|f| f.try_clone());

            (buffer_contents, file_clone)
        };

        // Use the custom write function if set
        if let Some(custom_fn) = &self.custom_write_fn {
            custom_fn(filename, extension, &buffer)?;
        }

        // Write to the file if a file handle exists
        if let Some(file_result) = file_option {
            let mut file =
                file_result.map_err(|e| anyhow!("Failed to clone file handle: {}", e))?;
            file.write_all(&buffer)
                .map_err(|e| anyhow!("Failed to write to file: {}", e))?;
        }

        Ok(())
    }

    /// Write raw bytes, potentially buffering and flushing if necessary.
    fn write_bytes(&mut self, filename: &str, extension: &str, data: &[u8]) -> Result<()> {
        let mut overflow = data.to_vec();
        while !overflow.is_empty() {
            let leftover = {
                let (entry, _) = self.get_file_entry(filename, extension)?;
                entry.buffer_file.write_bytes(&overflow)
            };
            if leftover.is_empty() {
                break;
            } else {
                // Buffer is full. Flush, then write leftover
                self.flush_buffer(filename, extension)?;
                overflow = leftover;
            }
        }
        Ok(())
    }

    /// Write a string, handling local buffer mode and custom line accumulation.
    pub fn write_string(&mut self, filename: &str, extension: &str, s: &str) -> Result<()> {
        if self.local_mode {
            // Write to local buffer
            self.local_buffer.push_str(s);
            self.local_buffer_pos += s.len();
        } else {
            // Write to file or custom
            self.write_bytes(filename, extension, s.as_bytes())?;
            // Also handle custom line accumulation
            if let Some(ref mut custom_fn) = self.custom_line_fn {
                self.custom_line_buffer.push_str(s);
            }
        }
        Ok(())
    }

    /// Write a character, handling local buffer mode and custom line accumulation.
    pub fn write_char(&mut self, filename: &str, extension: &str, c: char) -> Result<()> {
        let mut buf = [0; 4];
        let cbytes = c.encode_utf8(&mut buf); // Convert char to UTF-8
        if self.local_mode {
            self.local_buffer.push_str(cbytes);
            self.local_buffer_pos += cbytes.len();
        } else {
            self.write_bytes(filename, extension, cbytes.as_bytes())?;
            if let Some(ref mut custom_fn) = self.custom_line_fn {
                self.custom_line_buffer.push_str(cbytes);
            }
        }
        Ok(())
    }

    /// Write a double, formatting it and handling local buffer mode and custom line accumulation.
    pub fn write_double(&mut self, filename: &str, extension: &str, value: f64) -> Result<()> {
        let mut s = String::new();
        write!(&mut s, "{:.2}", value)?; // Format with two decimal places
        self.write_string(filename, extension, &s)?;
        Ok(())
    }

    /// Flush all buffers for all open files, akin to `freeWriteContext` calls to bufferFlush.
    pub fn flush_all(&mut self) -> Result<()> {
        // Clone the keys to avoid holding an immutable borrow while mutably borrowing self
        let keys: Vec<(String, String)> = self.open_files.keys().cloned().collect();

        for (filename, extension) in keys {
            self.flush_buffer(&filename, &extension)
                .map_err(|e| anyhow!("Error flushing {}{}: {}", filename, extension, e))?;

            // After flushing the buffer, flush the actual file if it exists
            if let Some(entry) = self
                .open_files
                .get_mut(&(filename.clone(), extension.clone()))
            {
                if let Some(ref mut file) = entry.file {
                    file.flush().map_err(|e| {
                        anyhow!("Failed to flush file {}{}: {}", filename, extension, e)
                    })?;
                }
            }
        }
        Ok(())
    }

    /// Write a CSV record using the `csv` crate. This automatically handles quotes, commas, etc.
    ///
    /// * `filename`: The base name of the file (no extension). We'll append `.csv`.
    /// * `fields`: A list of string fields to write as one CSV row.
    pub fn write_csv_record(&mut self, filename: &str, fields: &Vec<String>) -> Result<()> {
        let mut buffer = Vec::new();
        {
            let mut wtr = WriterBuilder::new()
                .has_headers(false)
                .from_writer(&mut buffer);
            wtr.write_record(fields)?;
            wtr.flush()?;
        }

        let extension = CSV_EXTENSION;
        if self.local_mode {
            let line = String::from_utf8_lossy(&buffer);
            self.local_buffer.push_str(&line);
            self.local_buffer_pos += line.len();
        } else {
            // Trim the '.' from CSV_EXTENSION when passing to write_bytes
            let trimmed_extension = extension.trim_start_matches('.');
            self.write_bytes(filename, trimmed_extension, &buffer)?;
        }
        Ok(())
    }
}

impl Drop for WriterContext {
    fn drop(&mut self) {
        if let Err(e) = self.flush_all() {
            #[cfg(debug_assertions)]
            panic!("Error during WriterContext drop: {}", e);
            #[cfg(not(debug_assertions))]
            eprintln!("Error during WriterContext drop: {}", e);
        }
    }
}
