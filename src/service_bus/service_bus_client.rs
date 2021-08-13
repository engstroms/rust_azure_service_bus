use crate::service_bus::client_config::ClientConfig;
use crate::service_bus::certificate_validator::CertificateValidator;
use std::error::Error;
use std::sync::Arc;
use ntex::connect::rustls::{RustlsConnector, TlsStream};
use ntex_amqp::client::SaslAuth;
use ntex::rt::net::TcpStream;
use log::{info, error};
use ntex_amqp::codec::protocol::{Transfer, TransferBody};
use ntex_amqp::error::AmqpProtocolError;
use ntex_amqp_codec::{Message};
use ntex_amqp::codec::Decode;
use futures::{StreamExt};
use tokio::sync::mpsc::{Receiver, channel};

pub struct ServiceBusClient {
    config: ClientConfig,
}

impl ServiceBusClient {
    pub fn new(config: ClientConfig) -> Self {
        ServiceBusClient {
            config,
        }
    }

    pub async fn start(&self) -> Result<Receiver<Message>, Box<dyn Error>> {
        info!("starting receiver...");

        let driver = self.create_driver().await?;
        let sink = driver.sink();
        ntex::rt::spawn(driver.start_default());

        let mut session = match sink.open_session().await {
            Ok(s) => s,
            Err(e) => return Err(format!("{:?}", e).into()),
        };

        let mut receiver = match session
            .build_receiver_link("my_receiver", format!("{}/Subscriptions/{}", self.config.topic, self.config.subscription_id))
            .attach()
            .await {
            Ok(r) => r,
            Err(e) => return Err(format!("{:?}", e).into())
        };

        receiver.set_link_credit(300);

        let (sender, channel_receiver) = channel(1000);

        ntex::rt::spawn(async move {
            info!("starting loop...");
            loop {
                match parse_next(receiver.next().await) {
                    Ok(message) => {
                        sender.send(message).await
                            .expect("could not put message on channel");
                    }
                    Err(e) => {
                        error!("Error:{}", e);
                    }
                }
            }
        });

        Ok(channel_receiver)
    }

    async fn create_driver(&self) -> Result<ntex_amqp::client::Client<TlsStream<TcpStream>>, Box<dyn Error>> {
        let mut rustls_config = rustls::ClientConfig::new();
        rustls_config.dangerous().set_certificate_verifier(Arc::new(CertificateValidator {}));

        let driver = ntex_amqp::client::Connector::new()
            .connector(RustlsConnector::new(Arc::new(rustls_config)))
            .hostname(&self.config.hostname)
            .connect_sasl(format!("{}:{}", self.config.hostname, self.config.port), SaslAuth {
                authz_id: "".into(),
                authn_id: self.config.user.clone().into(),
                password: self.config.password.clone().into(),
            }).await?;

        Ok(driver)
    }
}

fn parse_next(message: Option<Result<Transfer, AmqpProtocolError>>) -> Result<Message, Box<dyn Error>> {
    match message {
        None => Err("no data".into()),

        Some(result) => {
            match result {
                Ok(transfer) => {
                    parse_transfer_body(transfer.body.as_ref())
                }
                Err(e) => {
                    Err(format!("{:?}", e).into())
                }
            }
        }
    }
}

fn parse_transfer_body(body: Option<&TransferBody>) -> Result<Message, Box<dyn Error>> {
    match body {
        None => return Err("no data".into()),
        Some(transfer_body) => {
            match transfer_body {
                TransferBody::Data(bytes) => {
                    match Message::decode(&bytes) {
                        Ok(m) => Ok(m.1),
                        Err(e) => Err(format!("{:?}", e).into())
                    }
                },
                _ => Err("no interest to me".into())
            }
        }
    }
}