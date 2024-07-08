#[macro_use] extern crate rocket;

use rocket::get;
use rocket::serde::json::Json;
use serde::Serialize;
use rand::Rng;
use tokio::task;

#[derive(Serialize, Debug)]
pub struct PriceResult {
    id: i32,
    price: f64,
}

impl PriceResult {
    fn new(id: i32, price: f64) -> Self {
        Self { id, price }
    }
}

fn get_price_from_db(id: i32) -> PriceResult {
    println!("getPriceFromDb thread: {:?}", std::thread::current().id());
    let mut rng = rand::thread_rng();
    let price: f64 = rng.gen_range(1.0..51.0);
    let price: f64 = (price * 100.0).round() / 100.0;
    PriceResult::new(id, price)
}

#[get("/price?<id>")]
async fn get_price(id: i32) -> Result<Json<PriceResult>, rocket::response::status::Custom<String>> {
    println!("Current thread: {:?}", std::thread::current().id());
    let price_result = task::spawn_blocking(move || get_price_from_db(id))
        .await
        .map_err(|e| rocket::response::status::Custom(rocket::http::Status::InternalServerError, e.to_string()))?;
    Ok(Json(price_result))
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![get_price])
}
