use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::Serialize;
use serde_json::Value;


#[derive(Serialize)]
struct Response {
    req_id: String,
    msg: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(my_handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

pub(crate) async fn my_handler(event: LambdaEvent<Value>) -> Result<Response, Error> {
    let command: String = event.payload.to_string();

    let resp = Response {
        req_id: event.context.request_id,
        msg: format!("Command {command} executed."),
    };
    Ok(resp)
}