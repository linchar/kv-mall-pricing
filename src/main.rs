#[macro_use] extern crate rocket;

use rocket::get;
use rocket::serde::json::Json;
use serde::Serialize;
use rand::Rng;
use tokio::task;
use opentelemetry::global::ObjectSafeSpan;
use opentelemetry::trace::{SpanKind, Status};
use opentelemetry::{global, trace::Tracer};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_stdout::SpanExporter;

fn init_tracer() {
    global::set_text_map_propagator(TraceContextPropagator::new());
    let provider = TracerProvider::builder()
        .with_simple_exporter(SpanExporter::default())
        .build();
    global::set_tracer_provider(provider);
}

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
    let tracer = global::tracer("pricing_service");     // spanScope
    let mut span = tracer
        .span_builder(format!("get price for id: {}", id))
        .with_kind(SpanKind::Server)
        .start(&tracer);

    println!("Current thread: {:?}", std::thread::current().id());
    let price_result = task::spawn_blocking(move || get_price_from_db(id))
        .await
        .map_err(|e| rocket::response::status::Custom(rocket::http::Status::InternalServerError, e.to_string()))?;
    
    // Set span status to Ok after successful price retrieval
    span.set_status(Status::Ok);
    Ok(Json(price_result))
}

#[launch]
fn rocket() -> _ {
    init_tracer();
    rocket::build().mount("/", routes![get_price])
}
