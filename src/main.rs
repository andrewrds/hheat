use std::fs::File;
use std::io::prelude::*;
use std::env;
use reqwest::blocking::Client;
use serde_json::{Value, map::Map};
use http::header::{HeaderValue, AUTHORIZATION};
use toml::Value as TomlValue;
use dirs;

const HIVE_API_ENDPOINT: &str = "https://beekeeper.hivehome.com/1.0/";
 
fn main() {
    let args: Vec<String> = env::args().collect();
    let settings = load_settings();

    let client = Client::new();

    let token = login(&client, settings["username"].as_str().unwrap(), settings["password"].as_str().unwrap());
    let product_json = retrieve_products_json(&client, &token);
    let heating_object = find_heating_object(&product_json);

    if args.len() == 1 {
        output_status(&heating_object);
    } else {
        let target_temp = args[1].parse::<f64>().unwrap();
        set_target_temp(&client, &heating_object, &token, target_temp);
    }

    logout(&client, &token);
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

    let token = resp["token"].as_str().expect(&format!("Failed to get login token: {}", resp));
    return String::from(token);
}

fn logout(client: &Client, token: &str) {
    client
        .delete(&format!("{}auth/logout", HIVE_API_ENDPOINT))
        .header(AUTHORIZATION, HeaderValue::from_str(token).unwrap())
        .body("{}")
        .send()
        .expect("Logout failed");
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

fn output_status(heating_object: &Map<String, Value>) {
    let temp = heating_object["props"].as_object().unwrap()["temperature"].as_f64().unwrap();
    let target_temp = heating_object["state"].as_object().unwrap()["target"].as_f64().unwrap();
    println!("Temperature {:>6.1}°C", temp);
    println!("Target      {:>6.1}°C", target_temp);
}

fn set_target_temp(client: &Client, heating_object: &Map<String, Value>, token: &str, target_temp: f64) {
    let device_id = heating_object["id"].as_str().unwrap();
    client
        .post(&format!("{}nodes/heating/{}", HIVE_API_ENDPOINT, device_id))
        .header(AUTHORIZATION, HeaderValue::from_str(token).unwrap())
        .body(format!("{{\"target\":{}}}", target_temp))
        .send()
        .expect("Set temperature request failed");
}

