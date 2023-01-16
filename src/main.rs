mod cmd;
mod infra;
mod util;

#[tokio::main]
async fn main() {
    cmd::start().await;
}
