use crate::service_bus::client_config::ClientConfig;
use crate::service_bus::service_bus_client::ServiceBusClient;
use log::{info};

mod service_bus;

#[ntex::main]
async fn main() {
    std::env::set_var("RUST_LOG", "info,ntex=trace,ntex_amqp=trace,basic=trace");
    pretty_env_logger::init();

    let client_options = ClientConfig::new(
        "namespace.servicebus.windows.net",
        5671,
        "RootManageSharedAccessKey",
        "mykey",
        "topic",
        "sub-id",
    );

    let client = ServiceBusClient::new(client_options);
    let mut receiver = client.start().await.expect("could not start client");

    info!("starting main while...");
    while let Some(message) = receiver.recv().await {
        info!("message in main: {:?}", message);
    }

    info!("END!!!");
}
