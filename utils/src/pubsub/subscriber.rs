pub struct Subscriber {
    redis_url: String,
    channel: String,
    on_msg: Box<dyn FnMut(String)>,
}

impl Subscriber {
    pub fn new(redis_url: &str, channel: &str, on_msg: Box<dyn FnMut(String)>) -> Self {
        Self {
            redis_url: redis_url.to_string(),
            channel: channel.to_string(),
            on_msg,
        }
    }

    pub fn run<'a>(&mut self) {
        let client = redis::Client::open(self.redis_url.as_str()).unwrap();
        let mut connection = client.get_connection().unwrap();
        let mut pubsub = connection.as_pubsub();
        pubsub.subscribe(self.channel.as_str()).unwrap();

        loop {
            let msg = pubsub.get_message().unwrap();
            let payload: String = msg.get_payload().unwrap();
            (self.on_msg)(payload);
        }
    }
}
