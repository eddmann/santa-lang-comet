mod external_functions;

use ext_php_rs::prelude::*;
use ext_php_rs::types::{ZendHashTable, Zval};
use lang::{AoCRunner, Environment, Evaluator, Lexer, Parser, RunEvaluation, Time};
use std::time::{SystemTime, UNIX_EPOCH};

struct PhpTime {}
impl Time for PhpTime {
    fn now(&self) -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis()
    }
}

#[php_function(optional = "cwd")]
pub fn santa_aoc_run(source: &str, cwd: Option<String>) -> PhpResult<Zval> {
    if let Some(dir) = cwd {
        std::env::set_current_dir(dir).unwrap();
    }

    let mut runner = AoCRunner::new_with_external_functions(PhpTime {}, &crate::external_functions::definitions());

    match runner.run(source) {
        Ok(RunEvaluation::Script(result)) => {
            let mut output_ht = ZendHashTable::new();
            output_ht.insert("value", result.value)?;
            output_ht.insert("duration", result.duration as u64)?;

            let mut zval = Zval::new();
            zval.set_hashtable(output_ht);
            Ok(zval)
        }
        Ok(RunEvaluation::Solution { part_one, part_two }) => {
            let mut output_ht = ZendHashTable::new();

            if let Some(part_one) = part_one {
                let mut part_one_ht = ZendHashTable::new();
                part_one_ht.insert("value", part_one.value)?;
                part_one_ht.insert("duration", part_one.duration as u64)?;
                output_ht.insert("part_one", part_one_ht)?;
            }

            if let Some(part_two) = part_two {
                let mut part_two_ht = ZendHashTable::new();
                part_two_ht.insert("value", part_two.value)?;
                part_two_ht.insert("duration", part_two.duration as u64)?;
                output_ht.insert("part_two", part_two_ht)?;
            }

            let mut zval = Zval::new();
            zval.set_hashtable(output_ht);
            Ok(zval)
        }
        Err(error) => Err(error.message.into()),
    }
}

#[php_function(optional = "cwd")]
pub fn santa_aoc_test(source: &str, cwd: Option<String>, include_slow: Option<bool>) -> PhpResult<Zval> {
    if let Some(dir) = cwd {
        std::env::set_current_dir(dir).unwrap();
    }

    let mut runner = AoCRunner::new_with_external_functions(PhpTime {}, &crate::external_functions::definitions());

    match runner.test(source, include_slow.unwrap_or(false)) {
        Ok(test_cases) => {
            let mut output_ht = ZendHashTable::new();

            for test_case in test_cases {
                let mut test_case_ht = ZendHashTable::new();

                if let Some(part_one) = test_case.part_one {
                    let mut part_one_ht = ZendHashTable::new();
                    part_one_ht.insert("actual", part_one.actual)?;
                    part_one_ht.insert("expected", part_one.expected)?;
                    part_one_ht.insert("passed", part_one.passed)?;
                    test_case_ht.insert("part_one", part_one_ht)?;
                }

                if let Some(part_two) = test_case.part_two {
                    let mut part_two_ht = ZendHashTable::new();
                    part_two_ht.insert("actual", part_two.actual)?;
                    part_two_ht.insert("expected", part_two.expected)?;
                    part_two_ht.insert("passed", part_two.passed)?;
                    test_case_ht.insert("part_two", part_two_ht)?;
                }

                output_ht.push(test_case_ht)?;
            }

            let mut zval = Zval::new();
            zval.set_hashtable(output_ht);
            Ok(zval)
        }
        Err(error) => Err(error.message.into()),
    }
}

#[php_function(optional = "cwd")]
pub fn santa_evaluate(expression: &str, cwd: Option<String>) -> PhpResult<String> {
    if let Some(dir) = cwd {
        std::env::set_current_dir(dir).unwrap();
    }

    let mut evaluator = Evaluator::new_with_external_functions(&crate::external_functions::definitions());

    let lexer = Lexer::new(expression);
    let mut parser = Parser::new(lexer);
    let program = parser.parse().map_err(|error| error.message)?;

    match evaluator.evaluate_with_environment(&program, Environment::new()) {
        Ok(evaluated) => Ok(evaluated.to_string()),
        Err(error) => Err(error.message.into()),
    }
}

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
