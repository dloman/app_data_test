use actix_web::{web, App, HttpServer};
use braintree::{Braintree, Environment};
use std::io::BufReader;
use std::fs::File;
use std::collections::BTreeMap;
use std::sync::Mutex;
use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
struct AppState{
    counter: Option<i32>,
}

async fn increment(data: web::Data<Mutex<BTreeMap<String, AppState>>>, braintree: web::Data<Mutex<Braintree>>) -> String {
    let braintree = &*(braintree.lock().unwrap());
    let _transaction = braintree.transaction().create(Default::default());

    let data = &mut *(data.lock().unwrap());
    match data.get_mut("v") {
        Some(ref mut app_state) => {
            match app_state.counter {
                Some(ref mut counter) => *counter += 1,
                None => (),
            }
        },
        None => (),
    }

    format!("increment Request number: {:#?}", data) // <- response with count
}

async fn read(data: web::Data<Mutex<BTreeMap<String, AppState>>>) -> String {
    let data = &*(data.lock().unwrap()); // <- get counter's MutexGuard
    format!("read Request number: {:#?}", data) // <- response with count
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let file = File::open("state.json").expect("valid state.json is required");
    let reader = BufReader::new(file);
    let counter : BTreeMap<String, AppState> = serde_json::from_reader(reader).expect("failure reading state.json");
    let counter = web::Data::new(Mutex::new(counter));

    let braintree = web::Data::new(Mutex::new(Braintree::new(
                Environment::from_str(&std::env::var("ENVIRONMENT").expect("environment variable ENVIRONMENT is not defined")).unwrap(),
                std::env::var("MERCHANT_ID").expect("environment variable MERCHANT_ID is not defined"),
                std::env::var("PUBLIC_KEY").expect("environment variable PUBLIC_KEY is not defined"),
                std::env::var("PRIVATE_KEY").expect("environment variable PRIVATE_KEY is not defined"),
                )));

    HttpServer::new(move || {
        App::new()
            .app_data(counter.clone())
            .app_data(braintree.clone())
            .route("/", web::get().to(read))
            .route("/increment", web::post().to(increment))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
