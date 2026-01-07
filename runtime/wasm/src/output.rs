//! WASM output types for santa-lang.
//!
//! These types provide a clean, TypeScript-friendly interface with auto-generated
//! type definitions via tsify.

use lang::{Location, ParserErr, RunErr, RunResult, TestCase};
use serde::Serialize;
use tsify_next::Tsify;

// ============================================================
// COMMON TYPES
// ============================================================

/// Execution status.
#[derive(Debug, Clone, Copy, Serialize, Tsify)]
#[tsify(into_wasm_abi)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Pending,
    Running,
    Complete,
    Error,
}

/// Error location with 1-indexed line and column.
#[derive(Debug, Clone, Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct ErrorLocation {
    pub line: u32,
    pub column: u32,
}

/// Stack frame for error traces.
#[derive(Debug, Clone, Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct StackFrame {
    pub function: String,
    pub line: u32,
    pub column: u32,
}

/// Error information with location and stack trace.
#[derive(Debug, Clone, Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct ErrorInfo {
    pub message: String,
    pub location: ErrorLocation,
    pub stack: Vec<StackFrame>,
}

// ============================================================
// SCRIPT STATE
// ============================================================

/// Result of evaluating a script/expression.
#[derive(Debug, Clone, Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct ScriptState {
    #[serde(rename = "type")]
    pub state_type: &'static str,
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
}

impl ScriptState {
    pub fn pending() -> Self {
        Self {
            state_type: "script",
            status: Status::Pending,
            value: None,
            duration_ms: None,
            error: None,
        }
    }

    pub fn running() -> Self {
        Self {
            state_type: "script",
            status: Status::Running,
            value: None,
            duration_ms: None,
            error: None,
        }
    }

    pub fn complete(value: String, duration_ms: u64) -> Self {
        Self {
            state_type: "script",
            status: Status::Complete,
            value: Some(value),
            duration_ms: Some(duration_ms),
            error: None,
        }
    }

    pub fn error(error: ErrorInfo) -> Self {
        Self {
            state_type: "script",
            status: Status::Error,
            value: None,
            duration_ms: None,
            error: Some(error),
        }
    }
}

// ============================================================
// SOLUTION STATE
// ============================================================

/// Result of a single part (part_one or part_two).
#[derive(Debug, Clone, Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct PartResult {
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

impl PartResult {
    pub fn pending() -> Self {
        Self {
            status: Status::Pending,
            value: None,
            duration_ms: None,
        }
    }

    pub fn running() -> Self {
        Self {
            status: Status::Running,
            value: None,
            duration_ms: None,
        }
    }

    pub fn complete(result: &RunResult) -> Self {
        Self {
            status: Status::Complete,
            value: Some(result.value.clone()),
            duration_ms: Some(result.duration as u64),
        }
    }

    pub fn not_present() -> Self {
        Self {
            status: Status::Complete,
            value: None,
            duration_ms: None,
        }
    }
}

/// Result of evaluating a solution (part_one/part_two).
#[derive(Debug, Clone, Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct SolutionState {
    #[serde(rename = "type")]
    pub state_type: &'static str,
    pub status: Status,
    pub part_one: PartResult,
    pub part_two: PartResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
}

impl SolutionState {
    pub fn pending() -> Self {
        Self {
            state_type: "solution",
            status: Status::Pending,
            part_one: PartResult::pending(),
            part_two: PartResult::pending(),
            error: None,
        }
    }

    pub fn running() -> Self {
        Self {
            state_type: "solution",
            status: Status::Running,
            part_one: PartResult::pending(),
            part_two: PartResult::pending(),
            error: None,
        }
    }

    pub fn error(error: ErrorInfo) -> Self {
        Self {
            state_type: "solution",
            status: Status::Error,
            part_one: PartResult::pending(),
            part_two: PartResult::pending(),
            error: Some(error),
        }
    }
}

// ============================================================
// TEST STATE
// ============================================================

/// Result of a test part comparison.
#[derive(Debug, Clone, Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct TestPartResult {
    pub passed: bool,
    pub expected: String,
    pub actual: String,
}

impl From<&lang::TestCaseResult> for TestPartResult {
    fn from(result: &lang::TestCaseResult) -> Self {
        Self {
            passed: result.passed,
            expected: result.expected.clone(),
            actual: result.actual.clone(),
        }
    }
}

/// Status of an individual test case.
#[derive(Debug, Clone, Copy, Serialize, Tsify)]
#[tsify(into_wasm_abi)]
#[serde(rename_all = "lowercase")]
pub enum TestCaseStatus {
    Pending,
    Running,
    Complete,
    Skipped,
}

/// Individual test case result.
#[derive(Debug, Clone, Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct TestCaseState {
    pub index: u32,
    pub slow: bool,
    pub status: TestCaseStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_one: Option<TestPartResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_two: Option<TestPartResult>,
}

impl TestCaseState {
    pub fn pending(index: u32) -> Self {
        Self {
            index,
            slow: false,
            status: TestCaseStatus::Pending,
            part_one: None,
            part_two: None,
        }
    }

    pub fn from_result(index: u32, test_case: &TestCase) -> Self {
        if test_case.skipped {
            return Self {
                index,
                slow: test_case.slow,
                status: TestCaseStatus::Skipped,
                part_one: None,
                part_two: None,
            };
        }

        Self {
            index,
            slow: test_case.slow,
            status: TestCaseStatus::Complete,
            part_one: test_case.part_one.as_ref().map(TestPartResult::from),
            part_two: test_case.part_two.as_ref().map(TestPartResult::from),
        }
    }
}

/// Summary of test results.
#[derive(Debug, Clone, Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct TestSummary {
    pub total: u32,
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
}

impl TestSummary {
    pub fn empty(total: u32) -> Self {
        Self {
            total,
            passed: 0,
            failed: 0,
            skipped: 0,
        }
    }

    pub fn from_test_cases(test_cases: &[TestCase]) -> Self {
        let mut passed = 0u32;
        let mut failed = 0u32;
        let mut skipped = 0u32;

        for tc in test_cases {
            if tc.skipped {
                skipped += 1;
            } else {
                let part_one_passed = tc.part_one.as_ref().is_none_or(|p| p.passed);
                let part_two_passed = tc.part_two.as_ref().is_none_or(|p| p.passed);
                if part_one_passed && part_two_passed {
                    passed += 1;
                } else {
                    failed += 1;
                }
            }
        }

        Self {
            total: test_cases.len() as u32,
            passed,
            failed,
            skipped,
        }
    }
}

/// Result of running test cases.
#[derive(Debug, Clone, Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct TestState {
    #[serde(rename = "type")]
    pub state_type: &'static str,
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success: Option<bool>,
    pub summary: TestSummary,
    pub tests: Vec<TestCaseState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
}

impl TestState {
    pub fn pending(test_count: u32) -> Self {
        Self {
            state_type: "test",
            status: Status::Pending,
            success: None,
            summary: TestSummary::empty(test_count),
            tests: (1..=test_count).map(TestCaseState::pending).collect(),
            error: None,
        }
    }

    pub fn running(test_count: u32) -> Self {
        Self {
            state_type: "test",
            status: Status::Running,
            success: None,
            summary: TestSummary::empty(test_count),
            tests: (1..=test_count).map(TestCaseState::pending).collect(),
            error: None,
        }
    }

    pub fn complete(test_cases: &[TestCase]) -> Self {
        let summary = TestSummary::from_test_cases(test_cases);
        Self {
            state_type: "test",
            status: Status::Complete,
            success: Some(summary.failed == 0),
            summary,
            tests: test_cases
                .iter()
                .enumerate()
                .map(|(i, tc)| TestCaseState::from_result((i + 1) as u32, tc))
                .collect(),
            error: None,
        }
    }

    pub fn error(error: ErrorInfo) -> Self {
        Self {
            state_type: "test",
            status: Status::Error,
            success: None,
            summary: TestSummary::empty(0),
            tests: vec![],
            error: Some(error),
        }
    }
}

// ============================================================
// FORMAT RESULT
// ============================================================

/// Result of formatting source code.
#[derive(Debug, Clone, Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct FormatResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatted: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
}

impl FormatResult {
    pub fn success(formatted: String) -> Self {
        Self {
            success: true,
            formatted: Some(formatted),
            error: None,
        }
    }

    pub fn error(error: ErrorInfo) -> Self {
        Self {
            success: false,
            formatted: None,
            error: Some(error),
        }
    }
}

// ============================================================
// UTILITY FUNCTIONS
// ============================================================

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

    (line, column)
}

/// Extract function name from source location.
fn extract_function_name(source: &str, location: Location) -> String {
    let text = if location.end <= source.len() {
        &source[location.start..location.end]
    } else if location.start < source.len() {
        &source[location.start..]
    } else {
        return "<top-level>".to_string();
    };

    let trimmed = text.trim();
    if trimmed.is_empty() {
        return "<top-level>".to_string();
    }

    if trimmed.starts_with('|') {
        return "<lambda>".to_string();
    }

    let name: String = trimmed
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();

    if name.is_empty() { "<lambda>".to_string() } else { name }
}

/// Convert a RunErr to ErrorInfo with proper line/column info.
pub fn format_error(source: &str, error: &RunErr) -> ErrorInfo {
    let (line, column) = calculate_line_column(source, error.source);

    let stack: Vec<StackFrame> = error
        .trace
        .iter()
        .map(|loc| {
            let (frame_line, frame_column) = calculate_line_column(source, *loc);
            let func_name = extract_function_name(source, *loc);
            StackFrame {
                function: func_name,
                line: frame_line,
                column: frame_column,
            }
        })
        .collect();

    ErrorInfo {
        message: error.message.clone(),
        location: ErrorLocation { line, column },
        stack,
    }
}

/// Convert a ParserErr to ErrorInfo.
pub fn format_parser_error(source: &str, error: &ParserErr) -> ErrorInfo {
    let (line, column) = calculate_line_column(source, error.source);

    ErrorInfo {
        message: error.message.clone(),
        location: ErrorLocation { line, column },
        stack: vec![],
    }
}
