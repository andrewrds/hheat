use std::fs::File;
use std::io::prelude::*;
use reqwest::blocking::Client;
use serde_json::{Value, map::Map};
use http::header::{HeaderValue, AUTHORIZATION};
use toml::Value as TomlValue;
use dirs;

const HIVE_API_ENDPOINT: &str = "https://beekeeper.hivehome.com/1.0/";
 
fn main() {
    let settings = load_settings();

    let client = Client::new();

    let token = login(&client, settings["username"].as_str().unwrap(), settings["password"].as_str().unwrap());
    let product_json = retrieve_products_json(&client, &token);
    let heating_object = find_heating_object(&product_json);

    let temp = heating_object["props"].as_object().unwrap()["temperature"].as_f64().unwrap();
    let target_temp = heating_object["state"].as_object().unwrap()["target"].as_f64().unwrap();
    println!("Temperature {:>6.1}°C", temp);
    println!("Target      {:>6.1}°C", target_temp);
}

fn load_settings() -> TomlValue {
    let mut path = dirs::home_dir().unwrap();
    path.push(".hive-heat");
    let path = path.as_path();

    let mut file = File::open(path)
        .expect("Failed to open .hive-heat settings");
    let mut settings_string = String::new();
    file.read_to_string(&mut settings_string).unwrap();
    return settings_string.parse::<TomlValue>().unwrap();
}

fn login(client: &Client, username: &str, password: &str) -> String {
    let resp: Value = client
        .post(&format!("{}global/login", HIVE_API_ENDPOINT))
        .body(format!("{{\"username\":\"{}\",\"password\":\"{}\",\"devices\":true,\"products\":true,\"actions\":true,\"homes\":true}}", username, password))
        .send()
        .expect("Login request failed")
        .json()
        .expect("Failed to parse login response");

    let token = resp["token"].as_str().unwrap();
    return String::from(token);
}

fn retrieve_products_json(client: &Client, token: &str) -> Value {
    return client
        .get(&format!("{}products?after=", HIVE_API_ENDPOINT))
        .header(AUTHORIZATION, HeaderValue::from_str(token).unwrap())
        .send()
        .expect("Products request failed")
        .json()
        .expect("Failed to parse products response");
}

fn find_heating_object(product_json: &Value) -> &Map<String, Value> {
    for product in product_json.as_array().unwrap() {
        let product_object = product.as_object().unwrap();
        if product_object["type"].as_str().unwrap() == "heating" {
            return product_object;
        }
    }

   panic!();
}

