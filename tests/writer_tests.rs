extern crate fast_fec_rust;

use fast_fec_rust::writer::WriterContext;
use std::fmt::Write as FmtWrite;
use std::sync::{Arc, Mutex};

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    struct TestOutput {
        file_output: String,
        line_output: String,
    }

    #[test]
    fn test_writer() -> Result<()> {
        let test_output = Arc::new(Mutex::new(TestOutput {
            file_output: String::new(),
            line_output: String::new(),
        }));

        let to_file = {
            let test_output = Arc::clone(&test_output);
            move |_: &str, _: &str, contents: &[u8]| -> Result<()> {
                let mut out = test_output.lock().unwrap(); // Use `lock()` to access the data
                for &b in contents {
                    out.file_output.push(b as char);
                }
                Ok(())
            }
        };

        let to_line = {
            let test_output = Arc::clone(&test_output);
            move |_: &str, line: &str, _: &str| -> Result<()> {
                let mut out = test_output.lock().unwrap(); // Use `lock()` to access the data
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

        ctx.write_string("test", ".txt", "hi")?;
        {
            let out = test_output.lock().unwrap();
            assert_eq!(out.file_output, "");
            assert_eq!(out.line_output, "");
        }

        ctx.write_string("test", ".txt", " there")?;
        {
            let out = test_output.lock().unwrap();
            assert_eq!(out.file_output, "hi the");
            assert_eq!(out.line_output, "");
        }

        ctx.write_char("test", ".txt", '\n')?;
        ctx.end_line("")?;

        ctx.flush_all()?;

        {
            let out = test_output.lock().unwrap();
            assert_eq!(out.file_output, "hi there\n");
            assert_eq!(out.line_output, "hi there\n");
        }

        Ok(())
    }

    #[test]
    fn test_writer_end_on_buffer_size() -> Result<()> {
        let test_output = Arc::new(Mutex::new(TestOutput {
            file_output: String::new(),
            line_output: String::new(),
        }));

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

        ctx.write_string("test", ".txt", "hi")?;
        {
            let out = test_output.lock().unwrap();
            assert_eq!(out.file_output, "");
            assert_eq!(out.line_output, "");
        }

        ctx.write_string("test", ".txt", " there!")?;
        {
            let out = test_output.lock().unwrap();
            assert_eq!(out.file_output, "hi there!");
            assert_eq!(out.line_output, "");
        }

        ctx.write_char("test", ".txt", '\n')?;
        ctx.end_line("")?;
        ctx.flush_all()?;

        {
            let out = test_output.lock().unwrap();
            assert_eq!(out.file_output, "hi there!\n");
            assert_eq!(out.line_output, "hi there!\n");
        }

        Ok(())
    }

    #[test]
    fn test_writer_massive_buffer() -> Result<()> {
        let test_output = Arc::new(Mutex::new(TestOutput {
            file_output: String::new(),
            line_output: String::new(),
        }));

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
        {
            let out = test_output.lock().unwrap();
            assert_eq!(out.file_output, "");
            assert_eq!(out.line_output, "");
        }

        ctx.write_string("test", ".txt", " there!")?;
        {
            let out = test_output.lock().unwrap();
            assert_eq!(out.file_output, "");
            assert_eq!(out.line_output, "");
        }

        ctx.write_char("test", ".txt", '\n')?;
        ctx.end_line("")?;
        ctx.flush_all()?;

        {
            let out = test_output.lock().unwrap();
            assert_eq!(out.file_output, "hi there!\n");
            assert_eq!(out.line_output, "hi there!\n");
        }

        Ok(())
    }
}
