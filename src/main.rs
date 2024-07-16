#[macro_use] extern crate rocket;

use rocket::{Rocket, Build};
use rocket::get;
use rocket::serde::json::Json;
use serde::Serialize;
use rand::Rng;
use tokio::task;

use opentelemetry::{
    global, runtime,
    sdk::{propagation::TraceContextPropagator, trace, Resource},
    trace::TraceError,
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use tracing::{event, Level};
use tracing_subscriber::prelude::*;

fn init_tracer() -> Result<trace::Tracer, TraceError> {
    // Initialise OTLP Pipeline
    opentelemetry_otlp::new_pipeline()
        .tracing() // create OTLP tracing pipeline
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic() // create GRPC layer 
                .with_endpoint("http://otel-collector-opentelemetry-collector.collectors:4317"), // GRPC OTLP Jaeger Endpoint
        )
        // Trace provider configuration 
        .with_trace_config(
            trace::config().with_resource(Resource::new(vec![KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                "pricing-rust",
            )])),
        )
        .install_batch(runtime::Tokio) // configure a span exporter
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

#[tracing::instrument]
fn get_price_from_db(id: i32) -> PriceResult {
    println!("getPriceFromDb thread: {:?}", std::thread::current().id());
    let mut rng = rand::thread_rng();
    let price: f64 = rng.gen_range(1.0..51.0);
    let price: f64 = (price * 100.0).round() / 100.0;

    event!(
        Level::INFO,
        "otel.status_message" = "get_price_from_db",
    );

    PriceResult::new(id, price)
}

#[tracing::instrument]
#[get("/price?<id>")]
async fn get_price(id: i32) -> Result<Json<PriceResult>, rocket::response::status::Custom<String>> {
    println!("Current thread: {:?}", std::thread::current().id());
    let price_result = task::spawn_blocking(move || get_price_from_db(id))
        .await
        .map_err(|e| rocket::response::status::Custom(rocket::http::Status::InternalServerError, e.to_string()))?;
    // add metadata to the span
    event!(
        Level::INFO,
        "otel.status_message" = "get_price",
        "otel.status_code" = 200
    );

    Ok(Json(price_result))
}

// #[launch]
// replace launch macro with main so we can add Tracing setup
#[rocket::main]
async fn main() {
    // set the global propagator
    global::set_text_map_propagator(TraceContextPropagator::new());

    // initialize the tracer
    let tracer = init_tracer().unwrap();

    // create an opentelemetry layer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // create a subscriber 
    let subscriber = tracing_subscriber::Registry::default().with(telemetry);

    // set the global subscriber for the app
    tracing::subscriber::set_global_default(subscriber).unwrap();

    // Recall that an uninspected `Error` will cause a pretty-printed panic,
    // so rest assured errors do not go undetected when using `#[launch]`.
    let _ = rocket().launch().await;

    global::shutdown_tracer_provider();
}

fn rocket() -> Rocket<Build> {
    rocket::build().mount("/", routes![get_price])
}
