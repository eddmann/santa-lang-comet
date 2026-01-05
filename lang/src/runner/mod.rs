use crate::evaluator::{Environment, EnvironmentErr, EnvironmentRef, Evaluator, ExternalFnDef, Object, RuntimeErr};
use crate::lexer::{Lexer, Location};
use crate::parser::{Parser, ParserErr, ast::Section};
use std::rc::Rc;

#[cfg(test)]
mod tests;

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RunErr {
    pub message: String,
    pub source: Location,
    pub trace: Vec<Location>,
}

impl From<RuntimeErr> for RunErr {
    fn from(error: RuntimeErr) -> Self {
        RunErr {
            message: error.message,
            source: error.source,
            trace: error.trace,
        }
    }
}

impl From<ParserErr> for RunErr {
    fn from(error: ParserErr) -> Self {
        RunErr {
            message: error.message,
            source: error.source,
            trace: vec![],
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RunResult {
    pub value: String,
    pub duration: u128,
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum RunEvaluation {
    Solution {
        part_one: Option<RunResult>,
        part_two: Option<RunResult>,
    },
    Script(RunResult),
}

struct SourceEvaluation {
    environment: EnvironmentRef,
    result: Rc<Object>,
    part_one: Option<Rc<Section>>,
    part_two: Option<Rc<Section>>,
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TestCase {
    pub part_one: Option<TestCaseResult>,
    pub part_two: Option<TestCaseResult>,
    pub slow: bool,
    pub skipped: bool,
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TestCaseResult {
    pub expected: String,
    pub actual: String,
    pub passed: bool,
}

pub struct AoCRunner<T: Time> {
    evaluator: Evaluator,
    time: T,
}

pub trait Time {
    fn now(&self) -> u128;
}

impl<T: Time> AoCRunner<T> {
    pub fn new(time: T) -> Self {
        Self {
            evaluator: Evaluator::new(),
            time,
        }
    }

    pub fn new_with_external_functions(time: T, external_functions: &[ExternalFnDef]) -> Self {
        Self {
            evaluator: Evaluator::new_with_external_functions(external_functions),
            time,
        }
    }

    pub fn run(&mut self, source: &str) -> Result<RunEvaluation, RunErr> {
        let start = self.time.now();

        let evaluation = self.evaluate_source(source)?;

        if evaluation.part_one.is_none() && evaluation.part_two.is_none() {
            return Ok(RunEvaluation::Script(RunResult {
                value: evaluation.result.to_string(),
                duration: self.elapsed_millis(start),
            }));
        }

        let input = evaluation.environment.borrow().get_sections("input");
        if input.len() > 1 {
            return Err(RunErr {
                message: "Expected a single 'input' section".to_owned(),
                source: input[1].source,
                trace: vec![],
            });
        }
        let evaluated_input: Option<Rc<Object>> = if input.len() == 1 {
            Some(
                self.evaluator
                    .evaluate_with_environment(&input[0], Rc::clone(&evaluation.environment))?,
            )
        } else {
            None
        };

        let mut part_one_result: Option<RunResult> = None;
        if let Some(part_one) = evaluation.part_one {
            let start = self.time.now();
            let value = self.evaluate_solution(&part_one, Rc::clone(&evaluation.environment), &evaluated_input)?;
            part_one_result = Some(RunResult {
                value: value.to_string(),
                duration: self.elapsed_millis(start),
            });
        }

        let mut part_two_result: Option<RunResult> = None;
        if let Some(part_two) = evaluation.part_two {
            let start = self.time.now();
            let value = self.evaluate_solution(&part_two, Rc::clone(&evaluation.environment), &evaluated_input)?;
            part_two_result = Some(RunResult {
                value: value.to_string(),
                duration: self.elapsed_millis(start),
            });
        }

        Ok(RunEvaluation::Solution {
            part_one: part_one_result,
            part_two: part_two_result,
        })
    }

    pub fn test(&mut self, source: &str, include_slow: bool) -> Result<Vec<TestCase>, RunErr> {
        let evaluation = self.evaluate_source(source)?;

        let mut results = vec![];

        for (test, attributes) in evaluation.environment.borrow().get_sections_with_attributes("test") {
            let is_slow = Environment::section_has_attribute(&attributes, "slow");

            // Skip slow tests unless explicitly requested - but still include in results
            if is_slow && !include_slow {
                results.push(TestCase {
                    part_one: None,
                    part_two: None,
                    slow: true,
                    skipped: true,
                });
                continue;
            }

            let test_environment = Environment::from(Rc::clone(&evaluation.environment));
            let _test_result = self
                .evaluator
                .evaluate_with_environment(&test, Rc::clone(&test_environment))?;

            let expected_part_one = test_environment.borrow().get_sections("part_one");
            let expected_part_two = test_environment.borrow().get_sections("part_two");

            if expected_part_one.is_empty() && expected_part_two.is_empty() {
                continue;
            }

            if expected_part_one.len() > 1 {
                return Err(RunErr {
                    message: "Expected a single 'part_one' assertion".to_owned(),
                    source: expected_part_one[1].source,
                    trace: vec![],
                });
            }

            if expected_part_two.len() > 1 {
                return Err(RunErr {
                    message: "Expected a single 'part_two' assertion".to_owned(),
                    source: expected_part_two[1].source,
                    trace: vec![],
                });
            }

            let input = test_environment.borrow().get_sections("input");
            if input.len() > 1 {
                return Err(RunErr {
                    message: "Expected a single 'input' fixture".to_owned(),
                    source: input[1].source,
                    trace: vec![],
                });
            }
            let evaluated_input: Option<Rc<Object>> = if input.len() == 1 {
                Some(
                    self.evaluator
                        .evaluate_with_environment(&input[0].clone(), Rc::clone(&evaluation.environment))?,
                )
            } else {
                None
            };

            let mut part_one_result: Option<TestCaseResult> = None;
            if expected_part_one.len() == 1 {
                if let Some(part_one) = &evaluation.part_one {
                    let expected = self
                        .evaluator
                        .evaluate_with_environment(&expected_part_one[0], Rc::clone(&test_environment))?;
                    let value = self.evaluate_solution(part_one, Rc::clone(&test_environment), &evaluated_input)?;
                    part_one_result = Some(TestCaseResult {
                        expected: expected.to_string(),
                        actual: value.to_string(),
                        passed: expected == value,
                    });
                }
            }

            let mut part_two_result: Option<TestCaseResult> = None;
            if expected_part_two.len() == 1 {
                if let Some(part_two) = &evaluation.part_two {
                    let expected = self
                        .evaluator
                        .evaluate_with_environment(&expected_part_two[0], Rc::clone(&test_environment))?;
                    let value = self.evaluate_solution(part_two, Rc::clone(&test_environment), &evaluated_input)?;
                    part_two_result = Some(TestCaseResult {
                        expected: expected.to_string(),
                        actual: value.to_string(),
                        passed: expected == value,
                    });
                }
            }

            results.push(TestCase {
                part_one: part_one_result,
                part_two: part_two_result,
                slow: is_slow,
                skipped: false,
            });
        }

        Ok(results)
    }

    fn elapsed_millis(&self, start: u128) -> u128 {
        self.time.now() - start
    }

    fn evaluate_solution(
        &mut self,
        section: &Section,
        environment: EnvironmentRef,
        input: &Option<Rc<Object>>,
    ) -> Result<Rc<Object>, RuntimeErr> {
        let section_environment = Environment::from(environment);

        if let Some(input) = input {
            match section_environment
                .borrow_mut()
                .declare_variable("input", Rc::clone(input), false)
            {
                Ok(_) => {}
                Err(EnvironmentErr { message }) => {
                    return Err(RuntimeErr {
                        message,
                        source: section.source,
                        trace: vec![],
                    });
                }
            };
        }

        self.evaluator
            .evaluate_with_environment(section, Rc::clone(&section_environment))
    }

    fn evaluate_source(&mut self, source: &str) -> Result<SourceEvaluation, RunErr> {
        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer);
        let program = parser.parse()?;
        let environment = Environment::new();

        let result = self
            .evaluator
            .evaluate_with_environment(&program, Rc::clone(&environment))?;
        let part_one = environment.borrow().get_sections("part_one");
        let part_two = environment.borrow().get_sections("part_two");

        if part_one.len() > 1 {
            return Err(RunErr {
                message: "Expected single 'part_one' solution".to_owned(),
                source: part_one[1].source,
                trace: vec![],
            });
        }

        if part_two.len() > 1 {
            return Err(RunErr {
                message: "Expected single 'part_two' solution".to_owned(),
                source: part_two[1].source,
                trace: vec![],
            });
        }

        Ok(SourceEvaluation {
            environment,
            result,
            part_one: if part_one.len() == 1 {
                Some(Rc::clone(&part_one[0]))
            } else {
                None
            },
            part_two: if part_two.len() == 1 {
                Some(Rc::clone(&part_two[0]))
            } else {
                None
            },
        })
    }
}
