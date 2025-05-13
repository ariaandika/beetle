use std::io;
use tcio::Router;

#[tokio::main]
async fn main() -> io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let routes = Router::new();


    tcio::listen("0.0.0.0:3000", routes).await
}
