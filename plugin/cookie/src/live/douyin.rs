use dotenv_rs::get_vars_with_prefix;
use reqwest::{header::COOKIE, Client};
use std::{
    thread,
    time::{self, Duration},
};

pub fn local_cookie(path: String, key: &String) -> Result<String, Box<dyn std::error::Error>> {
    let maps = get_vars_with_prefix(path, key);
    match maps {
        Ok(map) => match map.get(key) {
            Some(value) => Ok(value.clone().unwrap()),
            None => return Err(format!("不存在 {}", key).into()),
        },
        Err(e) => return Err(e.into()),
    }
}
pub async fn auto_cookie(url: String) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::builder()
        .connect_timeout(time::Duration::from_secs(20))
        .build()?;
    let mut cookie = String::from("");
    for _i in 0..2 {
        match client
            .get(url.clone())
            .header(COOKIE, cookie.clone())
            .send()
            .await
        {
            Ok(resp) => match resp.headers().get("set-cookie") {
                Some(data) => match data.to_str() {
                    Ok(data) => {
                        cookie = format!(
                            "{}{}; ",
                            cookie,
                            data.split_once(";").unwrap().0.to_string()
                        )
                    }
                    Err(e) => return Err(e.into()),
                },
                None => return Err("cookie 获取失败".into()),
            },
            Err(e) => return Err(e.into()),
        }

        thread::sleep(Duration::from_millis(1000))
    }
    Ok(cookie)
}
