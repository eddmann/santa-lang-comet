use crate::evaluator::LazySequence;
use crate::evaluator::function::Function;
use crate::evaluator::object::Object;
use std::cell::RefCell;
use std::rc::Rc;

builtin! {
    range(from, to, step) match {
        (Object::Integer(from), Object::Integer(to), Object::Integer(step)) => {
            Ok(Rc::new(Object::LazySequence(LazySequence::inclusive_range_with_step(*from, *to, *step))))
        }
    }
}

builtin! {
    type_name(value) {
        Ok(Rc::new(Object::String(value.name())))
    }
}

builtin! {
    id(value) {
        Ok(Rc::clone(value))
    }
}

builtin! {
    memoize(function) [evaluator, source] match {
        Object::Function(Function::Closure { parameters, body, environment, }) => {
            let function = Function::MemoizedClosure {
                parameters: parameters.clone(),
                body: body.clone(),
                environment: Rc::clone(environment),
                cache: Rc::new(RefCell::new(std::collections::HashMap::default()))
            };
            Ok(Rc::new(Object::Function(function)))
        }
    }
}
