use log::*;
use std::time::Duration;

pub fn wait_redis(redis_url: &str) {
    loop {
        let client = redis::Client::open(redis_url).unwrap();
        match client.get_connection() {
          Ok(mut conn) => {
            let resp = redis::cmd("PING").query::<String>(&mut conn);
            if let Ok(pong) = resp {
                if pong == "PONG" {
                    break;
                }
            }
          }
          Err(err) => {
            warn!("{}", err);
          }
        }
        info!("Redis is not ready, sleeping for 1 second");
        std::thread::sleep(Duration::from_secs(1));
    }
}
