#![allow(clippy::collapsible_if)]

use crate::output::ConsoleEntry;
use santa_lang::{Arguments, Evaluation, ExpressionKind, ExternalFnDef, Location, Object, RuntimeErr};
use std::cell::RefCell;
use std::env;
use std::fs;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

thread_local! {
    /// Console buffer for capturing puts() output in JSON/JSONL mode.
    /// When Some, puts() will append to this buffer instead of printing.
    static CONSOLE_BUFFER: RefCell<Option<Vec<ConsoleEntry>>> = const { RefCell::new(None) };

    /// Start time for calculating timestamps in console entries.
    static START_TIME_MS: RefCell<u128> = const { RefCell::new(0) };
}

/// Enable console capture mode. Returns any previously captured entries.
pub fn enable_console_capture() -> Vec<ConsoleEntry> {
    let start = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();

    START_TIME_MS.with(|t| *t.borrow_mut() = start);
    CONSOLE_BUFFER.with(|buf| buf.borrow_mut().replace(Vec::new()).unwrap_or_default())
}

/// Disable console capture and return captured entries.
pub fn disable_console_capture() -> Vec<ConsoleEntry> {
    CONSOLE_BUFFER.with(|buf| buf.borrow_mut().take().unwrap_or_default())
}

/// Check if console capture is enabled.
#[allow(dead_code)]
pub fn is_console_capture_enabled() -> bool {
    CONSOLE_BUFFER.with(|buf| buf.borrow().is_some())
}

/// Get current timestamp in milliseconds since capture started.
fn get_timestamp_ms() -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();

    START_TIME_MS.with(|t| (now - *t.borrow()) as u64)
}

pub fn definitions() -> Vec<ExternalFnDef> {
    vec![
        (
            "puts".to_owned(),
            vec![ExpressionKind::RestIdentifier("values".to_owned())],
            Rc::new(puts),
        ),
        (
            "read".to_owned(),
            vec![ExpressionKind::Identifier("path".to_owned())],
            Rc::new(read),
        ),
    ]
}

/// Format an object value for puts() output.
/// Strings are displayed without surrounding quotes (unlike Display trait).
fn format_puts_value(value: &Object) -> String {
    match value {
        Object::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn puts(arguments: Arguments, _source: Location) -> Evaluation {
    match &**arguments.get("values").unwrap() {
        Object::List(values) => {
            // Skip if no values (spec says no event for puts with no args)
            if values.is_empty() {
                return Ok(Rc::new(Object::Nil));
            }

            // Build the message string (space-separated values)
            // Use format_puts_value to avoid quotes around strings
            let message: String = values
                .iter()
                .map(|v| format_puts_value(v))
                .collect::<Vec<_>>()
                .join(" ");

            // Check if we should capture or print
            let captured = CONSOLE_BUFFER.with(|buf| {
                if let Some(ref mut buffer) = *buf.borrow_mut() {
                    buffer.push(ConsoleEntry {
                        timestamp_ms: get_timestamp_ms(),
                        message: message.clone(),
                    });
                    true
                } else {
                    false
                }
            });

            if !captured {
                // Print to stdout as normal
                println!("{}", message);
            }

            Ok(Rc::new(Object::Nil))
        }
        _ => unreachable!(),
    }
}

fn read(arguments: Arguments, source: Location) -> Evaluation {
    match &**arguments.get("path").unwrap() {
        Object::String(path) => match Url::parse(path) {
            Ok(uri) if uri.scheme() == "aoc" => {
                let cache = format!(
                    "aoc{}_day{:0>2}.input",
                    uri.host().unwrap(),
                    uri.path().replace('/', "")
                );

                if let Ok(content) = fs::read_to_string(&cache) {
                    return Ok(Rc::new(Object::String(content)));
                }

                let token = match env::var_os("SANTA_CLI_SESSION_TOKEN") {
                    Some(token) => token.into_string().unwrap(),
                    None => {
                        return Err(RuntimeErr {
                            message: "Missing SANTA_CLI_SESSION_TOKEN environment variable".to_owned(),
                            source,
                            trace: vec![],
                        });
                    }
                };

                let request = ureq::get(&format!(
                    "https://adventofcode.com/{}/day{}/input",
                    uri.host().unwrap(),
                    uri.path()
                ))
                .set("Cookie", &format!("session={}", token));
                if let Ok(response) = request.call() {
                    if let Ok(input) = response.into_string() {
                        fs::write(cache, input.trim_end().as_bytes()).expect("");
                        return Ok(Rc::new(Object::String(input.trim_end().to_string())));
                    }
                }

                Err(RuntimeErr {
                    message: format!("Failed to read AoC input: {}", path),
                    source,
                    trace: vec![],
                })
            }
            Ok(_) => {
                if let Ok(response) = ureq::get(path).call() {
                    if let Ok(body) = response.into_string() {
                        return Ok(Rc::new(Object::String(body)));
                    }
                }

                Err(RuntimeErr {
                    message: format!("Failed to read URL: {}", path),
                    source,
                    trace: vec![],
                })
            }
            Err(_) => {
                if let Ok(content) = fs::read_to_string(path) {
                    return Ok(Rc::new(Object::String(content)));
                }

                Err(RuntimeErr {
                    message: format!("Failed to read file: {}", path),
                    source,
                    trace: vec![],
                })
            }
        },
        object => Err(RuntimeErr {
            message: format!(
                "Invalid arguments: read(path: {})\nExpected arguments:\nread(path: String)",
                object.name(),
            ),
            source,
            trace: vec![],
        }),
    }
}
