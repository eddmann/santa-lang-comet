use super::*;
use js_sys::Reflect;
use wasm_bindgen_test::*;

const SCRIPT: &str = "1..5 |> map(_ + 1) |> reduce(+);";

const SOLUTION: &str = r#"
input: "()())"

part_one: {
  input |> fold(0) |floor, direction| {
    if direction == "(" { floor + 1 } else { floor - 1 };
  }
}

part_two: {
  zip(1.., input) |> fold(0) |floor, [index, direction]| {
    let next_floor = if direction == "(" { floor + 1 } else { floor - 1 };
    if next_floor < 0 { break index } else { next_floor };
  }
}

test: {
  input: "()())"
  part_one: -1
  part_two: 5
}
"#;

// ============================================================
// evaluateScript tests
// ============================================================

#[wasm_bindgen_test]
fn evaluate_script_simple() {
    let result = evaluate_script(SCRIPT, None, None);

    assert_eq!(
        "script",
        Reflect::get(&result, &"type".into()).unwrap().as_string().unwrap()
    );
    assert_eq!(
        "complete",
        Reflect::get(&result, &"status".into()).unwrap().as_string().unwrap()
    );
    assert_eq!(
        "14",
        Reflect::get(&result, &"value".into()).unwrap().as_string().unwrap()
    );
    assert!(Reflect::get(&result, &"duration_ms".into()).unwrap().as_f64().is_some());
}

#[wasm_bindgen_test]
fn evaluate_script_with_error() {
    let result = evaluate_script("let x = 1 / 0;", None, None);

    assert_eq!(
        "script",
        Reflect::get(&result, &"type".into()).unwrap().as_string().unwrap()
    );
    assert_eq!(
        "error",
        Reflect::get(&result, &"status".into()).unwrap().as_string().unwrap()
    );

    let error = Reflect::get(&result, &"error".into()).unwrap();
    assert!(!error.is_null() && !error.is_undefined());

    let location = Reflect::get(&error, &"location".into()).unwrap();
    assert!(Reflect::get(&location, &"line".into()).unwrap().as_f64().is_some());
    assert!(Reflect::get(&location, &"column".into()).unwrap().as_f64().is_some());
}

#[wasm_bindgen_test]
fn evaluate_script_parse_error() {
    let result = evaluate_script("let x = ", None, None);

    assert_eq!(
        "error",
        Reflect::get(&result, &"status".into()).unwrap().as_string().unwrap()
    );

    let error = Reflect::get(&result, &"error".into()).unwrap();
    assert!(!error.is_null() && !error.is_undefined());
}

// ============================================================
// evaluateSolution tests
// ============================================================

#[wasm_bindgen_test]
fn evaluate_solution_both_parts() {
    let result = evaluate_solution(SOLUTION, None, None);

    assert_eq!(
        "solution",
        Reflect::get(&result, &"type".into()).unwrap().as_string().unwrap()
    );
    assert_eq!(
        "complete",
        Reflect::get(&result, &"status".into()).unwrap().as_string().unwrap()
    );

    let part_one = Reflect::get(&result, &"part_one".into()).unwrap();
    assert_eq!(
        "complete",
        Reflect::get(&part_one, &"status".into()).unwrap().as_string().unwrap()
    );
    assert_eq!(
        "-1",
        Reflect::get(&part_one, &"value".into()).unwrap().as_string().unwrap()
    );

    let part_two = Reflect::get(&result, &"part_two".into()).unwrap();
    assert_eq!(
        "complete",
        Reflect::get(&part_two, &"status".into()).unwrap().as_string().unwrap()
    );
    assert_eq!(
        "5",
        Reflect::get(&part_two, &"value".into()).unwrap().as_string().unwrap()
    );
}

#[wasm_bindgen_test]
fn evaluate_solution_part_one_only() {
    let source = r#"
        input: "test"
        part_one: { input |> size }
    "#;

    let result = evaluate_solution(source, None, None);

    assert_eq!(
        "complete",
        Reflect::get(&result, &"status".into()).unwrap().as_string().unwrap()
    );

    let part_one = Reflect::get(&result, &"part_one".into()).unwrap();
    assert_eq!(
        "4",
        Reflect::get(&part_one, &"value".into()).unwrap().as_string().unwrap()
    );

    // part_two should be complete but with no value
    let part_two = Reflect::get(&result, &"part_two".into()).unwrap();
    assert_eq!(
        "complete",
        Reflect::get(&part_two, &"status".into()).unwrap().as_string().unwrap()
    );
    assert!(Reflect::get(&part_two, &"value".into()).unwrap().is_undefined());
}

#[wasm_bindgen_test]
fn evaluate_solution_with_error() {
    let source = "part_one: { let x = }";

    let result = evaluate_solution(source, None, None);

    assert_eq!(
        "solution",
        Reflect::get(&result, &"type".into()).unwrap().as_string().unwrap()
    );
    assert_eq!(
        "error",
        Reflect::get(&result, &"status".into()).unwrap().as_string().unwrap()
    );

    let error = Reflect::get(&result, &"error".into()).unwrap();
    assert!(!error.is_null() && !error.is_undefined());
}

// ============================================================
// testSolution tests
// ============================================================

#[wasm_bindgen_test]
fn test_solution_passing() {
    let result = test_solution(SOLUTION, Some(true), None, None);

    assert_eq!(
        "test",
        Reflect::get(&result, &"type".into()).unwrap().as_string().unwrap()
    );
    assert_eq!(
        "complete",
        Reflect::get(&result, &"status".into()).unwrap().as_string().unwrap()
    );
    assert!(Reflect::get(&result, &"success".into()).unwrap().as_bool().unwrap());

    let summary = Reflect::get(&result, &"summary".into()).unwrap();
    assert_eq!(1.0, Reflect::get(&summary, &"total".into()).unwrap().as_f64().unwrap());
    assert_eq!(1.0, Reflect::get(&summary, &"passed".into()).unwrap().as_f64().unwrap());
    assert_eq!(0.0, Reflect::get(&summary, &"failed".into()).unwrap().as_f64().unwrap());
    assert_eq!(
        0.0,
        Reflect::get(&summary, &"skipped".into()).unwrap().as_f64().unwrap()
    );

    let tests = js_sys::Array::from(&Reflect::get(&result, &"tests".into()).unwrap());
    assert_eq!(1, tests.length());

    let test_case = tests.get(0);
    assert_eq!(
        1.0,
        Reflect::get(&test_case, &"index".into()).unwrap().as_f64().unwrap()
    );
    assert_eq!(
        "complete",
        Reflect::get(&test_case, &"status".into()).unwrap().as_string().unwrap()
    );

    let part_one = Reflect::get(&test_case, &"part_one".into()).unwrap();
    assert!(Reflect::get(&part_one, &"passed".into()).unwrap().as_bool().unwrap());
    assert_eq!(
        "-1",
        Reflect::get(&part_one, &"expected".into())
            .unwrap()
            .as_string()
            .unwrap()
    );
    assert_eq!(
        "-1",
        Reflect::get(&part_one, &"actual".into()).unwrap().as_string().unwrap()
    );
}

#[wasm_bindgen_test]
fn test_solution_failing() {
    let source = r#"
        part_one: { 42 }
        test: {
            part_one: 99
        }
    "#;

    let result = test_solution(source, None, None, None);

    assert_eq!(
        "complete",
        Reflect::get(&result, &"status".into()).unwrap().as_string().unwrap()
    );
    assert!(!Reflect::get(&result, &"success".into()).unwrap().as_bool().unwrap());

    let summary = Reflect::get(&result, &"summary".into()).unwrap();
    assert_eq!(1.0, Reflect::get(&summary, &"failed".into()).unwrap().as_f64().unwrap());
}

#[wasm_bindgen_test]
fn test_solution_with_error() {
    let source = "part_one: { let x = }";

    let result = test_solution(source, None, None, None);

    assert_eq!(
        "error",
        Reflect::get(&result, &"status".into()).unwrap().as_string().unwrap()
    );

    let error = Reflect::get(&result, &"error".into()).unwrap();
    assert!(!error.is_null() && !error.is_undefined());
}

// ============================================================
// format tests
// ============================================================

#[wasm_bindgen_test]
fn format_success() {
    let result = format("let x=1+2;");

    assert!(Reflect::get(&result, &"success".into()).unwrap().as_bool().unwrap());
    assert_eq!(
        "let x = 1 + 2\n",
        Reflect::get(&result, &"formatted".into()).unwrap().as_string().unwrap()
    );
    assert!(Reflect::get(&result, &"error".into()).unwrap().is_undefined());
}

#[wasm_bindgen_test]
fn format_error() {
    let result = format("let x = ");

    assert!(!Reflect::get(&result, &"success".into()).unwrap().as_bool().unwrap());
    assert!(Reflect::get(&result, &"formatted".into()).unwrap().is_undefined());

    let error = Reflect::get(&result, &"error".into()).unwrap();
    assert!(!error.is_null() && !error.is_undefined());
}

// ============================================================
// isFormatted tests
// ============================================================

#[wasm_bindgen_test]
fn is_formatted_true() {
    assert!(is_formatted("let x = 1 + 2\n"));
}

#[wasm_bindgen_test]
fn is_formatted_false() {
    assert!(!is_formatted("let x=1+2;"));
}

#[wasm_bindgen_test]
fn is_formatted_invalid_returns_false() {
    assert!(!is_formatted("let x = "));
}
