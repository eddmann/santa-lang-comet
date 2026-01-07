//! santa-lang WebAssembly runtime.
//!
//! This module provides a clean API for evaluating santa-lang code from JavaScript/TypeScript.
//!
//! ## API
//!
//! - `evaluateScript(source, options?)` - Evaluate a script/expression
//! - `evaluateSolution(source, options?)` - Evaluate a solution (part_one/part_two)
//! - `testSolution(source, options?)` - Run test cases
//! - `format(source)` - Format source code
//! - `isFormatted(source)` - Check if source is formatted

mod external_functions;
mod output;

use js_sys::{Function, Object};
use lang::{AoCRunner, RunEvaluation, Time};
use output::{FormatResult, PartResult, ScriptState, SolutionState, Status, TestState};
use wasm_bindgen::prelude::{JsValue, wasm_bindgen};

#[cfg(test)]
mod tests;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(thread_local_v2, js_namespace = globalThis, js_name = performance)]
    pub static JS_PERFORMANCE: web_sys::Performance;
}

struct WebTime {}
impl Time for WebTime {
    fn now(&self) -> u128 {
        JS_PERFORMANCE.with(|perf| perf.now() as u128)
    }
}

/// Emit a progress callback with the current state.
fn emit_progress<T: serde::Serialize>(on_progress: &Option<Function>, state: &T) {
    if let Some(callback) = on_progress {
        if let Ok(value) = serde_wasm_bindgen::to_value(state) {
            let _ = callback.call1(&JsValue::NULL, &value);
        }
    }
}

// ============================================================
// EVALUATE SCRIPT
// ============================================================

/// Evaluate a script/expression.
///
/// Returns a `ScriptState` with the result value and duration.
///
/// ## Options
/// - `externalFunctions`: Object mapping function names to JavaScript functions
/// - `onProgress`: Callback invoked with state updates
#[wasm_bindgen(js_name = evaluateScript)]
pub fn evaluate_script(source: &str, external_functions: Option<Object>, on_progress: Option<Function>) -> JsValue {
    let mut state = ScriptState::pending();
    emit_progress(&on_progress, &state);

    state = ScriptState::running();
    emit_progress(&on_progress, &state);

    let ext_fns = external_functions
        .map(|o| external_functions::definitions(&o))
        .unwrap_or_default();

    let mut runner = AoCRunner::new_with_external_functions(WebTime {}, &ext_fns);

    state = match runner.run(source) {
        Ok(result) => {
            if let RunEvaluation::Script(run_result) = result {
                ScriptState::complete(run_result.value, run_result.duration as u64)
            } else {
                // Shouldn't happen for scripts, but handle gracefully
                ScriptState::complete("".to_string(), 0)
            }
        }
        Err(error) => ScriptState::error(output::format_error(source, &error)),
    };

    emit_progress(&on_progress, &state);
    serde_wasm_bindgen::to_value(&state).unwrap()
}

// ============================================================
// EVALUATE SOLUTION
// ============================================================

/// Evaluate a solution (part_one/part_two).
///
/// Returns a `SolutionState` with results for each part.
///
/// ## Options
/// - `externalFunctions`: Object mapping function names to JavaScript functions
/// - `onProgress`: Callback invoked with state updates
#[wasm_bindgen(js_name = evaluateSolution)]
pub fn evaluate_solution(source: &str, external_functions: Option<Object>, on_progress: Option<Function>) -> JsValue {
    let mut state = SolutionState::pending();
    emit_progress(&on_progress, &state);

    state = SolutionState::running();
    emit_progress(&on_progress, &state);

    let ext_fns = external_functions
        .map(|o| external_functions::definitions(&o))
        .unwrap_or_default();

    let mut runner = AoCRunner::new_with_external_functions(WebTime {}, &ext_fns);

    // Mark part_one as running
    state.part_one = PartResult::running();
    emit_progress(&on_progress, &state);

    state = match runner.run(source) {
        Ok(result) => {
            if let RunEvaluation::Solution { part_one, part_two } = result {
                // Update part_one
                let p1 = part_one
                    .as_ref()
                    .map(PartResult::complete)
                    .unwrap_or_else(PartResult::not_present);

                // Emit part_one complete, part_two running
                state.part_one = p1.clone();
                state.part_two = PartResult::running();
                emit_progress(&on_progress, &state);

                // Update part_two
                let p2 = part_two
                    .as_ref()
                    .map(PartResult::complete)
                    .unwrap_or_else(PartResult::not_present);

                SolutionState {
                    state_type: "solution",
                    status: Status::Complete,
                    part_one: p1,
                    part_two: p2,
                    error: None,
                }
            } else {
                // Source was actually a script - return empty solution
                SolutionState {
                    state_type: "solution",
                    status: Status::Complete,
                    part_one: PartResult::not_present(),
                    part_two: PartResult::not_present(),
                    error: None,
                }
            }
        }
        Err(error) => SolutionState::error(output::format_error(source, &error)),
    };

    emit_progress(&on_progress, &state);
    serde_wasm_bindgen::to_value(&state).unwrap()
}

// ============================================================
// TEST SOLUTION
// ============================================================

/// Run test cases defined in the source.
///
/// Returns a `TestState` with results for each test case.
///
/// ## Options
/// - `includeSlow`: Whether to run tests marked with @slow (default: false)
/// - `externalFunctions`: Object mapping function names to JavaScript functions
/// - `onProgress`: Callback invoked with state updates
#[wasm_bindgen(js_name = testSolution)]
pub fn test_solution(
    source: &str,
    include_slow: Option<bool>,
    external_functions: Option<Object>,
    on_progress: Option<Function>,
) -> JsValue {
    let test_count = count_test_sections(source);

    let mut state = TestState::pending(test_count);
    emit_progress(&on_progress, &state);

    state = TestState::running(test_count);
    emit_progress(&on_progress, &state);

    let ext_fns = external_functions
        .map(|o| external_functions::definitions(&o))
        .unwrap_or_default();

    let mut runner = AoCRunner::new_with_external_functions(WebTime {}, &ext_fns);

    state = match runner.test(source, include_slow.unwrap_or(false)) {
        Ok(test_cases) => TestState::complete(&test_cases),
        Err(error) => TestState::error(output::format_error(source, &error)),
    };

    emit_progress(&on_progress, &state);
    serde_wasm_bindgen::to_value(&state).unwrap()
}

/// Count test sections in source.
fn count_test_sections(source: &str) -> u32 {
    source.matches("test:").count() as u32
}

// ============================================================
// FORMAT
// ============================================================

/// Format santa-lang source code.
///
/// Returns a `FormatResult` with the formatted code or error.
#[wasm_bindgen]
pub fn format(source: &str) -> JsValue {
    let result = match lang::format(source) {
        Ok(formatted) => FormatResult::success(formatted),
        Err(error) => FormatResult::error(output::format_parser_error(source, &error)),
    };
    serde_wasm_bindgen::to_value(&result).unwrap()
}

/// Check if source code is already formatted.
///
/// Returns `true` if the source is already formatted, `false` otherwise.
/// Returns `false` on parse errors.
#[wasm_bindgen(js_name = isFormatted)]
pub fn is_formatted(source: &str) -> bool {
    lang::is_formatted(source).unwrap_or(false)
}
