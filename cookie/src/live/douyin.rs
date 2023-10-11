use dotenv_rs::get_vars_with_prefix;
use std::{process::exit, time::Duration,thread};
use reqwest::{Client,header::COOKIE};


pub fn local_cookie(path:String,key: &String) -> Result<String, std::io::Error> {
    let maps = get_vars_with_prefix(path, key);
    match maps {
        Ok(map) => match map.get(key) {
            Some(value) => Ok(value.clone().unwrap()),
            None => {
                println!("不存在 {}", key);
                exit(0)
            }
        },
        Err(map) => {
            println!("{}",map.to_string());
            exit(0)
        }
    }
}
pub async fn auto_cookie(url:String) -> Result<String, Box<dyn std::error::Error>>{
    let client = Client::new();
    let mut cookie = String::from("");
    for _i in 0..2 {
        let resp = client
            .get(url.clone())
            .header(COOKIE, cookie.clone())
            .send()
            .await?;
        match resp.headers().get("set-cookie") {
            Some(data) => match data.to_str() {
                Ok(data) => cookie = format!("{}{}; ",cookie,data.split_once(";").unwrap().0.to_string()),
                Err(_) => todo!(),
            },
            None => todo!(),
        }
        thread::sleep(Duration::from_millis(500))
    }
    Ok(cookie)
}