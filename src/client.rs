use std::fs::read;
use rumqttc::{self, Incoming, Client, Connection, MqttOptions, Publish, PubAck, QoS, ConnectionError};

#[cfg(feature= "async")]
use rumqttc::{EventLoop, Subscribe, Request};
use async_trait::async_trait;
use async_channel::Sender;

pub trait AWSEventHandler {

    fn on_connect() {
        println!("Default connection!");
    }
    fn on_publish(message: Publish) {
        println!("Default publish");
    }

    fn on_puback(message: PubAck) {
        println!("Default puback");
    }

    fn start_event_listener(&self, mut connection: Connection) {
        for notification in connection.iter() {
            match notification {
                Ok(notification_type) => match notification_type.0 {
                    Some(Incoming::Publish(message)) => {
                        Self::on_publish(message);
                    },
                    Some(Incoming::Connected) => {
                        Self::on_connect();
                    },
                    _ => (),
                    None => (),
                },
                Err(_) => (),
            }
        }
    }
}

#[cfg(feature= "async")]
#[async_trait]
pub trait AWSAsyncEventHandler {

    fn on_connect() {
        println!("Default connection!");
    }
    fn on_publish(message: Publish) {
        println!("Default publish");
    }

    fn on_puback(message: PubAck) {
        println!("Default puback");
    }

    async fn start_async_event_listener(&self, mut eventloop: EventLoop) {
        loop {
            match eventloop.poll().await {
                Ok(incoming) => {
                    println!("Incoming message!");
                    match incoming.0 {
                        Some(Incoming::Publish(message)) => {
                            Self::on_publish(message);
                        },
                        Some(Incoming::Connected) => {
                            Self::on_connect();
                        },
                        Some(Incoming::PubAck(puback)) => {
                            Self::on_puback(puback);
                        },
                        _ => (),
                    }
                },
                Err(_) => (),
            }
        }
    }

}

pub struct AWSIoTSettings {
        client_id: String,
        ca_path: String,
        client_cert_path: String,
        client_key_path: String,
        aws_iot_endpoint: String,
}

impl AWSIoTSettings {
    pub fn new(
        client_id: String,
        ca_path: String,
        client_cert_path: String,
        client_key_path: String,
        aws_iot_endpoint: String) -> AWSIoTSettings {

        AWSIoTSettings { client_id, ca_path, client_cert_path, client_key_path, aws_iot_endpoint }
    
    }
}

pub struct AWSIoTClient {
    pub client: Client,
}

impl AWSIoTClient {
    pub fn new(
        settings: AWSIoTSettings
        ) -> Result<(AWSIoTClient, Connection), ConnectionError> {

        let mut mqtt_options = MqttOptions::new(settings.client_id, settings.aws_iot_endpoint, 8883);
        mqtt_options.set_ca(read(settings.ca_path)?)
            .set_client_auth(read(settings.client_cert_path)?, read(settings.client_key_path)?)
            .set_keep_alive(10);

            let (client, connection) = Client::new(mqtt_options, 10);
            Ok((AWSIoTClient { client: client }, connection))
    }

    /// Subscribe to any topic.
    pub fn subscribe (&mut self, topic_name: String, qos: QoS) {
        self.client.subscribe(topic_name, qos).unwrap();
    }

    /// Publish to any topic.
    pub fn publish (&mut self, topic_name: String, qos: QoS, payload: &str) {
        self.client.publish(topic_name, qos, false, payload).unwrap();
    }
}

#[cfg(feature= "async")]
pub struct AWSIoTAsyncClient {
    pub sender: Sender<Request>,
}

#[cfg(feature= "async")]
impl AWSIoTAsyncClient {

    pub async fn new(
        client_id: &str,
        ca_path: &str,
        client_cert_path: &str,
        client_key_path: &str,
        aws_iot_endpoint: &str) -> Result<(AWSIoTAsyncClient, EventLoop), ConnectionError> {

        let mut mqtt_options = MqttOptions::new(client_id, aws_iot_endpoint, 8883);
        mqtt_options.set_ca(read(ca_path)?)
            .set_client_auth(read(client_cert_path)?, read(client_key_path)?)
            .set_keep_alive(10);
        let eventloop = EventLoop::new(mqtt_options, 10).await;
        let requests_tx = eventloop.handle();
        Ok((AWSIoTAsyncClient { sender: requests_tx }, eventloop))
    }

    /// Subscribe to any topic.
    pub async fn subscribe (&mut self, topic_name: String, qos: QoS) {
        let subscribe = Subscribe::new(topic_name, qos);
        self.sender.send(Request::Subscribe(subscribe)).await.unwrap();
    }

    /// Publish to any topic.
    pub async fn publish (&mut self, topic_name: String, qos: QoS, payload: &str) {
        let publish = Publish::new(topic_name, qos, payload);
        self.sender.send(Request::Publish(publish)).await.unwrap();
    }
}
