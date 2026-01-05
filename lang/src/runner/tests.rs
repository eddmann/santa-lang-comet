use super::*;

use expect_test::{Expect, expect};

#[test]
fn script() {
    assert_run(
        r#"
            let x = 1;
            let y = 5;
            x..y |> map(_ + 1) |> list;
        "#,
        expect![[r#"
            Ok(
                Script(
                    RunResult {
                        value: "[2, 3, 4, 5]",
                        duration: 0,
                    },
                ),
            )"#]],
    )
}

#[test]
fn solution_with_both_parts() {
    assert_run(
        r#"
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
        "#,
        expect![[r#"
            Ok(
                Solution {
                    part_one: Some(
                        RunResult {
                            value: "-1",
                            duration: 0,
                        },
                    ),
                    part_two: Some(
                        RunResult {
                            value: "5",
                            duration: 0,
                        },
                    ),
                },
            )"#]],
    )
}

#[test]
fn solution_with_only_part_one() {
    assert_run(
        r#"
            input: "()())"

            part_one: {
                input |> fold(0) |floor, direction| {
                    if direction == "(" { floor + 1 } else { floor - 1 };
                }
            }
        "#,
        expect![[r#"
            Ok(
                Solution {
                    part_one: Some(
                        RunResult {
                            value: "-1",
                            duration: 0,
                        },
                    ),
                    part_two: None,
                },
            )"#]],
    )
}

#[test]
fn solution_with_only_part_two() {
    assert_run(
        r#"
            input: "()())"

            part_two: {
                zip(1.., input) |> fold(0) |floor, [index, direction]| {
                    let next_floor = if direction == "(" { floor + 1 } else { floor - 1 };
                    if next_floor < 0 { break index } else { next_floor };
                }
            }
        "#,
        expect![[r#"
            Ok(
                Solution {
                    part_one: None,
                    part_two: Some(
                        RunResult {
                            value: "5",
                            duration: 0,
                        },
                    ),
                },
            )"#]],
    )
}

#[test]
fn passing_test_case_with_both_parts() {
    assert_test(
        r#"
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
        "#,
        expect![[r#"
            Ok(
                [
                    TestCase {
                        part_one: Some(
                            TestCaseResult {
                                expected: "-1",
                                actual: "-1",
                                passed: true,
                            },
                        ),
                        part_two: Some(
                            TestCaseResult {
                                expected: "5",
                                actual: "5",
                                passed: true,
                            },
                        ),
                        slow: false,
                        skipped: false,
                    },
                ],
            )"#]],
    )
}

#[test]
fn passing_test_case_with_part_one() {
    assert_test(
        r#"
            part_one: {
                input |> fold(0) |floor, direction| {
                    if direction == "(" { floor + 1 } else { floor - 1 };
                }
            }

            test: {
                input: "()())"
                part_one: -1
            }
        "#,
        expect![[r#"
            Ok(
                [
                    TestCase {
                        part_one: Some(
                            TestCaseResult {
                                expected: "-1",
                                actual: "-1",
                                passed: true,
                            },
                        ),
                        part_two: None,
                        slow: false,
                        skipped: false,
                    },
                ],
            )"#]],
    )
}

#[test]
fn passing_test_case_with_part_two() {
    assert_test(
        r#"
            part_two: {
                zip(1.., input) |> fold(0) |floor, [index, direction]| {
                    let next_floor = if direction == "(" { floor + 1 } else { floor - 1 };
                    if next_floor < 0 { break index } else { next_floor };
                }
            }

            test: {
                input: "()())"
                part_two: 5
            }
        "#,
        expect![[r#"
            Ok(
                [
                    TestCase {
                        part_one: None,
                        part_two: Some(
                            TestCaseResult {
                                expected: "5",
                                actual: "5",
                                passed: true,
                            },
                        ),
                        slow: false,
                        skipped: false,
                    },
                ],
            )"#]],
    )
}

#[test]
fn failing_test_case_with_both_parts() {
    assert_test(
        r#"
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
                part_one: -2
                part_two: 6
            }
        "#,
        expect![[r#"
            Ok(
                [
                    TestCase {
                        part_one: Some(
                            TestCaseResult {
                                expected: "-2",
                                actual: "-1",
                                passed: false,
                            },
                        ),
                        part_two: Some(
                            TestCaseResult {
                                expected: "6",
                                actual: "5",
                                passed: false,
                            },
                        ),
                        slow: false,
                        skipped: false,
                    },
                ],
            )"#]],
    )
}

#[test]
fn script_with_runtime_error() {
    assert_run(
        "1 * \"1\"",
        expect![[r#"
            Err(
                RunErr {
                    message: "Unsupported operation: Integer * String",
                    source: 2..7,
                    trace: [],
                },
            )"#]],
    )
}

#[test]
fn script_with_parser_error() {
    assert_run(
        "1 ^ 3",
        expect![[r#"
            Err(
                RunErr {
                    message: "Illegal token",
                    source: 2..3,
                    trace: [],
                },
            )"#]],
    )
}

#[test]
fn solution_with_runtime_error() {
    assert_run(
        "part_one: 1 * \"1\"",
        expect![[r#"
            Err(
                RunErr {
                    message: "Unsupported operation: Integer * String",
                    source: 12..17,
                    trace: [],
                },
            )"#]],
    )
}

#[test]
fn solution_with_parser_error() {
    assert_run(
        "part_one: 1 ^ 3",
        expect![[r#"
            Err(
                RunErr {
                    message: "Illegal token",
                    source: 12..13,
                    trace: [],
                },
            )"#]],
    )
}

#[test]
fn solution_with_multiple_input_sections() {
    assert_run(
        r#"
            input: {}
            input: {}
            part_one: {}
            part_two: {}
        "#,
        expect![[r#"
            Err(
                RunErr {
                    message: "Expected a single 'input' section",
                    source: 35..57,
                    trace: [],
                },
            )"#]],
    )
}

#[test]
fn solution_with_multiple_part_one_sections() {
    assert_run(
        r#"
            input: {}
            part_one: {}
            part_one: {}
            part_two: {}
        "#,
        expect![[r#"
            Err(
                RunErr {
                    message: "Expected single 'part_one' solution",
                    source: 60..85,
                    trace: [],
                },
            )"#]],
    )
}

#[test]
fn solution_with_multiple_part_two_sections() {
    assert_run(
        r#"
            input: {}
            part_one: {}
            part_two: {}
            part_two: {}
        "#,
        expect![[r#"
            Err(
                RunErr {
                    message: "Expected single 'part_two' solution",
                    source: 85..106,
                    trace: [],
                },
            )"#]],
    )
}

#[test]
fn test_with_multiple_input_sections() {
    assert_test(
        r#"
            input: {}
            part_one: {}
            part_two: {}
            test: {
                input: {}
                input: {}
                part_one: {}
                part_two: {}
            }
        "#,
        expect![[r#"
            Err(
                RunErr {
                    message: "Expected a single 'input' fixture",
                    source: 135..161,
                    trace: [],
                },
            )"#]],
    )
}

#[test]
fn test_with_multiple_part_one_sections() {
    assert_test(
        r#"
            input: {}
            part_one: {}
            part_two: {}
            test: {
                input: {}
                part_one: {}
                part_one: {}
                part_two: {}
            }
        "#,
        expect![[r#"
            Err(
                RunErr {
                    message: "Expected a single 'part_one' assertion",
                    source: 164..193,
                    trace: [],
                },
            )"#]],
    )
}

#[test]
fn test_with_multiple_part_two_sections() {
    assert_test(
        r#"
            input: {}
            part_one: {}
            part_two: {}
            test: {
                input: {}
                part_one: {}
                part_two: {}
                part_two: {}
            }
        "#,
        expect![[r#"
            Err(
                RunErr {
                    message: "Expected a single 'part_two' assertion",
                    source: 193..218,
                    trace: [],
                },
            )"#]],
    )
}

struct StubTime {}
impl Time for StubTime {
    fn now(&self) -> u128 {
        0
    }
}

fn assert_run(input: &str, expected: Expect) {
    let result = AoCRunner::new(StubTime {}).run(input);
    let actual = format!("{:#?}", result);
    expected.assert_eq(&actual)
}

fn assert_test(input: &str, expected: Expect) {
    let result = AoCRunner::new(StubTime {}).test(input, true);
    let actual = format!("{:#?}", result);
    expected.assert_eq(&actual)
}

fn assert_test_with_slow(input: &str, include_slow: bool, expected: Expect) {
    let result = AoCRunner::new(StubTime {}).test(input, include_slow);
    let actual = format!("{:#?}", result);
    expected.assert_eq(&actual)
}

#[test]
fn slow_test_is_included_when_flag_set() {
    assert_test_with_slow(
        r#"
            part_one: { 42 }

            @slow
            test: {
                input: "data"
                part_one: 42
            }
        "#,
        true,
        expect![[r#"
            Ok(
                [
                    TestCase {
                        part_one: Some(
                            TestCaseResult {
                                expected: "42",
                                actual: "42",
                                passed: true,
                            },
                        ),
                        part_two: None,
                        slow: true,
                        skipped: false,
                    },
                ],
            )"#]],
    )
}

#[test]
fn slow_test_is_skipped_by_default() {
    assert_test_with_slow(
        r#"
            part_one: { 42 }

            @slow
            test: {
                input: "data"
                part_one: 42
            }
        "#,
        false,
        expect![[r#"
            Ok(
                [
                    TestCase {
                        part_one: None,
                        part_two: None,
                        slow: true,
                        skipped: true,
                    },
                ],
            )"#]],
    )
}

#[test]
fn mixed_slow_and_normal_tests() {
    assert_test_with_slow(
        r#"
            part_one: { 42 }

            test: {
                input: "data"
                part_one: 42
            }

            @slow
            test: {
                input: "data"
                part_one: 42
            }
        "#,
        false,
        expect![[r#"
            Ok(
                [
                    TestCase {
                        part_one: Some(
                            TestCaseResult {
                                expected: "42",
                                actual: "42",
                                passed: true,
                            },
                        ),
                        part_two: None,
                        slow: false,
                        skipped: false,
                    },
                    TestCase {
                        part_one: None,
                        part_two: None,
                        slow: true,
                        skipped: true,
                    },
                ],
            )"#]],
    )
}

#[test]
fn mixed_slow_and_normal_tests_with_include_slow() {
    assert_test_with_slow(
        r#"
            part_one: { 42 }

            test: {
                input: "data"
                part_one: 42
            }

            @slow
            test: {
                input: "data"
                part_one: 42
            }
        "#,
        true,
        expect![[r#"
            Ok(
                [
                    TestCase {
                        part_one: Some(
                            TestCaseResult {
                                expected: "42",
                                actual: "42",
                                passed: true,
                            },
                        ),
                        part_two: None,
                        slow: false,
                        skipped: false,
                    },
                    TestCase {
                        part_one: Some(
                            TestCaseResult {
                                expected: "42",
                                actual: "42",
                                passed: true,
                            },
                        ),
                        part_two: None,
                        slow: true,
                        skipped: false,
                    },
                ],
            )"#]],
    )
}
