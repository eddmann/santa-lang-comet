mod external_functions;
mod handler;

use lang::Object;
use std::collections::HashMap;
use std::env;
use std::rc::Rc;

// https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html

fn main() -> ! {
    let runtime_api = format!(
        "http://{}/2018-06-01/runtime",
        env::var("AWS_LAMBDA_RUNTIME_API").expect("Enviornment variable not found: AWS_LAMBDA_RUNTIME_API")
    );

    let mut handler = match handler::init() {
        Ok(initialised) => initialised,
        Err(error) => {
            ureq::post(&format!("{}/init/error", runtime_api))
                .send_json(ureq::json!({
                    "errorType": error.name,
                    "errorMessage": error.message,
                }))
                .expect("Failed to send error to Lambda API");
            panic!("Failed to initialise");
        }
    };

    loop {
        let response = ureq::get(&format!("{}/invocation/next", runtime_api))
            .call()
            .expect("Failed to get next invocation from Lambda API");
        let request_id = response
            .header("Lambda-Runtime-Aws-Request-Id")
            .expect("Invocation request id not found")
            .to_owned();

        let event: Rc<Object> = response.into_json().expect("Failed to parse event");

        #[allow(clippy::mutable_key_type)]
        let mut context_map: HashMap<Rc<Object>, Rc<Object>> = HashMap::new();
        context_map.insert(
            Rc::new(Object::String("request_id".to_owned())),
            Rc::new(Object::String(request_id.to_owned())),
        );
        let context = Rc::new(Object::Dictionary(context_map.into()));

        match handler(event, context) {
            Ok(response) => {
                ureq::post(&format!("{}/invocation/{}/response", runtime_api, request_id))
                    .send_json(&response)
                    .expect("Failed to send response to Lambda API");
            }
            Err(error) => {
                ureq::post(&format!("{}/invocation/{}/error", runtime_api, request_id))
                    .send_json(ureq::json!({
                        "errorType": error.name,
                        "errorMessage": error.message,
                    }))
                    .expect("Failed to send error to Lambda API");
            }
        };
    }
}
