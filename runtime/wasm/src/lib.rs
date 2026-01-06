mod external_functions;

use js_sys::{Array, Object};
use lang::{AoCRunner, Environment, Evaluator, Lexer, Parser, Time};
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

#[wasm_bindgen]
pub fn aoc_run(source: &str, js_functions: Object) -> Result<JsValue, JsValue> {
    let mut runner =
        AoCRunner::new_with_external_functions(WebTime {}, &crate::external_functions::definitions(&js_functions));

    match runner.run(source) {
        Ok(result) => Ok(serde_wasm_bindgen::to_value(&result).unwrap()),
        Err(error) => Err(serde_wasm_bindgen::to_value(&error).unwrap()),
    }
}

#[wasm_bindgen]
pub fn aoc_test(source: &str, js_functions: Object, include_slow: Option<bool>) -> Result<JsValue, JsValue> {
    let mut runner =
        AoCRunner::new_with_external_functions(WebTime {}, &crate::external_functions::definitions(&js_functions));

    match runner.test(source, include_slow.unwrap_or(false)) {
        Ok(test_cases) => Ok(JsValue::from(
            test_cases
                .iter()
                .map(|test_case| serde_wasm_bindgen::to_value(test_case).unwrap())
                .collect::<Array>(),
        )),
        Err(error) => Err(serde_wasm_bindgen::to_value(&error).unwrap()),
    }
}

#[wasm_bindgen]
pub fn evaluate(expression: &str, js_functions: Option<Object>) -> Result<JsValue, JsValue> {
    let external_functions = if let Some(js_functions) = js_functions {
        crate::external_functions::definitions(&js_functions)
    } else {
        vec![]
    };
    let mut evaluator = Evaluator::new_with_external_functions(&external_functions);

    let lexer = Lexer::new(expression);
    let mut parser = Parser::new(lexer);
    let program = match parser.parse() {
        Ok(parsed) => parsed,
        Err(error) => return Err(serde_wasm_bindgen::to_value(&error).unwrap()),
    };

    match evaluator.evaluate_with_environment(&program, Environment::new()) {
        Ok(evaluated) => Ok(JsValue::from(evaluated.to_string())),
        Err(error) => Err(serde_wasm_bindgen::to_value(&error).unwrap()),
    }
}

#[wasm_bindgen]
pub fn format(source: &str) -> Result<JsValue, JsValue> {
    match lang::format(source) {
        Ok(formatted) => Ok(JsValue::from_str(&formatted)),
        Err(error) => Err(serde_wasm_bindgen::to_value(&error).unwrap()),
    }
}

#[wasm_bindgen]
pub fn is_formatted(source: &str) -> Result<bool, JsValue> {
    match lang::is_formatted(source) {
        Ok(result) => Ok(result),
        Err(error) => Err(serde_wasm_bindgen::to_value(&error).unwrap()),
    }
}
