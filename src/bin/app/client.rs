use paho_mqtt as mqtt;

use crate::app::helpers::regex_from_mqtt_wildcard;
use crate::cli::CLIError;
use itertools::{zip, Itertools};
use mqtt::{AsyncClient, DeliveryToken, Message};
use paho_mqtt::{ConnectOptionsBuilder, QOS_1};
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;

pub trait MessageHandler {
    fn handle_message(&self, msg: Message);
    fn get_handled_topic(&self) -> &str;
}

pub struct Credentials {
    pub username: String,
    pub password: String,
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
    pub fn new(url: &str, credentials: &Option<Credentials>) -> Result<Client, CLIError> {
        let instance = AsyncClient::new(url).unwrap();
        let connect_opts = credentials
            .as_ref()
            .map(|Credentials { username, password }| {
                ConnectOptionsBuilder::new()
                    .user_name(username)
                    .password(password)
                    .finalize()
            });
        instance.connect(connect_opts).wait().or_else(|_| {
            Err(CLIError::CannotConnectToBroker {
                url: url.to_string(),
            })
        })?;
        Ok(Client { instance })
    }

    pub fn publish<S, Obj>(&self, topic: S, obj: &Obj) -> DeliveryToken
    where
        S: Into<String>,
        Obj: ?Sized + Serialize,
    {
        let msg = mqtt::Message::new(topic, serde_json::to_string(obj).unwrap(), mqtt::QOS_1);
        self.instance.publish(msg)
    }

    pub fn update_subscriptions(
        &mut self,
        subs: Vec<Box<dyn MessageHandler>>,
    ) -> Result<(), CLIError> {
        let all_topics: Vec<String> = subs
            .iter()
            .map(|s| s.get_handled_topic())
            .map_into()
            .collect();
        let regexes: Vec<_> = subs
            .iter()
            .map(|s| s.get_handled_topic())
            .map(regex_from_mqtt_wildcard)
            .collect();
        let all_qos = [QOS_1].repeat(all_topics.len());

        self.instance
            .unsubscribe("#")
            .wait()
            .or(Err(CLIError::SubscriptionCouldNotBeUpdated))?;
        self.instance.set_message_callback(move |_, msg_opt| {
            if let Some(msg) = msg_opt {
                if let Some((sub, _)) = zip(&subs, &regexes).find(|(_, r)| r.is_match(msg.topic()))
                {
                    sub.handle_message(msg);
                }
            }
        });
        self.instance
            .subscribe_many(&all_topics, &all_qos)
            .wait()
            .map(|_| ())
            .or(Err(CLIError::SubscriptionCouldNotBeUpdated))
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.instance.disconnect(None).wait().unwrap();
    }
}
