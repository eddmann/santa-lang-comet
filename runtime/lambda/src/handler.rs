use lang::{Environment, Evaluator, Lexer, Object, Parser};
use std::env;
use std::rc::Rc;

pub struct HandlerErr {
    pub name: String,
    pub message: String,
}

pub type Handler = Box<dyn FnMut(Rc<Object>, Rc<Object>) -> Result<Rc<Object>, HandlerErr>>;

pub fn init() -> Result<Handler, HandlerErr> {
    let (source, section_name) = get_handler()?;

    let lexer = Lexer::new(&source);
    let mut parser = Parser::new(lexer);
    let program = parser.parse().map_err(|error| HandlerErr {
        name: "Runtime.InitialisationFailed".to_owned(),
        message: error.message,
    })?;

    let global_environment = Environment::new();
    let mut evaluator = Evaluator::new_with_external_functions(&crate::external_functions::definitions());

    evaluator
        .evaluate_with_environment(&program, Rc::clone(&global_environment))
        .map_err(|error| HandlerErr {
            name: "Runtime.InitialisationFailed".to_owned(),
            message: error.message,
        })?;

    let section = global_environment
        .borrow()
        .get_sections(&section_name)
        .first()
        .ok_or_else(|| HandlerErr {
            name: "Runtime.HandlerNotFound".to_owned(),
            message: format!("Section not found: {}", section_name),
        })?
        .clone();

    Ok(Box::new(move |event: Rc<Object>, context: Rc<Object>| {
        let environment = Environment::from(Rc::clone(&global_environment));
        environment.borrow_mut().set_variable("event", event);
        environment.borrow_mut().set_variable("context", context);

        evaluator
            .evaluate_with_environment(&section, Rc::clone(&environment))
            .map_err(|error| HandlerErr {
                name: "Invocation.Error".to_owned(),
                message: error.message,
            })
    }))
}

fn get_handler() -> Result<(String, String), HandlerErr> {
    let handler = get_env("_HANDLER")?;
    let parts: Vec<&str> = handler.splitn(2, '.').collect();

    let script_name = parts.first().unwrap_or(&"handler");
    let script_path = format!("{}/{}.santa", get_env("LAMBDA_TASK_ROOT")?, script_name);
    let source = std::fs::read_to_string(&script_path).map_err(|_| HandlerErr {
        name: "Runtime.HandlerNotFound".to_owned(),
        message: format!("Script not found: {}", script_path),
    })?;

    let section_name = parts.get(1).unwrap_or(&"handler").to_string();

    Ok((source, section_name))
}

fn get_env(name: &str) -> Result<String, HandlerErr> {
    env::var(name).map_err(|_| HandlerErr {
        name: "Runtime.EnvNotFound".to_owned(),
        message: format!("Enviornment variable not found: {}", name),
    })
}
