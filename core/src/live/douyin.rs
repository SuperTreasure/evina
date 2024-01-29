use super::{
    super::{get_user_agent, is_alphabetic, re_match, Cli},
    Information,
};
use clap::Parser;

use cookie::{auto_cookie, local_cookie};
use logger_rust::log_warn;
use reqwest::{
    header::{ACCEPT, ACCEPT_LANGUAGE, COOKIE, USER_AGENT},
    Client,
};
use serde_json::Value;
use std::{collections::HashMap, error::Error, process::exit, time};

const _ACCEPT: &str = "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7";
const _ACCEPT_LANGUAGE: &str = "zh-CN,zh;q=0.9,en;q=0.8,en-GB;q=0.7,en-US;q=0.6";

async fn get_real_id(rid: String) -> Result<HashMap<&'static str, String>, Box<dyn Error>> {
    let mut map = HashMap::new();
    let cli = Cli::parse();
    let client = Client::builder().connect_timeout(time::Duration::from_secs(20)).build().unwrap();
    // 直播间地址以 / 结尾 如: https://v.douyin.com/ie6wQq5g/
    // 去除结尾 / 返回去除后的地址
    let mut id = if rid.clone().ends_with("/") { rid.clone().rsplit_once("/").unwrap().0.to_string() } else { rid.clone() };

    // 直播间以链接形式
    if id.clone().contains("http") {
        // 切割直播间地址，获取直播间 id
        id = id.rsplit_once("/").unwrap().1.to_string();

        // 判断 id 是否含有字母，含字母为手机端临时 id
        if is_alphabetic(id.clone()) {
            match phone(cli, format!("https://v.douyin.com/{}", id), client.clone()).await {
                Ok(id) => {
                    map.insert("id", id);
                    Ok(map)
                }
                Err(e) => return Err(e.into()),
            }
        } else {
            map.insert("id", id);
            Ok(map)
        }
    } else {
        if is_alphabetic(id.clone()) {
            match phone(cli, format!("https://v.douyin.com/{}", id), client.clone()).await {
                Ok(id) => {
                    map.insert("id", id);
                    Ok(map)
                }
                Err(e) => return Err(e.into()),
            }
        } else {
            map.insert("id", id);
            Ok(map)
        }
    }
}

async fn phone(cli: Cli, id: String, client: Client) -> Result<String, Box<dyn Error>> {
    match client.get(format!("https://v.douyin.com/{}", id)).header(USER_AGENT, get_user_agent()).send().await {
        Ok(response) => match response.url().query() {
            Some(_) => {
                let url = format!("https://www.douyin.com{}", response.url().path().rsplit_once("share").unwrap().1);
                match client
                    .get(url)
                    .header(USER_AGENT, get_user_agent())
                    .header(COOKIE, local_cookie(cli.config_file, String::from("cookie_douyin")).unwrap())
                    .header(ACCEPT, _ACCEPT)
                    .header(ACCEPT_LANGUAGE, _ACCEPT_LANGUAGE)
                    .send()
                    .await
                {
                    Ok(response) => match response.text().await {
                        Ok(data) => match re_match(r#"https://live.douyin.com/(\d+)"#, &data.clone()) {
                            Some(id) => Ok(id),
                            None => return Err(format!("抖音 | {}: 短链接失效，或者主播未开播", id).into()),
                        },
                        Err(e) => return Err(e.into()),
                    },
                    Err(e) => return Err(e.into()),
                }
            }
            None => return Err(format!("抖音 | {}: 短链接转换失败", id).into()),
        },
        Err(e) => return Err(e.into()),
    }
}

pub async fn get_rtmp_url(id: Option<String>) -> Result<Information, Box<dyn Error>> {
    let client = Client::builder().connect_timeout(time::Duration::from_secs(20)).build().unwrap();
    match id {
        Some(id) => match get_real_id(id).await {
            Ok(map) => {
                let url = format!("https://live.douyin.com/webcast/room/web/enter/?aid=6383&app_name=douyin_web&live_id=1&device_platform=web&language=zh-CN&enter_from=web_live&cookie_enabled=true&screen_width=1366&screen_height=768&browser_language=zh-CN&browser_platform=Win32&browser_name=Edge&browser_version=116.0.0.0&web_rid={}&room_id_str=7248777764587817767&enter_source=&Room-Enter-User-Login-Ab=0&is_need_double_stream=false",map["id"]);
                match auto_cookie(format!("https://live.douyin.com/{}", map["id"])).await {
                    Ok(cookie) => {
                        match client.get(url).header(USER_AGENT, get_user_agent()).header(COOKIE, cookie).send().await {
                            Ok(resp) => match resp.json::<Value>().await {
                                Ok(result) => match result["data"]["data"][0]["stream_url"]["live_core_sdk_data"]["pull_data"]["stream_data"].as_str() {
                                    Some(stream_data) => {
                                        let stream_data_json: Value = serde_json::from_str(stream_data)?;
                                        let rtmp = match &stream_data_json["data"]["origin"] {
                                            Value::Object(origin) => origin["main"]["flv"].to_string(),
                                            Value::Bool(_) => todo!(),
                                            Value::Number(_) => todo!(),
                                            Value::String(_) => todo!(),
                                            Value::Array(_) => todo!(),
                                            Value::Null => match &stream_data_json["data"]["ao"] {
                                                Value::Object(origin) => origin["main"]["flv"].to_string(),
                                                Value::Bool(_) => todo!(),
                                                Value::Number(_) => todo!(),
                                                Value::String(_) => todo!(),
                                                Value::Array(_) => todo!(),
                                                Value::Null => match &stream_data_json["data"]["uhd"] {
                                                    Value::Object(origin) => origin["main"]["flv"].to_string(),
                                                    Value::Bool(_) => todo!(),
                                                    Value::Number(_) => todo!(),
                                                    Value::String(_) => todo!(),
                                                    Value::Array(_) => todo!(),
                                                    Value::Null => match &stream_data_json["data"]["hd"] {
                                                        Value::Object(origin) => origin["main"]["flv"].to_string(),
                                                        Value::Bool(_) => todo!(),
                                                        Value::Number(_) => todo!(),
                                                        Value::String(_) => todo!(),
                                                        Value::Array(_) => todo!(),
                                                        Value::Null => todo!(),
                                                    },
                                                },
                                            },
                                        };
                                        Ok(Information {
                                            platform: "抖音".to_owned(),
                                            rid: map["id"].clone(),
                                            name: result["data"]["user"]["nickname"].to_string().replace(r#"""#, ""),
                                            rtmp: rtmp.replace(r#"""#, ""),
                                        })
                                    }
                                    None => Ok(Information {
                                        platform: "抖音".to_owned(),
                                        rid: map["id"].clone(),
                                        name: result["data"]["user"]["nickname"].to_string().replace(r#"""#, ""),
                                        rtmp: "直播间未开播".to_owned(),
                                    }),
                                },
                                // Ok(result) => match result["data"]["data"][0]["stream_url"]["flv_pull_url"].as_object()
                                // {
                                //     Some(rtmp) => Ok(Information {
                                //         platform: "抖音".to_owned(),
                                //         rid: map["id"].clone(),
                                //         name: result["data"]["user"]["nickname"].to_string().replace(r#"""#, ""),
                                //         rtmp: rtmp.values().rev().last().unwrap().to_string().replace(r#"""#, ""),
                                //     }),
                                //     None => Ok(Information {
                                //         platform: "抖音".to_owned(),
                                //         rid: map["id"].clone(),
                                //         name: result["data"]["user"]["nickname"].to_string().replace(r#"""#, ""),
                                //         rtmp: "直播间未开播".to_owned(),
                                //     }),
                                // },
                                Err(e) => return Err(e.into()),
                            },
                            Err(e) => return Err(e.into()),
                        }
                    }
                    Err(e) => return Err(e.into()),
                }
            }
            Err(e) => return Err(e.into()),
        },
        None => {
            log_warn!("ID为空, 请输入真实有效的 ID");
            exit(0)
        }
    }
}
