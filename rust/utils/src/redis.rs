use log::*;
use std::time::Duration;

pub fn wait_redis(redis_url: &str) {
    let client = redis::Client::open(redis_url).unwrap();
    let mut conn = client.get_connection().unwrap();
    loop {
        let resp = redis::cmd("PING").query::<String>(&mut conn);
        if let Ok(pong) = resp {
            if pong == "PONG" {
                break;
            }
        }
        info!("Redis is not ready, sleeping for 1 second");
        std::thread::sleep(Duration::from_secs(1));
    }
}
