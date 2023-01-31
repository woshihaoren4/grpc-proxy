mod cmd;
mod config;
mod infra;
mod util;
mod app;

#[tokio::main]
async fn main() {
    cmd::start().await;
}
