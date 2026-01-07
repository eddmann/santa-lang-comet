use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn script() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd
        .arg(format!("{}/fixtures/script.santa", env!("CARGO_MANIFEST_DIR")))
        .assert();
    assert.success().stdout("14\n");
}

#[test]
fn solution() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd
        .arg(format!("{}/fixtures/solution.santa", env!("CARGO_MANIFEST_DIR")))
        .assert();
    assert
        .success()
        .stdout(predicate::str::contains("Part 1: \u{1b}[32m232\u{1b}[0m"))
        .stdout(predicate::str::contains("Part 2: \u{1b}[32m1783\u{1b}[0m"));
}

#[test]
fn test_solution() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd
        .arg("-t")
        .arg(format!("{}/fixtures/solution.santa", env!("CARGO_MANIFEST_DIR")))
        .assert();
    assert
        .success()
        .stdout(predicate::str::contains("Testcase #1"))
        .stdout(predicate::str::contains("Part 1: -1 \u{1b}[32m✔\u{1b}[0m"))
        .stdout(predicate::str::contains("Part 2: 5 \u{1b}[32m✔\u{1b}[0m"));
}

#[test]
fn repl() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd.arg("-r").write_stdin("[1, 2] + [3]").assert();
    assert.success().stdout(predicate::str::contains("[1, 2, 3]"));
}

#[test]
fn eval_simple_expression() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd.arg("-e").arg("1 + 2").assert();
    assert.success().stdout("3\n");
}

#[test]
fn eval_complex_expression() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd.arg("-e").arg("map(|x| x * 2, [1, 2, 3])").assert();
    assert.success().stdout("[2, 4, 6]\n");
}

#[test]
fn eval_aoc_solution() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd.arg("-e").arg("part_one: { 42 }").assert();
    assert
        .success()
        .stdout(predicate::str::contains("Part 1: \u{1b}[32m42\u{1b}[0m"));
}

#[test]
fn stdin_simple_expression() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd.write_stdin("1 + 2").assert();
    assert.success().stdout("3\n");
}

#[test]
fn stdin_aoc_solution() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd.write_stdin("part_one: { 42 }").assert();
    assert
        .success()
        .stdout(predicate::str::contains("Part 1: \u{1b}[32m42\u{1b}[0m"));
}

#[test]
fn stdin_empty() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd.write_stdin("").assert();
    // Empty program should succeed with no output
    assert.success();
}

#[test]
fn fmt_stdout_simple_expression() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd.arg("-f").arg("-e").arg("1+2").assert();
    assert.success().stdout("1 + 2\n");
}

#[test]
fn fmt_stdout_list() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd.arg("-f").arg("-e").arg("[1,2,3]").assert();
    assert.success().stdout("[1, 2, 3]\n");
}

#[test]
fn fmt_stdout_lambda() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd.arg("-f").arg("-e").arg("|x|x+1").assert();
    assert.success().stdout("|x| x + 1\n");
}

#[test]
fn fmt_stdout_let() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd.arg("-f").arg("-e").arg("let x=1").assert();
    assert.success().stdout("let x = 1\n");
}

#[test]
fn fmt_stdout_string_with_escapes() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd.arg("-f").arg("-e").arg(r#""hello\nworld""#).assert();
    assert.success().stdout("\"hello\\nworld\"\n");
}

#[test]
fn fmt_stdin() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd.arg("-f").write_stdin("1+2").assert();
    assert.success().stdout("1 + 2\n");
}

#[test]
fn fmt_check_passes_for_formatted() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd.arg("--fmt-check").arg("-e").arg("1 + 2\n").assert();
    assert.success();
}

#[test]
fn fmt_check_fails_for_unformatted() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd.arg("--fmt-check").arg("-e").arg("1+2").assert();
    assert.code(1);
}

#[test]
fn fmt_write_requires_file() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd.arg("--fmt-write").arg("-e").arg("1+2").assert();
    assert.code(1).stderr(predicate::str::contains("requires a file path"));
}

#[test]
fn fmt_invalid_syntax_returns_error() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd.arg("-f").arg("-e").arg("let = ").assert();
    assert.code(2).stderr(predicate::str::contains("Parse error"));
}

// JSON Output Format Tests

#[test]
fn json_script_simple() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd.arg("-o").arg("json").arg("-e").arg("1 + 2").assert();
    assert
        .success()
        .stdout(predicate::str::contains(r#""type":"script""#))
        .stdout(predicate::str::contains(r#""status":"complete""#))
        .stdout(predicate::str::contains(r#""value":"3""#));
}

#[test]
fn json_script_with_console() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd.arg("-o").arg("json").arg("-e").arg(r#"puts("hello"); 42"#).assert();
    assert
        .success()
        .stdout(predicate::str::contains(r#""type":"script""#))
        .stdout(predicate::str::contains(r#""value":"42""#))
        .stdout(predicate::str::contains(r#""message":"hello""#));
}

#[test]
fn json_solution() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd
        .arg("-o")
        .arg("json")
        .arg(format!("{}/fixtures/solution.santa", env!("CARGO_MANIFEST_DIR")))
        .assert();
    assert
        .success()
        .stdout(predicate::str::contains(r#""type":"solution""#))
        .stdout(predicate::str::contains(r#""part_one":"#))
        .stdout(predicate::str::contains(r#""part_two":"#))
        .stdout(predicate::str::contains(r#""status":"complete""#));
}

#[test]
fn json_solution_single_part() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd.arg("-o").arg("json").arg("-e").arg("part_one: { 42 }").assert();
    assert
        .success()
        .stdout(predicate::str::contains(r#""type":"solution""#))
        .stdout(predicate::str::contains(r#""part_one":"#))
        .stdout(predicate::str::contains(r#""value":"42""#))
        // part_two should not be present when not defined
        .stdout(predicate::str::contains(r#""part_two""#).not());
}

#[test]
fn json_error_runtime() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd.arg("-o").arg("json").arg("-e").arg(r#"1 * "x""#).assert();
    assert
        .code(2)
        .stdout(predicate::str::contains(r#""type":"error""#))
        .stdout(predicate::str::contains(r#""message":"#))
        .stdout(predicate::str::contains(r#""location":"#))
        .stdout(predicate::str::contains(r#""line":1"#));
}

#[test]
fn json_error_parse() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd.arg("-o").arg("json").arg("-e").arg("1 ^ 2").assert();
    assert
        .code(2)
        .stdout(predicate::str::contains(r#""type":"error""#))
        .stdout(predicate::str::contains(r#""message":"#));
}

#[test]
fn json_test_passing() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd
        .arg("-o")
        .arg("json")
        .arg("-t")
        .arg(format!("{}/fixtures/solution.santa", env!("CARGO_MANIFEST_DIR")))
        .assert();
    assert
        .success()
        .stdout(predicate::str::contains(r#""type":"test""#))
        .stdout(predicate::str::contains(r#""success":true"#))
        .stdout(predicate::str::contains(r#""passed":1"#))
        .stdout(predicate::str::contains(r#""failed":0"#));
}

#[test]
fn json_test_failing() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd
        .arg("-o")
        .arg("json")
        .arg("-t")
        .arg("-e")
        .arg(
            r#"
            part_one: { 99 }
            test: {
                input: "x"
                part_one: 42
            }
            "#,
        )
        .assert();
    assert
        .code(3)
        .stdout(predicate::str::contains(r#""type":"test""#))
        .stdout(predicate::str::contains(r#""success":false"#))
        .stdout(predicate::str::contains(r#""passed":false"#))
        .stdout(predicate::str::contains(r#""expected":"42""#))
        .stdout(predicate::str::contains(r#""actual":"99""#));
}

#[test]
fn json_test_skipped() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd
        .arg("-o")
        .arg("json")
        .arg("-t")
        .arg(format!(
            "{}/fixtures/solution_with_slow_test.santa",
            env!("CARGO_MANIFEST_DIR")
        ))
        .assert();
    assert
        .success()
        .stdout(predicate::str::contains(r#""type":"test""#))
        .stdout(predicate::str::contains(r#""skipped":1"#))
        .stdout(predicate::str::contains(r#""status":"skipped""#));
}

#[test]
fn json_test_skipped_included_with_slow_flag() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd
        .arg("-o")
        .arg("json")
        .arg("-t")
        .arg("-s")
        .arg(format!(
            "{}/fixtures/solution_with_slow_test.santa",
            env!("CARGO_MANIFEST_DIR")
        ))
        .assert();
    assert
        .success()
        .stdout(predicate::str::contains(r#""type":"test""#))
        .stdout(predicate::str::contains(r#""skipped":0"#))
        .stdout(predicate::str::contains(r#""passed":2"#));
}

// JSONL Output Format Tests

#[test]
fn jsonl_script_simple() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd.arg("-o").arg("jsonl").arg("-e").arg("1 + 2").assert();
    // First line is initial state
    assert
        .success()
        .stdout(predicate::str::contains(r#""type":"script""#))
        .stdout(predicate::str::contains(r#""status":"pending""#))
        // Patches include running and complete
        .stdout(predicate::str::contains(r#""op":"replace""#))
        .stdout(predicate::str::contains(r#""/status""#))
        .stdout(predicate::str::contains(r#""running""#))
        .stdout(predicate::str::contains(r#""complete""#));
}

#[test]
fn jsonl_solution() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd.arg("-o").arg("jsonl").arg("-e").arg("part_one: { 42 }").assert();
    assert
        .success()
        .stdout(predicate::str::contains(r#""type":"solution""#))
        .stdout(predicate::str::contains(r#""/part_one/status""#))
        .stdout(predicate::str::contains(r#""/part_one/value""#));
}

#[test]
fn jsonl_error() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd.arg("-o").arg("jsonl").arg("-e").arg(r#"1 * "x""#).assert();
    assert
        .code(2)
        .stdout(predicate::str::contains(r#""type":"script""#))
        .stdout(predicate::str::contains(r#""/error""#))
        .stdout(predicate::str::contains(r#""message""#));
}

#[test]
fn jsonl_test() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd
        .arg("-o")
        .arg("jsonl")
        .arg("-t")
        .arg(format!("{}/fixtures/solution.santa", env!("CARGO_MANIFEST_DIR")))
        .assert();
    assert
        .success()
        .stdout(predicate::str::contains(r#""type":"test""#))
        .stdout(predicate::str::contains(r#""/tests/0/status""#))
        .stdout(predicate::str::contains(r#""/summary/passed""#));
}

// Output mode validation

#[test]
fn invalid_output_mode() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd.arg("-o").arg("xml").arg("-e").arg("1").assert();
    assert.code(1).stderr(predicate::str::contains("Invalid output format"));
}

// Version with JSON output format tests

#[test]
fn version_json_output() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd.arg("--version").arg("-o").arg("json").assert();
    assert
        .success()
        .stdout(predicate::str::contains(r#""reindeer":"Comet""#))
        .stdout(predicate::str::contains(r#""version":"#));
}

#[test]
fn version_jsonl_output() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd.arg("--version").arg("-o").arg("jsonl").assert();
    assert
        .success()
        .stdout(predicate::str::contains(r#""reindeer":"Comet""#))
        .stdout(predicate::str::contains(r#""version":"#));
}

#[test]
fn version_json_output_flag_order() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("santa-cli").unwrap();
    let assert = cmd.arg("-o").arg("json").arg("--version").assert();
    assert
        .success()
        .stdout(predicate::str::contains(r#""reindeer":"Comet""#))
        .stdout(predicate::str::contains(r#""version":"#));
}
