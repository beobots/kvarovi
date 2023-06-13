#[tokio::main]
async fn main() {
    println!("Console main");
    electricity::console_lib().await;
}
