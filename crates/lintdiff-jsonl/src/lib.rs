//! JSON Lines parsing for lintdiff.
//!
//! Provides efficient parsing of JSONL (JSON Lines) format,
//! commonly used in diagnostic tool output.

use std::io::{BufRead, BufReader, Cursor, Read};
use thiserror::Error;

/// Parse JSON Lines from a reader.
///
/// # Examples
/// ```
/// use lintdiff_jsonl::JsonlParser;
/// use serde_json::json;
///
/// let data = "{\"type\":\"diagnostic\",\"message\":\"error\"}\n{\"type\":\"note\",\"message\":\"info\"}";
/// let mut parser = JsonlParser::from_string(data);
///
/// while let Some(value) = parser.next()? {
///     println!("{:?}", value);
/// }
/// # Ok::<(), lintdiff_jsonl::JsonlError>(())
/// ```
pub struct JsonlParser<'a> {
    inner: BufReader<Box<dyn Read + 'a>>,
    exhausted: bool,
}

impl<'a> JsonlParser<'a> {
    /// Create a new parser from a reader.
    pub fn new<R: Read + 'a>(reader: R) -> Self {
        Self {
            inner: BufReader::new(Box::new(reader)),
            exhausted: false,
        }
    }

    /// Create a parser from a string.
    #[must_use]
    pub fn from_string(s: &'a str) -> Self {
        Self::new(Cursor::new(s))
    }

    /// Parse the next JSON object.
    ///
    /// # Errors
    /// Returns an error if parsing fails.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<Option<serde_json::Value>, JsonlError> {
        if self.exhausted {
            return Ok(None);
        }

        let mut line = String::new();
        let bytes_read = self
            .inner
            .read_line(&mut line)
            .map_err(|e| JsonlError::Io {
                message: e.to_string(),
            })?;

        if bytes_read == 0 {
            self.exhausted = true;
            return Ok(None);
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            // Skip empty lines but continue reading
            return self.next();
        }

        match serde_json::from_str(trimmed) {
            Ok(value) => Ok(Some(value)),
            Err(e) => Err(JsonlError::Parse {
                line: trimmed.to_string(),
                details: e.to_string(),
            }),
        }
    }

    /// Collect all remaining objects.
    ///
    /// # Errors
    /// Returns an error if parsing fails.
    pub fn collect(&mut self) -> Result<Vec<serde_json::Value>, JsonlError> {
        let mut results = Vec::new();

        while let Some(value) = self.next()? {
            results.push(value);
        }

        Ok(results)
    }

    /// Parse all objects from a string.
    ///
    /// # Errors
    /// Returns an error if parsing fails.
    pub fn parse(s: &'a str) -> Result<Vec<serde_json::Value>, JsonlError> {
        let mut parser = Self::from_string(s);
        parser.collect()
    }
}

/// Error when parsing JSONL.
#[derive(Debug, Clone, Error)]
pub enum JsonlError {
    /// Failed to parse a JSON line.
    #[error("Failed to parse line: {line}: {details}")]
    Parse {
        /// The line that failed to parse.
        line: String,
        /// The error details.
        details: String,
    },
    /// I/O error occurred.
    #[error("I/O error: {message}")]
    Io {
        /// The error message.
        message: String,
    },
    /// Serialization error.
    #[error("Serialization error: {details}")]
    Serialize {
        /// The error details.
        details: String,
    },
}

/// A builder for creating JSONL output.
#[derive(Default)]
pub struct JsonlBuilder {
    buffer: Vec<u8>,
}

impl JsonlBuilder {
    /// Create a new builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a JSON value to the builder.
    ///
    /// # Errors
    /// Returns an error if serialization fails.
    pub fn push(&mut self, value: &serde_json::Value) -> Result<(), JsonlError> {
        let mut line = serde_json::to_string(value).map_err(|e| JsonlError::Serialize {
            details: e.to_string(),
        })?;
        line.push('\n');
        self.buffer.extend(line.into_bytes());
        Ok(())
    }

    /// Build the final JSONL string.
    #[must_use]
    pub fn build(self) -> String {
        String::from_utf8_lossy(&self.buffer).into_owned()
    }

    /// Get the current buffer length.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if the buffer is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

/// Parse a JSONL string into a vector of values.
///
/// # Errors
/// Returns an error if parsing fails.
///
/// # Examples
/// ```
/// use lintdiff_jsonl::parse_jsonl;
///
/// let data = "{\"a\":1}\n{\"b\":2}";
/// let values = parse_jsonl(data)?;
/// assert_eq!(values.len(), 2);
/// # Ok::<(), lintdiff_jsonl::JsonlError>(())
/// ```
pub fn parse_jsonl(s: &str) -> Result<Vec<serde_json::Value>, JsonlError> {
    let mut parser = JsonlParser::from_string(s);
    parser.collect()
}

/// Convert a vector of values to a JSONL string.
///
/// # Errors
/// Returns an error if serialization fails.
///
/// # Examples
/// ```
/// use lintdiff_jsonl::to_jsonl;
/// use serde_json::json;
///
/// let values = vec![json!({"a": 1}), json!({"b": 2})];
/// let jsonl = to_jsonl(&values)?;
/// assert!(jsonl.contains("{\"a\":1}"));
/// # Ok::<(), lintdiff_jsonl::JsonlError>(())
/// ```
pub fn to_jsonl(values: &[serde_json::Value]) -> Result<String, JsonlError> {
    let mut builder = JsonlBuilder::new();
    for value in values {
        builder.push(value)?;
    }
    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_empty_string() {
        let mut parser = JsonlParser::from_string("");
        let result = parser.next().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_single_object() {
        let mut parser = JsonlParser::from_string("{\"a\":1}");
        let result = parser.next().unwrap();
        assert_eq!(result, Some(json!({"a": 1})));
    }

    #[test]
    fn test_builder_empty() {
        let builder = JsonlBuilder::new();
        assert!(builder.is_empty());
        assert_eq!(builder.len(), 0);
    }
}
