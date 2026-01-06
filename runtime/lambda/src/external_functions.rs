#![allow(clippy::collapsible_if)]

use lang::{Arguments, Evaluation, ExpressionKind, ExternalFnDef, Location, Object, RuntimeErr};
use std::fs;
use std::rc::Rc;

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

fn puts(arguments: Arguments, _source: Location) -> Evaluation {
    match &**arguments.get("values").unwrap() {
        Object::List(values) => {
            for value in values {
                print!("{} ", value);
            }
            println!();
            Ok(Rc::new(Object::Nil))
        }
        _ => unreachable!(),
    }
}

fn read(arguments: Arguments, source: Location) -> Evaluation {
    match &**arguments.get("path").unwrap() {
        Object::String(path) => {
            if let Ok(content) = fs::read_to_string(path) {
                return Ok(Rc::new(Object::String(content)));
            }

            if let Ok(response) = ureq::get(path).call() {
                if let Ok(body) = response.into_string() {
                    return Ok(Rc::new(Object::String(body)));
                }
            }

            Err(RuntimeErr {
                message: format!("Failed to read: {}", path),
                source,
                trace: vec![],
            })
        }
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
