use crate::live::{is_alphabetic, re_result, useag, Cli, Information};
use clap::Parser;
use cookie::{auto_cookie, local_cookie};
use reqwest::{
    header::{ACCEPT, ACCEPT_LANGUAGE, COOKIE, USER_AGENT},
    Client,
};
use serde_json::Value;

const _ACCEPT:&str = "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7";
const _ACCEPT_LANGUAGE: &str = "zh-CN,zh;q=0.9,en;q=0.8,en-GB;q=0.7,en-US;q=0.6";

#[tokio::main]
pub async fn douyin(rid: String) -> Result<Information, Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let client = Client::new();
    // 直播间地址以 / 结尾 如: https://v.douyin.com/ie6wQq5g/
    // 去除结尾 / 返回去除后的地址
    let mut id: String = if rid.clone().ends_with("/") {
        rid.clone().rsplit_once("/").unwrap().0.to_string()
    } else {
        rid.clone()
    };

    // 直播间以链接形式
    if id.clone().contains("http") {
        // 切割直播间地址，获取直播间 id
        id = id.rsplit_once("/").unwrap().1.to_string();

        // 判断 id 是否含有字母，含字母为手机端临时 id
        if is_alphabetic(id.clone()) {
            let real_id = phone(cli, format!("https://v.douyin.com/{}", id), client.clone()).await;
            id = match real_id {
                Ok(id) => id,
                Err(_) => todo!(),
            }
        }
    } else {
        if is_alphabetic(id.clone()) {
            let real_id = phone(
                cli,
                format!("https://v.douyin.com/{}", id.clone()),
                client.clone(),
            )
            .await;
            id = match real_id {
                Ok(id) => id,
                Err(_) => todo!(),
            }
        }
    }

    // 获取直播间 rtmp 流地址
    return get_rtmp(id.clone(), client).await;
}

async fn phone(
    cli: Cli,
    mut url: String,
    client: Client,
) -> Result<String, Box<dyn std::error::Error>> {
    let resp = client.get(url).header(USER_AGENT, useag()).send().await?;
    if resp.url().query() == None {
        println!("链接失效，请填写新链接");
        std::process::exit(0x0100)
    } else {
        url = format!(
            "https://www.douyin.com{}",
            resp.url().path().rsplit_once("share").unwrap().1
        );
        let resp = client
            .get(url)
            .header(USER_AGENT, useag())
            .header(
                COOKIE,
                local_cookie(cli.file, String::from("cookie_douyin")).unwrap(),
            )
            .header(ACCEPT, _ACCEPT)
            .header(ACCEPT_LANGUAGE, _ACCEPT_LANGUAGE)
            .send()
            .await?
            .text()
            .await?;
        let id = re_result(r#"https://live.douyin.com/(\d+)"#, resp.clone());
        match id {
            Ok(_) => Ok(id?),
            Err(_) => todo!(),
        }
    }
}

async fn get_rtmp(id: String, client: Client) -> Result<Information, Box<dyn std::error::Error>> {
    let url = format!("https://live.douyin.com/webcast/room/web/enter/?aid=6383&app_name=douyin_web&live_id=1&device_platform=web&language=zh-CN&enter_from=web_live&cookie_enabled=true&screen_width=1366&screen_height=768&browser_language=zh-CN&browser_platform=Win32&browser_name=Edge&browser_version=116.0.0.0&web_rid={}&room_id_str=7248777764587817767&enter_source=&Room-Enter-User-Login-Ab=0&is_need_double_stream=false",id);
    let resp = client
        .get(url)
        .header(USER_AGENT, useag())
        .header(
            COOKIE,
            auto_cookie(format!("https://live.douyin.com/{}", id)).await?,
        )
        .send()
        .await?
        .json::<Value>()
        .await?;
    let rtmp = resp["data"]["data"][0]["stream_url"]["flv_pull_url"].as_object();
    match rtmp {
        Some(rtmp) => Ok(Information {
            platform: "抖音".to_owned(),
            id: id.clone(),
            name: resp["data"]["user"]["nickname"]
                .to_string()
                .replace(r#"""#, ""),
            rtmp: rtmp
                .values()
                .rev()
                .last()
                .unwrap()
                .to_string()
                .replace(r#"""#, ""),
        }),
        None => Ok(Information {
            platform: "抖音".to_owned(),
            id: id.clone(),
            name: resp["data"]["user"]["nickname"]
                .to_string()
                .replace(r#"""#, ""),
            rtmp: "直播间未开播".to_owned(),
        }),
    }
}
