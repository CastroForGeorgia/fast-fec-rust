extern crate fast_fec_rust;

use anyhow::Result;
use fast_fec_rust::writer::WriterContext;
use std::sync::{Arc, Mutex};

#[cfg(test)]
mod tests {
    use super::*;

    /// A structure to capture file and line outputs during tests.
    struct TestOutput {
        file_output: String,
        line_output: String,
    }

    fn reset_output() -> Arc<Mutex<TestOutput>> {
        Arc::new(Mutex::new(TestOutput {
            file_output: String::new(),
            line_output: String::new(),
        }))
    }

    #[test]
    fn test_writer() -> Result<()> {
        let test_output = reset_output();

        let to_file = {
            let test_output = Arc::clone(&test_output);
            move |_: &str, _: &str, contents: &[u8]| -> Result<()> {
                let mut out = test_output.lock().unwrap();
                for &b in contents {
                    out.file_output.push(b as char);
                }
                Ok(())
            }
        };

        let to_line = {
            let test_output = Arc::clone(&test_output);
            move |_: &str, line: &str, _: &str| -> Result<()> {
                let mut out = test_output.lock().unwrap();
                out.line_output.clear();
                out.line_output.push_str(line);
                Ok(())
            }
        };

        let mut ctx = WriterContext::new(
            "".into(),
            "".into(),
            false,
            3,
            Some(Box::new(to_file)),
            Some(Box::new(to_line)),
        );

        // Write partial string
        ctx.write_string("test", ".txt", "hi")?;
        assert_eq!(test_output.lock().unwrap().file_output, "");
        assert_eq!(test_output.lock().unwrap().line_output, "");

        // Write more to overflow the buffer
        ctx.write_string("test", ".txt", " there")?;
        assert_eq!(test_output.lock().unwrap().file_output, "hi the");
        assert_eq!(test_output.lock().unwrap().line_output, "");

        // Write newline and flush
        ctx.write_char("test", ".txt", '\n')?;
        ctx.end_line("")?;
        ctx.flush_all()?;

        let out = test_output.lock().unwrap();
        assert_eq!(out.file_output, "hi there\n");
        assert_eq!(out.line_output, "hi there\n");

        Ok(())
    }

    #[test]
    fn test_writer_end_on_buffer_size() -> Result<()> {
        let test_output = reset_output();

        let to_file = {
            let test_output = Arc::clone(&test_output);
            move |_: &str, _: &str, contents: &[u8]| -> Result<()> {
                let mut out = test_output.lock().unwrap();
                for &b in contents {
                    out.file_output.push(b as char);
                }
                Ok(())
            }
        };

        let to_line = {
            let test_output = Arc::clone(&test_output);
            move |_: &str, line: &str, _: &str| -> Result<()> {
                let mut out = test_output.lock().unwrap();
                out.line_output.clear();
                out.line_output.push_str(line);
                Ok(())
            }
        };

        let mut ctx = WriterContext::new(
            "".into(),
            "".into(),
            false,
            3, // Buffer size remains 3
            Some(Box::new(to_file)),
            Some(Box::new(to_line)),
        );

        ctx.write_string("test", ".txt", "hi")?;
        assert_eq!(test_output.lock().unwrap().file_output, ""); // No flush yet

        ctx.write_string("test", ".txt", " there!")?;
        assert_eq!(test_output.lock().unwrap().file_output, "hi the"); // Partial flush

        ctx.write_char("test", ".txt", '\n')?;
        ctx.end_line("")?;
        ctx.flush_all()?; // Ensure all data is flushed

        let out = test_output.lock().unwrap();
        assert_eq!(out.file_output, "hi there!\n"); // After full flush
        assert_eq!(out.line_output, "hi there!\n");

        Ok(())
    }

    #[test]
    fn test_writer_massive_buffer() -> Result<()> {
        let test_output = reset_output();

        let to_file = {
            let test_output = Arc::clone(&test_output);
            move |_: &str, _: &str, contents: &[u8]| -> Result<()> {
                let mut out = test_output.lock().unwrap();
                for &b in contents {
                    out.file_output.push(b as char);
                }
                Ok(())
            }
        };

        let to_line = {
            let test_output = Arc::clone(&test_output);
            move |_: &str, line: &str, _: &str| -> Result<()> {
                let mut out = test_output.lock().unwrap();
                out.line_output.clear();
                out.line_output.push_str(line);
                Ok(())
            }
        };

        let mut ctx = WriterContext::new(
            "".into(),
            "".into(),
            false,
            300,
            Some(Box::new(to_file)),
            Some(Box::new(to_line)),
        );

        ctx.write_string("test", ".txt", "hi")?;
        assert_eq!(test_output.lock().unwrap().file_output, "");

        ctx.write_string("test", ".txt", " there!")?;
        assert_eq!(test_output.lock().unwrap().file_output, "");

        ctx.write_char("test", ".txt", '\n')?;
        ctx.end_line("")?;
        ctx.flush_all()?;

        let out = test_output.lock().unwrap();
        assert_eq!(out.file_output, "hi there!\n");
        assert_eq!(out.line_output, "hi there!\n");

        Ok(())
    }

    #[test]
    fn test_line_buffer() -> Result<()> {
        let test_output = reset_output();

        let to_file = {
            let test_output = Arc::clone(&test_output);
            move |_: &str, _: &str, _: &[u8]| -> Result<()> { Ok(()) }
        };

        let to_line = {
            let test_output = Arc::clone(&test_output);
            move |_: &str, line: &str, _: &str| -> Result<()> {
                let mut out = test_output.lock().unwrap();
                out.line_output.clear();
                out.line_output.push_str(line);
                Ok(())
            }
        };

        let mut ctx = WriterContext::new(
            "".into(),
            "".into(),
            false,
            300,
            Some(Box::new(to_file)),
            Some(Box::new(to_line)),
        );

        ctx.write_string("test", ".txt", "hi there\n")?;
        ctx.end_line("")?;
        assert_eq!(test_output.lock().unwrap().line_output, "hi there\n");

        ctx.write_string("test", ".txt", "how are you today?\n")?;
        ctx.end_line("")?;
        assert_eq!(
            test_output.lock().unwrap().line_output,
            "how are you today?\n"
        );

        Ok(())
    }
}
