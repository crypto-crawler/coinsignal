use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
};

pub struct PriceUpdater {
    redis_url: String,
    prices: Arc<Mutex<HashMap<String, f64>>>,
}

#[derive(Serialize, Deserialize)]
struct MarkPrice {
    currency: String,
    price: f64,
}

impl PriceUpdater {
    pub fn new(redis_url: &str) -> Self {
        let updater = PriceUpdater {
            redis_url: redis_url.to_string(),
            prices: Arc::new(Mutex::new(HashMap::new())),
        };

        updater.run(); // create a thread

        updater
    }

    pub fn get_price(&self, currency: &str) -> Option<f64> {
        if let Some(price) = self.prices.lock().unwrap().get(currency) {
            Some(*price)
        } else {
            None
        }
    }

    fn run(&self) {
        let prices_clone = self.prices.clone();
        let redis_url = self.redis_url.to_string();
        thread::spawn(move || {
            let client = redis::Client::open(redis_url).unwrap();
            let mut connection = client.get_connection().unwrap();
            let mut pubsub = connection.as_pubsub();
            pubsub.subscribe("coinsignal:mark_price").unwrap();

            loop {
                let msg = pubsub.get_message().unwrap();
                let payload: String = msg.get_payload().unwrap();
                if let Ok(mark_price) = serde_json::from_str::<MarkPrice>(&payload) {
                    let mut guard = prices_clone.lock().unwrap();
                    guard.insert(mark_price.currency.clone(), mark_price.price);
                }
            }
        });
    }
}
