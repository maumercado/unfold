//! Structured parse error handling for better error display.
//!
//! Provides detailed error information including line numbers and context.

/// Structured parse error for better error display
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub context_line: Option<String>,  // The actual line from the file
    pub filename: String,
}

impl ParseError {
    /// Create a ParseError from a serde_json error
    pub fn from_serde_error(e: &serde_json::Error, contents: &str, filename: &str) -> Self {
        let line = e.line();
        let column = e.column();

        // Extract the problematic line from the file contents
        let context_line = contents
            .lines()
            .nth(line.saturating_sub(1))
            .map(|s| s.to_string());

        // Classify the error for a friendlier message
        let message = match e.classify() {
            serde_json::error::Category::Io => format!("I/O error: {}", e),
            serde_json::error::Category::Syntax => {
                // Extract just the syntax error description
                let full = e.to_string();
                // serde_json format: "message at line X column Y"
                if let Some(idx) = full.find(" at line ") {
                    full[..idx].to_string()
                } else {
                    full
                }
            }
            serde_json::error::Category::Data => format!("Data error: {}", e),
            serde_json::error::Category::Eof => "Unexpected end of file".to_string(),
        };

        ParseError {
            message,
            line: Some(line),
            column: Some(column),
            context_line,
            filename: filename.to_string(),
        }
    }

}
