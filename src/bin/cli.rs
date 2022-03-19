use clap::Parser;

/// MQTT Client GUI for Machine Vision Systems
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Connect to the MQTT broker at the specified host.
    #[clap(short, long, default_value = "localhost")]
    pub host: String,

    /// Connect to the host on the specified port.
    #[clap(short, long, default_value = "1883")]
    pub port: u16,

    /// MQTT topic prefix.
    #[clap(short = 't', long, default_value = "merlic")]
    pub prefix: String,
}

#[derive(Debug)]
pub enum CLIError {
    CannotConnectToBroker { url: String },
    SubscriptionCouldNotBeUpdated,
}
