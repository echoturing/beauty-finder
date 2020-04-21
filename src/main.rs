use libxml::parser::*;

use beauty_finder::crawler::run;

#[tokio::main]
async fn main() {
    run().await;
}
