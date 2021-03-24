use redis::{self, Commands};
use serde::Serialize;

pub struct Publisher {
    connection: redis::Connection,
}

impl Publisher {
    pub fn new(redis_url: &str) -> Self {
        let client = redis::Client::open(redis_url).unwrap();
        let connection = client.get_connection().unwrap();

        Self { connection }
    }

    pub fn publish<T>(&mut self, topic: &str, msg: &T)
    where
        T: Sized + Serialize,
    {
        let msg_str = serde_json::to_string(msg).unwrap();
        let _ = self.connection.publish::<&str, String, i64>(topic, msg_str);
    }
}
