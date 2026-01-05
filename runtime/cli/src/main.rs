#![allow(clippy::collapsible_if)]

mod external_functions;
mod output;

#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use getopts::Options;
use output::OutputMode;
use rustyline::DefaultEditor;
use santa_lang::{AoCRunner, Environment, Evaluator, Lexer, Location, Object, Parser, RunErr, RunEvaluation, Time};
use std::fs;
use std::io::Read;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(test)]
mod tests;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let mut opts = Options::new();
    opts.optopt("e", "eval", "evaluate inline script", "SCRIPT");
    opts.optopt("o", "output", "output format: text, json, jsonl", "FORMAT");
    opts.optflag("t", "test", "run the solution's test suite");
    opts.optflag("s", "slow", "include slow tests (marked with @slow)");
    opts.optflag("r", "repl", "begin an interactive REPL session");
    opts.optflag("f", "fmt", "format source code to stdout");
    opts.optflag("", "fmt-write", "format source code in place");
    opts.optflag("", "fmt-check", "check if source is formatted (exit 1 if not)");
    opts.optflag("h", "help", "list available commands");
    opts.optflag("v", "version", "display version information");
    #[cfg(feature = "profile")]
    opts.optflag("p", "profile", "profile the execution");

    let matches = opts.parse(&args[1..])?;

    // Parse output mode
    let output_mode = match matches.opt_str("o").as_deref() {
        None | Some("text") => OutputMode::Text,
        Some("json") => OutputMode::Json,
        Some("jsonl") => OutputMode::Jsonl,
        Some(other) => {
            eprintln!("Error: Invalid output format '{}'. Use: text, json, jsonl", other);
            std::process::exit(1);
        }
    };

    if matches.opt_present("h") {
        print_help();
        return Ok(());
    }

    if matches.opt_present("v") {
        println!("santa-lang Comet {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if matches.opt_present("r") {
        return repl();
    }

    // Handle formatting options
    let fmt_stdout = matches.opt_present("f");
    let fmt_write = matches.opt_present("fmt-write");
    let fmt_check = matches.opt_present("fmt-check");

    if fmt_stdout || fmt_write || fmt_check {
        return handle_format(&matches, fmt_stdout, fmt_write, fmt_check);
    }

    // Determine source: -e flag > file argument > stdin
    let (source, source_path): (String, Option<String>) = if let Some(eval_script) = matches.opt_str("e") {
        // Eval mode - use inline script
        (eval_script, None)
    } else if matches.free.len() == 1 {
        // File mode
        let path = &matches.free[0];
        let canonical = fs::canonicalize(path)?;
        let source = fs::read_to_string(&canonical)?;
        (source, Some(canonical.to_string_lossy().into_owned()))
    } else if !atty::is(atty::Stream::Stdin) {
        // Stdin mode - read from stdin when not a TTY
        let mut source = String::new();
        std::io::stdin().read_to_string(&mut source)?;
        (source, None)
    } else {
        print_help();
        std::process::exit(1);
    };

    // Only change directory if we have a file path
    if let Some(ref path) = source_path {
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::env::set_current_dir(parent)?;
        }
    }

    if matches.opt_present("t") {
        let include_slow = matches.opt_present("s");
        return aoc_test(&source, source_path.as_deref(), include_slow, output_mode);
    }

    #[cfg(feature = "profile")]
    let profiler = if matches.opt_present("p") {
        Some(
            pprof::ProfilerGuardBuilder::default()
                .frequency(1000)
                .blocklist(&["libc", "libgcc", "pthread", "vdso"])
                .build()
                .unwrap(),
        )
    } else {
        None
    };

    aoc_run(&source, source_path.as_deref(), output_mode)?;

    #[cfg(feature = "profile")]
    if let Some(guard) = profiler {
        let report = guard.report().build().unwrap();

        let flamegraph = std::fs::File::create("flamegraph.svg").unwrap();
        report.flamegraph(flamegraph).unwrap();

        use pprof::protos::Message;
        use std::io::Write;
        let mut protobuf = std::fs::File::create("profile.pb").unwrap();
        let profile = report.pprof().unwrap();
        let mut content = Vec::new();
        profile.write_to_vec(&mut content).unwrap();
        protobuf.write_all(&content).unwrap();

        println!("\nProfile ⏱️");
        println!("- Flamegraph: ./flamegraph.svg");
        println!("- Protobuf: ./profile.pb");
    }

    Ok(())
}

struct CliTime {}
impl Time for CliTime {
    fn now(&self) -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis()
    }
}

fn print_help() {
    println!(
        "santa-lang CLI - Comet {}

USAGE:
    santa-cli <SCRIPT>              Run solution file
    santa-cli -e <CODE>             Evaluate inline script
    santa-cli -t <SCRIPT>           Run test suite
    santa-cli -t -s <SCRIPT>        Run tests including @slow
    santa-cli -o json <SCRIPT>      Output as JSON
    santa-cli -o jsonl <SCRIPT>     Output as JSON Lines (streaming)
    santa-cli -r                    Start REPL
    santa-cli -h                    Show this help
    cat file | santa-cli            Read from stdin

OPTIONS:
    -e, --eval <CODE>    Evaluate inline script
    -o, --output FORMAT  Output format: text (default), json, jsonl
    -t, --test           Run the solution's test suite
    -s, --slow           Include @slow tests (use with -t)
    -r, --repl           Start interactive REPL
    -f, --fmt            Format source and print to stdout
    --fmt-write          Format source and write in place
    --fmt-check          Check if source is formatted
    -p, --profile        Enable CPU profiling
    -h, --help           Show this help message
    -v, --version        Display version information

ENVIRONMENT:
    SANTA_CLI_SESSION_TOKEN    AOC session token for aoc:// URLs",
        env!("CARGO_PKG_VERSION")
    );
}

fn repl() -> Result<()> {
    let environment = Environment::new();

    let mut functions = crate::external_functions::definitions();
    let shared_environment = Rc::clone(&environment);
    functions.push((
        "env".to_owned(),
        vec![],
        Rc::new(move |_, _| {
            println!("Environment:");
            for (name, value) in shared_environment.borrow().variables() {
                println!("  {} = {}", name, value);
            }
            Ok(Rc::new(Object::Nil))
        }),
    ));

    let mut evaluator = Evaluator::new_with_external_functions(&functions);

    println!(
        "   ,--.\n  ()   \\\n   /    \\\n _/______\\_\n(__________)\n(/  @  @  \\)\n(`._,()._,')  Santa REPL\n(  `-'`-'  )\n \\        /\n  \\,,,,,,/\n"
    );

    let mut rl = DefaultEditor::new()?;

    loop {
        match rl.readline(">> ") {
            Ok(line) => {
                let expression = line.as_str();
                rl.add_history_entry(expression)?;

                let lexer = Lexer::new(expression);
                let mut parser = Parser::new(lexer);
                let program = match parser.parse() {
                    Ok(parsed) => parsed,
                    Err(error) => {
                        println!("{}", error.message);
                        continue;
                    }
                };

                match evaluator.evaluate_with_environment(&program, Rc::clone(&environment)) {
                    Ok(evaluated) => println!("{}", evaluated),
                    Err(error) => println!("{}", error.message),
                };
            }
            Err(_) => {
                println!("Goodbye");
                break;
            }
        }
    }

    Ok(())
}

fn aoc_run(source: &str, source_path: Option<&str>, output_mode: OutputMode) -> Result<()> {
    // Enable console capture for JSON/JSONL modes
    if output_mode != OutputMode::Text {
        crate::external_functions::enable_console_capture();
    }

    let mut runner = AoCRunner::new_with_external_functions(CliTime {}, &crate::external_functions::definitions());

    match output_mode {
        OutputMode::Text => match runner.run(source) {
            Ok(RunEvaluation::Script(result)) => {
                println!("{}", result.value);
                Ok(())
            }
            Ok(RunEvaluation::Solution { part_one, part_two }) => {
                if let Some(part_one) = part_one {
                    println!(
                        "Part 1: \x1b[32m{}\x1b[0m \x1b[90m{}ms\x1b[0m",
                        part_one.value, part_one.duration
                    )
                }

                if let Some(part_two) = part_two {
                    println!(
                        "Part 2: \x1b[32m{}\x1b[0m \x1b[90m{}ms\x1b[0m",
                        part_two.value, part_two.duration
                    )
                }

                Ok(())
            }
            Err(error) => {
                print_error(source_path.unwrap_or("<stdin>"), source, error);
                std::process::exit(2);
            }
        },
        OutputMode::Json => match runner.run(source) {
            Ok(result) => {
                let console = crate::external_functions::disable_console_capture();
                let json = output::format_run_json(&result, console);
                println!("{}", json);
                Ok(())
            }
            Err(error) => {
                let _ = crate::external_functions::disable_console_capture();
                let json = serde_json::to_string(&output::format_error_json(source, &error)).unwrap();
                println!("{}", json);
                std::process::exit(2);
            }
        },
        OutputMode::Jsonl => {
            aoc_run_jsonl(source, &mut runner)
        }
    }
}

fn aoc_run_jsonl(source: &str, runner: &mut AoCRunner<CliTime>) -> Result<()> {
    use output::*;
    use std::io;

    let mut writer = JsonlWriter::new(io::stdout());
    let (has_part_one, has_part_two) = is_solution_source(source);
    let is_solution = has_part_one || has_part_two;

    if is_solution {
        // Emit initial solution state
        let initial = JsonlSolutionInitial {
            output_type: "solution",
            status: "pending",
            part_one: if has_part_one {
                Some(JsonlPartInitial {
                    status: "pending",
                    value: None,
                    duration_ms: None,
                })
            } else {
                None
            },
            part_two: if has_part_two {
                Some(JsonlPartInitial {
                    status: "pending",
                    value: None,
                    duration_ms: None,
                })
            } else {
                None
            },
            console: vec![],
        };
        writer.write_initial(&initial)?;

        // Emit running status
        writer.write_patches(&[JsonlWriter::<io::Stdout>::replace_patch("/status", "running")])?;
    } else {
        // Emit initial script state
        let initial = JsonlScriptInitial {
            output_type: "script",
            status: "pending",
            value: None,
            duration_ms: None,
            console: vec![],
        };
        writer.write_initial(&initial)?;

        // Emit running status
        writer.write_patches(&[JsonlWriter::<io::Stdout>::replace_patch("/status", "running")])?;
    }

    // Run and emit results
    // Note: For full streaming we'd need hooks into the runner. For now we emit after completion.
    match runner.run(source) {
        Ok(result) => {
            let console = crate::external_functions::disable_console_capture();

            match result {
                RunEvaluation::Script(ref r) => {
                    // Emit console entries
                    for entry in &console {
                        writer.write_patches(&[JsonlWriter::<io::Stdout>::add_patch("/console/-", entry)])?;
                    }
                    // Emit completion
                    writer.write_patches(&[
                        JsonlWriter::<io::Stdout>::replace_patch("/status", "complete"),
                        JsonlWriter::<io::Stdout>::replace_patch("/value", &r.value),
                        JsonlWriter::<io::Stdout>::replace_patch("/duration_ms", r.duration as u64),
                    ])?;
                }
                RunEvaluation::Solution { ref part_one, ref part_two } => {
                    // Emit console entries first
                    for entry in &console {
                        writer.write_patches(&[JsonlWriter::<io::Stdout>::add_patch("/console/-", entry)])?;
                    }

                    if let Some(p1) = part_one {
                        writer.write_patches(&[JsonlWriter::<io::Stdout>::replace_patch("/part_one/status", "running")])?;
                        writer.write_patches(&[
                            JsonlWriter::<io::Stdout>::replace_patch("/part_one/status", "complete"),
                            JsonlWriter::<io::Stdout>::replace_patch("/part_one/value", &p1.value),
                            JsonlWriter::<io::Stdout>::replace_patch("/part_one/duration_ms", p1.duration as u64),
                        ])?;
                    }

                    if let Some(p2) = part_two {
                        writer.write_patches(&[JsonlWriter::<io::Stdout>::replace_patch("/part_two/status", "running")])?;
                        writer.write_patches(&[
                            JsonlWriter::<io::Stdout>::replace_patch("/part_two/status", "complete"),
                            JsonlWriter::<io::Stdout>::replace_patch("/part_two/value", &p2.value),
                            JsonlWriter::<io::Stdout>::replace_patch("/part_two/duration_ms", p2.duration as u64),
                        ])?;
                    }

                    writer.write_patches(&[JsonlWriter::<io::Stdout>::replace_patch("/status", "complete")])?;
                }
            }
            Ok(())
        }
        Err(error) => {
            let _ = crate::external_functions::disable_console_capture();
            let error_output = format_error_json(source, &error);
            writer.write_patches(&[
                JsonlWriter::<io::Stdout>::replace_patch("/status", "error"),
                JsonlWriter::<io::Stdout>::add_patch("/error", &error_output),
            ])?;
            std::process::exit(2);
        }
    }
}

fn aoc_test(source: &str, source_path: Option<&str>, include_slow: bool, output_mode: OutputMode) -> Result<()> {
    // Enable console capture for JSON/JSONL modes
    if output_mode != OutputMode::Text {
        crate::external_functions::enable_console_capture();
    }

    let mut runner = AoCRunner::new_with_external_functions(CliTime {}, &crate::external_functions::definitions());

    match output_mode {
        OutputMode::Text => match runner.test(source, include_slow) {
            Ok(test_cases) => {
                let mut exit_code = 0;

                for (number, test_case) in test_cases.iter().enumerate() {
                    if number > 0 {
                        println!()
                    }

                    if test_case.skipped {
                        println!("\x1b[4mTestcase #{}\x1b[0m \x1b[33m(skipped)\x1b[0m", number + 1);
                        continue;
                    }

                    if test_case.slow {
                        println!("\x1b[4mTestcase #{}\x1b[0m \x1b[33m(slow)\x1b[0m", number + 1);
                    } else {
                        println!("\x1b[4mTestcase #{}\x1b[0m", number + 1);
                    }

                    if test_case.part_one.is_none() && test_case.part_two.is_none() {
                        println!("No expectations");
                        continue;
                    }

                    if let Some(part_one) = &test_case.part_one {
                        if part_one.passed {
                            println!("Part 1: {} \x1b[32m✔\x1b[0m", part_one.actual);
                        } else {
                            println!(
                                "Part 1: {} \x1b[31m✘ (Expected: {})\x1b[0m",
                                part_one.actual, part_one.expected
                            );
                            exit_code = 3;
                        }
                    }

                    if let Some(part_two) = &test_case.part_two {
                        if part_two.passed {
                            println!("Part 2: {} \x1b[32m✔\x1b[0m", part_two.actual);
                        } else {
                            println!(
                                "Part 2: {} \x1b[31m✘ (Expected: {})\x1b[0m",
                                part_two.actual, part_two.expected
                            );
                            exit_code = 3;
                        }
                    }
                }

                if exit_code != 0 {
                    std::process::exit(exit_code);
                }

                Ok(())
            }
            Err(error) => {
                print_error(source_path.unwrap_or("<stdin>"), source, error);
                std::process::exit(2);
            }
        },
        OutputMode::Json => match runner.test(source, include_slow) {
            Ok(test_cases) => {
                let console = crate::external_functions::disable_console_capture();
                let (has_part_one, has_part_two) = output::is_solution_source(source);
                let json = output::format_test_json(&test_cases, has_part_one, has_part_two, console);
                println!("{}", json);

                // Determine exit code based on test results
                let has_failures = test_cases.iter().any(|tc| {
                    !tc.skipped
                        && (tc.part_one.as_ref().is_some_and(|p| !p.passed)
                            || tc.part_two.as_ref().is_some_and(|p| !p.passed))
                });
                if has_failures {
                    std::process::exit(3);
                }
                Ok(())
            }
            Err(error) => {
                let _ = crate::external_functions::disable_console_capture();
                let json = serde_json::to_string(&output::format_error_json(source, &error)).unwrap();
                println!("{}", json);
                std::process::exit(2);
            }
        },
        OutputMode::Jsonl => aoc_test_jsonl(source, &mut runner, include_slow),
    }
}

fn aoc_test_jsonl(source: &str, runner: &mut AoCRunner<CliTime>, include_slow: bool) -> Result<()> {
    use output::*;
    use std::io;

    let mut writer = JsonlWriter::new(io::stdout());
    let (has_part_one, has_part_two) = is_solution_source(source);

    // We need to determine total tests before running, which requires parsing
    // For now, run tests and emit initial state with test count
    match runner.test(source, include_slow) {
        Ok(test_cases) => {
            let console = crate::external_functions::disable_console_capture();
            let total = test_cases.len() as u32;

            // Emit initial state
            let initial_tests: Vec<JsonlTestCaseInitial> = test_cases
                .iter()
                .enumerate()
                .map(|(i, tc)| JsonlTestCaseInitial {
                    index: (i + 1) as u32,
                    slow: tc.slow,
                    status: "pending",
                    part_one: None,
                    part_two: None,
                })
                .collect();

            let initial = JsonlTestInitial {
                output_type: "test",
                status: "pending",
                success: None,
                summary: TestSummary {
                    total,
                    passed: 0,
                    failed: 0,
                    skipped: 0,
                },
                tests: initial_tests,
                console: vec![],
            };
            writer.write_initial(&initial)?;

            // Emit running status
            writer.write_patches(&[JsonlWriter::<io::Stdout>::replace_patch("/status", "running")])?;

            // Emit console entries
            for entry in &console {
                writer.write_patches(&[JsonlWriter::<io::Stdout>::add_patch("/console/-", entry)])?;
            }

            // Emit test results
            let mut passed = 0u32;
            let mut failed = 0u32;
            let mut skipped = 0u32;

            for (i, tc) in test_cases.iter().enumerate() {
                let path_prefix = format!("/tests/{}", i);

                if tc.skipped {
                    skipped += 1;
                    writer.write_patches(&[
                        JsonlWriter::<io::Stdout>::replace_patch(&format!("{}/status", path_prefix), "skipped"),
                        JsonlWriter::<io::Stdout>::replace_patch("/summary/skipped", skipped),
                    ])?;
                } else {
                    // Emit running
                    writer.write_patches(&[JsonlWriter::<io::Stdout>::replace_patch(
                        &format!("{}/status", path_prefix),
                        "running",
                    )])?;

                    // Determine result
                    let part_one_passed = tc.part_one.as_ref().is_none_or(|p| p.passed);
                    let part_two_passed = tc.part_two.as_ref().is_none_or(|p| p.passed);
                    let all_passed = part_one_passed && part_two_passed;

                    if all_passed {
                        passed += 1;
                    } else {
                        failed += 1;
                    }

                    // Build patches
                    let mut patches = vec![JsonlWriter::<io::Stdout>::replace_patch(
                        &format!("{}/status", path_prefix),
                        "complete",
                    )];

                    if has_part_one {
                        if let Some(p1) = &tc.part_one {
                            patches.push(JsonlWriter::<io::Stdout>::replace_patch(
                                &format!("{}/part_one", path_prefix),
                                JsonTestPartResult {
                                    passed: p1.passed,
                                    expected: p1.expected.clone(),
                                    actual: p1.actual.clone(),
                                },
                            ));
                        }
                    }

                    if has_part_two {
                        if let Some(p2) = &tc.part_two {
                            patches.push(JsonlWriter::<io::Stdout>::replace_patch(
                                &format!("{}/part_two", path_prefix),
                                JsonTestPartResult {
                                    passed: p2.passed,
                                    expected: p2.expected.clone(),
                                    actual: p2.actual.clone(),
                                },
                            ));
                        }
                    }

                    if all_passed {
                        patches.push(JsonlWriter::<io::Stdout>::replace_patch("/summary/passed", passed));
                    } else {
                        patches.push(JsonlWriter::<io::Stdout>::replace_patch("/summary/failed", failed));
                    }

                    writer.write_patches(&patches)?;
                }
            }

            // Emit completion
            let success = failed == 0;
            writer.write_patches(&[
                JsonlWriter::<io::Stdout>::replace_patch("/status", "complete"),
                JsonlWriter::<io::Stdout>::replace_patch("/success", success),
            ])?;

            if !success {
                std::process::exit(3);
            }
            Ok(())
        }
        Err(error) => {
            let _ = crate::external_functions::disable_console_capture();
            // Emit error - for tests we need a minimal initial state first
            let initial = JsonlTestInitial {
                output_type: "test",
                status: "pending",
                success: None,
                summary: TestSummary {
                    total: 0,
                    passed: 0,
                    failed: 0,
                    skipped: 0,
                },
                tests: vec![],
                console: vec![],
            };
            writer.write_initial(&initial)?;
            writer.write_patches(&[JsonlWriter::<io::Stdout>::replace_patch("/status", "running")])?;

            let error_output = format_error_json(source, &error);
            writer.write_patches(&[
                JsonlWriter::<io::Stdout>::replace_patch("/status", "error"),
                JsonlWriter::<io::Stdout>::add_patch("/error", &error_output),
            ])?;
            std::process::exit(2);
        }
    }
}

fn handle_format(matches: &getopts::Matches, to_stdout: bool, write_file: bool, check_only: bool) -> Result<()> {
    // Determine source: -e flag > file argument > stdin
    let (source, source_path): (String, Option<String>) = if let Some(eval_script) = matches.opt_str("e") {
        (eval_script, None)
    } else if matches.free.len() == 1 {
        let path = &matches.free[0];
        let canonical = fs::canonicalize(path)?;
        let source = fs::read_to_string(&canonical)?;
        (source, Some(canonical.to_string_lossy().into_owned()))
    } else if !atty::is(atty::Stream::Stdin) {
        let mut source = String::new();
        std::io::stdin().read_to_string(&mut source)?;
        (source, None)
    } else {
        eprintln!("Error: No source provided for formatting");
        std::process::exit(1);
    };

    match santa_lang::format(&source) {
        Ok(formatted) => {
            if check_only {
                if formatted == source {
                    // Already formatted
                    std::process::exit(0);
                } else {
                    // Needs formatting
                    if let Some(path) = &source_path {
                        eprintln!("{} needs formatting", path);
                    } else {
                        eprintln!("Input needs formatting");
                    }
                    std::process::exit(1);
                }
            } else if write_file {
                if let Some(path) = &source_path {
                    fs::write(path, &formatted)?;
                    println!("Formatted {}", path);
                } else {
                    eprintln!("Error: --fmt-write requires a file path");
                    std::process::exit(1);
                }
            } else if to_stdout {
                print!("{}", formatted);
            }
            Ok(())
        }
        Err(error) => {
            eprintln!("Parse error: {}", error.message);
            std::process::exit(2);
        }
    }
}

fn print_error(source_path: &str, source: &str, error: RunErr) {
    let (line, column) = calculate_line_column(source, error.source);

    println!("\x1b[31m{}\x1b[0m\n", error.message);

    for (position, source_line) in source.split('\n').enumerate() {
        if line > 1 && (position < line - 2 || position > line + 2) {
            continue;
        }

        if position == line {
            println!("  \x1b[37m{:0>2}: {}\x1b[0m", position + 1, source_line);
            println!(
                "  \x1b[31m{}\x1b[0m",
                " ".repeat(format!("{:0>2}: ", position + 1).len() + column) + "^~~"
            );
        } else {
            println!("  \x1b[2m{:0>2}: {}\x1b[0m", position + 1, source_line);
        }
    }

    println!("\n{}:\x1b[32m{}:{}\x1b[0m", source_path, line + 1, column + 1);

    if !error.trace.is_empty() {
        for location in error.trace {
            let (line, column) = calculate_line_column(source, location);
            println!(
                "  \x1b[2m{}:\x1b[0m\x1b[32m{}:{}\x1b[0m",
                &source[location.start..location.end]
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" "),
                line + 1,
                column + 1
            );
        }
    }
}

fn calculate_line_column(source: &str, location: Location) -> (usize, usize) {
    let mut line = 0;
    let mut column = 0;

    for (position, character) in source.chars().enumerate() {
        if position == location.start {
            return (line, column);
        }

        column += 1;
        if character == '\n' {
            line += 1;
            column = 0;
        }
    }

    // Location is at or beyond end of source (e.g., EOF error)
    (line, column)
}
