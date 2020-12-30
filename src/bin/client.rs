use paho_mqtt as mqtt;

use mqtt::{AsyncClient, DeliveryToken};

pub struct Client {
    instance: AsyncClient,
}

impl Client {
    pub fn new() -> Client {
        println!("Connecting...");
        let instance = AsyncClient::new("tcp://localhost:1883").unwrap();
        instance.connect(None).wait();
        Client { instance }
    }

    pub fn publish(&self) -> DeliveryToken {
        println!("Publishing...");
        let msg = mqtt::Message::new("test", "Hello Rust MQTT world!", mqtt::QOS_1);
        self.instance.publish(msg)
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        println!("Disconnecting...");
        self.instance.disconnect(None).wait();
    }
}
