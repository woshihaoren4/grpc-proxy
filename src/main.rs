mod cmd;

#[tokio::main]
async fn main() {
    cmd::start().await;
}
