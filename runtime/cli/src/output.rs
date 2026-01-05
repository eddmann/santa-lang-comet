//! CLI output formatting for JSON and JSONL modes.
//!
//! This module implements the CLI Output Format Specification (Section 16 of lang.txt).
//! It provides machine-readable output formats for integration with editors, CI systems,
//! and other tools.

use santa_lang::{Location, RunErr, RunEvaluation, TestCase};
use serde::Serialize;
use std::io::{self, Write};

/// Output mode for CLI execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    /// Human-readable output with ANSI colors (default)
    Text,
    /// Single JSON object after execution completes
    Json,
    /// Real-time streaming with JSON Lines
    Jsonl,
}

/// Console output entry from puts() calls.
#[derive(Debug, Clone, Serialize)]
pub struct ConsoleEntry {
    pub timestamp_ms: u64,
    pub message: String,
}

/// Error location with 1-indexed line and column.
#[derive(Debug, Clone, Serialize)]
pub struct ErrorLocation {
    pub line: u32,
    pub column: u32,
}

/// Stack frame for error traces.
#[derive(Debug, Clone, Serialize)]
pub struct StackFrame {
    pub function: String,
    pub line: u32,
    pub column: u32,
}

/// Part result for JSON output.
#[derive(Debug, Clone, Serialize)]
pub struct JsonPartResult {
    pub status: &'static str,
    pub value: String,
    pub duration_ms: u64,
}

/// Test part result for JSON output.
#[derive(Debug, Clone, Serialize)]
pub struct JsonTestPartResult {
    pub passed: bool,
    pub expected: String,
    pub actual: String,
}

/// Test case for JSON output.
#[derive(Debug, Clone, Serialize)]
pub struct JsonTestCase {
    pub index: u32,
    pub slow: bool,
    pub status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_one: Option<JsonTestPartResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_two: Option<JsonTestPartResult>,
}

/// Test summary counts.
#[derive(Debug, Clone, Serialize)]
pub struct TestSummary {
    pub total: u32,
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
}

/// JSON output for solution execution.
#[derive(Debug, Clone, Serialize)]
pub struct JsonSolutionOutput {
    #[serde(rename = "type")]
    pub output_type: &'static str,
    pub status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_one: Option<JsonPartResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_two: Option<JsonPartResult>,
    pub console: Vec<ConsoleEntry>,
}

/// JSON output for script execution.
#[derive(Debug, Clone, Serialize)]
pub struct JsonScriptOutput {
    #[serde(rename = "type")]
    pub output_type: &'static str,
    pub status: &'static str,
    pub value: String,
    pub duration_ms: u64,
    pub console: Vec<ConsoleEntry>,
}

/// JSON output for test execution.
#[derive(Debug, Clone, Serialize)]
pub struct JsonTestOutput {
    #[serde(rename = "type")]
    pub output_type: &'static str,
    pub status: &'static str,
    pub success: bool,
    pub summary: TestSummary,
    pub tests: Vec<JsonTestCase>,
    pub console: Vec<ConsoleEntry>,
}

/// JSON output for errors.
#[derive(Debug, Clone, Serialize)]
pub struct JsonErrorOutput {
    #[serde(rename = "type")]
    pub output_type: &'static str,
    pub message: String,
    pub location: ErrorLocation,
    pub stack: Vec<StackFrame>,
}

/// Calculate 1-indexed line and column from byte offset.
pub fn calculate_line_column(source: &str, location: Location) -> (u32, u32) {
    let mut line: u32 = 1;
    let mut column: u32 = 1;

    for (position, character) in source.chars().enumerate() {
        if position == location.start {
            return (line, column);
        }

        if character == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }

    // Location is at or beyond end of source (e.g., EOF error)
    (line, column)
}

/// Format a RunErr as JSON error output.
pub fn format_error_json(source: &str, error: &RunErr) -> JsonErrorOutput {
    let (line, column) = calculate_line_column(source, error.source);

    let stack: Vec<StackFrame> = error
        .trace
        .iter()
        .map(|loc| {
            let (frame_line, frame_column) = calculate_line_column(source, *loc);
            // Extract function name from source - look for identifier before the call
            let func_name = extract_function_name(source, *loc);
            StackFrame {
                function: func_name,
                line: frame_line,
                column: frame_column,
            }
        })
        .collect();

    JsonErrorOutput {
        output_type: "error",
        message: error.message.clone(),
        location: ErrorLocation { line, column },
        stack,
    }
}

/// Extract function name from source location.
/// Returns "<lambda>" for anonymous functions, "<top-level>" for top-level code,
/// or the actual function name if available.
fn extract_function_name(source: &str, location: Location) -> String {
    // Get the text at the location
    let text = if location.end <= source.len() {
        &source[location.start..location.end]
    } else if location.start < source.len() {
        &source[location.start..]
    } else {
        return "<top-level>".to_string();
    };

    // Try to extract the function name (first identifier-like word)
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return "<top-level>".to_string();
    }

    // Check for lambda/closure patterns
    if trimmed.starts_with('|') {
        return "<lambda>".to_string();
    }

    // Extract first identifier
    let name: String = trimmed
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();

    if name.is_empty() {
        "<lambda>".to_string()
    } else {
        name
    }
}

/// Format run evaluation result as JSON.
pub fn format_run_json(result: &RunEvaluation, console: Vec<ConsoleEntry>) -> String {
    match result {
        RunEvaluation::Solution { part_one, part_two } => {
            let output = JsonSolutionOutput {
                output_type: "solution",
                status: "complete",
                part_one: part_one.as_ref().map(|p| JsonPartResult {
                    status: "complete",
                    value: p.value.clone(),
                    duration_ms: p.duration as u64,
                }),
                part_two: part_two.as_ref().map(|p| JsonPartResult {
                    status: "complete",
                    value: p.value.clone(),
                    duration_ms: p.duration as u64,
                }),
                console,
            };
            serde_json::to_string(&output).unwrap()
        }
        RunEvaluation::Script(result) => {
            let output = JsonScriptOutput {
                output_type: "script",
                status: "complete",
                value: result.value.clone(),
                duration_ms: result.duration as u64,
                console,
            };
            serde_json::to_string(&output).unwrap()
        }
    }
}

/// Format test results as JSON.
pub fn format_test_json(
    test_cases: &[TestCase],
    has_part_one: bool,
    has_part_two: bool,
    console: Vec<ConsoleEntry>,
) -> String {
    let mut passed = 0u32;
    let mut failed = 0u32;
    let mut skipped = 0u32;

    let tests: Vec<JsonTestCase> = test_cases
        .iter()
        .enumerate()
        .map(|(i, tc)| {
            if tc.skipped {
                skipped += 1;
                JsonTestCase {
                    index: (i + 1) as u32,
                    slow: tc.slow,
                    status: "skipped",
                    part_one: None,
                    part_two: None,
                }
            } else {
                // Determine if test passed (all parts must pass)
                let part_one_passed = tc.part_one.as_ref().is_none_or(|p| p.passed);
                let part_two_passed = tc.part_two.as_ref().is_none_or(|p| p.passed);
                let all_passed = part_one_passed && part_two_passed;

                if all_passed {
                    passed += 1;
                } else {
                    failed += 1;
                }

                JsonTestCase {
                    index: (i + 1) as u32,
                    slow: tc.slow,
                    status: "complete",
                    part_one: if has_part_one {
                        tc.part_one.as_ref().map(|p| JsonTestPartResult {
                            passed: p.passed,
                            expected: p.expected.clone(),
                            actual: p.actual.clone(),
                        })
                    } else {
                        None
                    },
                    part_two: if has_part_two {
                        tc.part_two.as_ref().map(|p| JsonTestPartResult {
                            passed: p.passed,
                            expected: p.expected.clone(),
                            actual: p.actual.clone(),
                        })
                    } else {
                        None
                    },
                }
            }
        })
        .collect();

    let total = test_cases.len() as u32;
    let success = failed == 0;

    let output = JsonTestOutput {
        output_type: "test",
        status: "complete",
        success,
        summary: TestSummary {
            total,
            passed,
            failed,
            skipped,
        },
        tests,
        console,
    };

    serde_json::to_string(&output).unwrap()
}

// ============================================================================
// JSONL Streaming Support
// ============================================================================

/// JSONL patch operation per RFC 6902.
#[derive(Debug, Clone, Serialize)]
pub struct JsonPatch {
    pub op: &'static str,
    pub path: String,
    pub value: serde_json::Value,
}

/// Initial state for JSONL solution streaming.
#[derive(Debug, Clone, Serialize)]
pub struct JsonlSolutionInitial {
    #[serde(rename = "type")]
    pub output_type: &'static str,
    pub status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_one: Option<JsonlPartInitial>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_two: Option<JsonlPartInitial>,
    pub console: Vec<ConsoleEntry>,
}

/// Initial state for a part in JSONL streaming.
#[derive(Debug, Clone, Serialize)]
pub struct JsonlPartInitial {
    pub status: &'static str,
    pub value: Option<String>,
    pub duration_ms: Option<u64>,
}

/// Initial state for JSONL script streaming.
#[derive(Debug, Clone, Serialize)]
pub struct JsonlScriptInitial {
    #[serde(rename = "type")]
    pub output_type: &'static str,
    pub status: &'static str,
    pub value: Option<String>,
    pub duration_ms: Option<u64>,
    pub console: Vec<ConsoleEntry>,
}

/// Initial state for JSONL test streaming.
#[derive(Debug, Clone, Serialize)]
pub struct JsonlTestInitial {
    #[serde(rename = "type")]
    pub output_type: &'static str,
    pub status: &'static str,
    pub success: Option<bool>,
    pub summary: TestSummary,
    pub tests: Vec<JsonlTestCaseInitial>,
    pub console: Vec<ConsoleEntry>,
}

/// Initial test case state for JSONL streaming.
#[derive(Debug, Clone, Serialize)]
pub struct JsonlTestCaseInitial {
    pub index: u32,
    pub slow: bool,
    pub status: &'static str,
    pub part_one: Option<JsonTestPartResult>,
    pub part_two: Option<JsonTestPartResult>,
}

/// JSONL streaming writer.
pub struct JsonlWriter<W: Write> {
    writer: W,
}

impl<W: Write> JsonlWriter<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Write initial state line.
    pub fn write_initial<T: Serialize>(&mut self, state: &T) -> io::Result<()> {
        let json = serde_json::to_string(state)?;
        writeln!(self.writer, "{}", json)?;
        self.writer.flush()
    }

    /// Write a patch array.
    pub fn write_patches(&mut self, patches: &[JsonPatch]) -> io::Result<()> {
        let json = serde_json::to_string(patches)?;
        writeln!(self.writer, "{}", json)?;
        self.writer.flush()
    }

    /// Create a replace patch.
    pub fn replace_patch(path: &str, value: impl Serialize) -> JsonPatch {
        JsonPatch {
            op: "replace",
            path: path.to_string(),
            value: serde_json::to_value(value).unwrap(),
        }
    }

    /// Create an add patch (for appending to arrays).
    pub fn add_patch(path: &str, value: impl Serialize) -> JsonPatch {
        JsonPatch {
            op: "add",
            path: path.to_string(),
            value: serde_json::to_value(value).unwrap(),
        }
    }
}

/// Determine if source is a solution (has part_one/part_two) or script.
pub fn is_solution_source(source: &str) -> (bool, bool) {
    // Simple heuristic: check if source contains part_one: or part_two:
    // This matches the runner's behavior
    let has_part_one = source.contains("part_one:");
    let has_part_two = source.contains("part_two:");
    (has_part_one, has_part_two)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_line_column() {
        let source = "line1\nline2\nline3";
        // Position 0 = 'l' at line 1, col 1
        assert_eq!(calculate_line_column(source, Location { start: 0, end: 1 }), (1, 1));
        // Position 5 = '\n' at line 1, col 6
        assert_eq!(calculate_line_column(source, Location { start: 5, end: 6 }), (1, 6));
        // Position 6 = 'l' at line 2, col 1
        assert_eq!(calculate_line_column(source, Location { start: 6, end: 7 }), (2, 1));
        // Position 12 = 'l' at line 3, col 1
        assert_eq!(
            calculate_line_column(source, Location { start: 12, end: 13 }),
            (3, 1)
        );
    }

    #[test]
    fn test_extract_function_name_lambda() {
        let source = "|x| x + 1";
        let name = extract_function_name(source, Location { start: 0, end: 9 });
        assert_eq!(name, "<lambda>");
    }

    #[test]
    fn test_extract_function_name_identifier() {
        let source = "calculate(x)";
        let name = extract_function_name(source, Location { start: 0, end: 9 });
        assert_eq!(name, "calculate");
    }

    #[test]
    fn test_is_solution_source() {
        assert_eq!(is_solution_source("part_one: { 42 }"), (true, false));
        assert_eq!(is_solution_source("part_two: { 42 }"), (false, true));
        assert_eq!(
            is_solution_source("part_one: { 1 }\npart_two: { 2 }"),
            (true, true)
        );
        assert_eq!(is_solution_source("1 + 2"), (false, false));
    }
}
