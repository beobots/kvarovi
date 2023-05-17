#[tokio::main]
async fn main() {
    let _ = electricity::collect_data().await;

    // let _ = electricity::start_scheduler().await;

    // tokio::time::sleep(core::time::Duration::from_secs(100)).await;
}
