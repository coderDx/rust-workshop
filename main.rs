use bsc::{temp_sensor::BoardTempSensor, wifi::wifi};
// Generic rust libraries for embedded systems
use embedded_svc::mqtt::client::{Publish, QoS};
use esp32_c3_dkc02_bsc as bsc;
// Wrappers for various ESP-IDF specific  services (WiFi, Network, Httpd, Logging, etc.)
use esp_idf_svc::mqtt::client::{EspMqttClient, MqttClientConfiguration};
use std::{thread::sleep, time::Duration};

// imported message topics
use mqtt_messages::{hello_topic, temperature_data_topic};

#[toml_cfg::toml_config]
pub struct Config {
    #[default("localhost")]
    mqtt_host: &'static str,
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}

fn main() -> anyhow::Result<()> {
    let unique_root_topic = "dx1609";
    // A hack to make sure that a few patches to the ESP-IDF
    // which are implemented in Rust are linked to the final executable
    esp_idf_sys::link_patches();

    let app_config = CONFIG;

    let _wifi = wifi(app_config.wifi_ssid, app_config.wifi_psk)?;

    let url = format!("mqtt://{}", app_config.mqtt_host);
    let cfg = MqttClientConfiguration::default();
    // `move`: Converts any variables captured by reference or
    // mutable reference to variables captured by value.
    let mut client = EspMqttClient::new(url, &cfg, move |_| {})?;

    let payload: &[u8] = "Connected".as_bytes();

    // fn publish(topic: &str, qos: client::QoS, retain: bool, payload: &[u8])
    // -> Result<client::MessageId, Self::Error>
    client.publish(
        &hello_topic(unique_root_topic),
        QoS::AtLeastOnce,
        true,
        payload,
    )?;

    let mut temp_sensor = BoardTempSensor::new_taking_peripherals();
    loop {
        sleep(Duration::from_secs(1));
        let temp = temp_sensor.read_owning_peripherals();

        println!("{}", temp);
        client.publish(
            &temperature_data_topic(unique_root_topic),
            QoS::AtLeastOnce,
            false,
            &temp.to_be_bytes() as &[u8],
        )?;
    }
}