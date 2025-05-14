use std::io;
use beetle::Router;

#[tokio::main]
async fn main() -> io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let routes = Router::new();


    beetle::listen("0.0.0.0:3000", routes).await
}
