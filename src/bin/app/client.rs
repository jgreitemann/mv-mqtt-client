use paho_mqtt as mqtt;

use mqtt::{AsyncClient, DeliveryToken};
use mvjson::{Action, ModeType};

pub struct Client {
    instance: AsyncClient,
}

impl Client {
    pub fn new(host: &str) -> Client {
        let instance = AsyncClient::new(host).unwrap();
        instance.connect(None).wait();
        Client { instance }
    }

    pub fn publish<S: Into<String>>(&self, topic: S, action: &Action) -> DeliveryToken {
        let msg = mqtt::Message::new(topic, serde_json::to_string(&action).unwrap(), mqtt::QOS_1);
        self.instance.publish(msg)
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.instance.disconnect(None).wait();
    }
}
