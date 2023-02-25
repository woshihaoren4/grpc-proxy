mod app;
mod cmd;
mod config;
mod infra;
mod util;

#[tokio::main]
async fn main() {
    cmd::start().await;
}
