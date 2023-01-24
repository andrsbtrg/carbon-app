use ec3api;

use std::env;

fn main() {
    dotenv::dotenv().expect("No .env file found!");
    let api_key = env::var("API_KEY").expect("API Key missing!");


    let app = ec3api::Ec3api::new(&api_key);
    // dbg!(app);

    let materials = app.get_epds().unwrap();
}
