use std::error::Error;
use std::fs;
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

    let mut token = load_token().unwrap_or_else(|| {
        login(&client, &settings)
    });

    let product_json = retrieve_products_json(&client, &token).unwrap_or_else(|_| {
        // If retrieving product JSON fails, the login token may have expired so try logging in again
        token = login(&client, &settings);
        retrieve_products_json(&client, &token).unwrap()
    });

    let heating_object = find_heating_object(&product_json);

    if args.len() == 1 {
        output_status(&heating_object);
    } else {
        let target_temp = args[1].parse::<f64>().unwrap();
        set_target_temp(&client, &heating_object, &token, target_temp);
    }
}

fn load_settings() -> TomlValue {
    let mut path = dirs::home_dir().unwrap();
    path.push(".hive-heat");
    path.push("conf.toml");
    let path = path.as_path();

    let mut file = File::open(path)
        .expect("Failed to open ~/.hive-heat/conf.toml");
    let mut settings_string = String::new();
    file.read_to_string(&mut settings_string).unwrap();
    return settings_string.parse::<TomlValue>().unwrap();
}

fn load_token() -> Option<String> {
    let mut path = dirs::home_dir().unwrap();
    path.push(".hive-heat");
    path.push("token");
    let path = path.as_path();

    match File::open(path) {
        Ok(mut file)  => {
	    let mut token = String::new();
            file.read_to_string(&mut token).unwrap();
            Some(token)
        }
        Err(_) => None,
    }
}

fn login(client: &Client, settings: &TomlValue) -> String {
    let token = send_login_request(&client, settings["username"].as_str().unwrap(), settings["password"].as_str().unwrap());
    // Save the login token so it can be used again, otherwise login rate limits will be hit
    save_token(&token);
    token
}

fn send_login_request(client: &Client, username: &str, password: &str) -> String {
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

fn save_token(token: &str) {
    let mut path = dirs::home_dir().unwrap();
    path.push(".hive-heat");
    path.push("token");
    let path = path.as_path();

    fs::write(path, token)
        .expect("Failed to write to ~/.hive-heat/token");
}


fn retrieve_products_json(client: &Client, token: &str) -> Result<Value, Box<dyn Error>> {
    let product_json = client
        .get(&format!("{}products?after=", HIVE_API_ENDPOINT))
        .header(AUTHORIZATION, HeaderValue::from_str(token)?)
        .send()?
        .json()
        .expect("Failed to parse products response");

    Ok(product_json)
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
    let working = heating_object["props"].as_object().unwrap()["working"].as_bool().unwrap();
    let working_indicator = if working {"🔥"} else {""};
    println!("Temperature {:>6.1}°C", temp);
    println!("Target      {:>6.1}°C {}", target_temp, working_indicator);
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

