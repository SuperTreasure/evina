use dotenv_rs::get_vars_with_prefix;
use std::{process::exit, time::Duration,thread};
use reqwest::{ClientBuilder,header::COOKIE};


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
pub async fn auto_cookie(url: String) -> Result<String, Box<dyn std::error::Error>> {
    let client = ClientBuilder::new()
        .timeout(std::time::Duration::from_secs(10)) // 设置 10 秒超时
        .build()?;
    let mut cookie = String::from("");
    for _i in 0..2 {
        let resp = client.get(url.clone()).header(COOKIE, cookie.clone()).send().await?;
        match resp.headers().get("set-cookie") {
            Some(data) => {
                let data_str = match data.to_str() {
                    Ok(str) => str,
                    Err(_) => {
                        eprintln!("set-cookie header is not valid UTF-8");
                        continue;
                    }
                };
                if let Some((first, _)) = data_str.split_once(";") {
                    cookie = format!("{}{}; ", cookie, first);
                } else {
                    eprintln!("set-cookie header does not contain ';'");
                }
            }
            None => {
                eprintln!("No set-cookie header found");
            }
        }
        thread::sleep(Duration::from_millis(500));
    }
    Ok(cookie)
}
