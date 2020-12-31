use paho_mqtt as mqtt;

use itertools::Itertools;
use mqtt::{AsyncClient, DeliveryToken, Message};
use paho_mqtt::QOS_1;
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;

pub trait MessageHandler {
    fn handle_message(&self, msg: Message);
    fn get_handled_topic(&self) -> &str;
}

pub struct Subscription<T: DeserializeOwned, F: Fn(T)> {
    pub topic: String,
    pub callback: F,
    _marker: PhantomData<T>,
}

impl<T, F> Subscription<T, F>
where
    T: DeserializeOwned,
    F: Fn(T),
{
    pub fn new<S: Into<String>>(topic: S, callback: F) -> Self {
        Self {
            topic: topic.into(),
            callback,
            _marker: PhantomData,
        }
    }

    pub fn boxed_new<S: Into<String>>(topic: S, callback: F) -> Box<Self> {
        Box::new(Self::new(topic, callback))
    }
}

impl<T, F> MessageHandler for Subscription<T, F>
where
    T: DeserializeOwned,
    F: Fn(T),
{
    fn handle_message(&self, msg: Message) {
        let obj = serde_json::from_slice(msg.payload()).unwrap();
        (self.callback)(obj);
    }

    fn get_handled_topic(&self) -> &str {
        &self.topic
    }
}

pub struct Client {
    instance: AsyncClient,
}

impl Client {
    pub fn new(host: &str) -> Client {
        let instance = AsyncClient::new(host).unwrap();
        instance.connect(None).wait().unwrap();
        Client { instance }
    }

    pub fn publish<S, Obj>(&self, topic: S, obj: &Obj) -> DeliveryToken
    where
        S: Into<String>,
        Obj: ?Sized + Serialize,
    {
        let msg = mqtt::Message::new(topic, serde_json::to_string(obj).unwrap(), mqtt::QOS_1);
        self.instance.publish(msg)
    }

    pub fn update_subscriptions(&mut self, subs: Vec<Box<dyn MessageHandler>>) {
        let all_topics: Vec<String> = subs
            .iter()
            .map(|s| s.get_handled_topic())
            .map_into()
            .collect();
        let all_qos = [QOS_1].repeat(all_topics.len());

        self.instance.unsubscribe("#").wait().unwrap();
        self.instance.set_message_callback(move |_, msg_opt| {
            if let Some(msg) = msg_opt {
                if let Some(sub) = subs.iter().find(|s| s.get_handled_topic() == msg.topic()) {
                    sub.handle_message(msg);
                }
            }
        });
        self.instance.subscribe_many(&all_topics, &all_qos);
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.instance.disconnect(None).wait().unwrap();
    }
}
