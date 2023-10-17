use chrono::Local;
use md5::{Digest, Md5};
use reqwest::Client;
use serde_json::Value;

use crate::live::{eval, re_result, reqwest, useag, Information};

#[tokio::main]
pub async fn douyu(rid: String) -> Result<Information, Box<dyn std::error::Error>> {
    let client = Client::new();
    let id: String;
    if rid.clone().contains("http") {
        id = rid.clone();
    } else {
        id = String::from("https://m.douyu.com/") + &rid.clone();
    }

    let response = client
        .get(id)
        .header(reqwest::header::USER_AGENT, useag())
        .send()
        .await?
        .text()
        .await?;
    // 得到真实的直播间id。
    let real_id = re_result(r#"rid":(\d{1,8}),"vipId"#, response.clone()).unwrap();
    // 得到直播间名称。
    let re_name = re_result(r#"nickname":"(.*?)","#, response.clone()).unwrap();
    // 得到 rtmp 流地址
    let rtmp_json = get_rtmp(real_id.clone(), client.clone()).await?;

    let rtmp_url = rtmp_json["data"]["rtmp_url"].as_str();

    match rtmp_url {
        Some(rtmp) => Ok(Information {
            platform: "斗鱼".to_owned(),
            id: real_id,
            name:re_name,
            rtmp: rtmp.to_owned() + "/" + rtmp_json["data"]["rtmp_live"].as_str().unwrap(),
        }),
        None => Ok(Information {
            platform: "斗鱼".to_owned(),
            id: real_id,
            name:re_name,
            rtmp: "直播间未开播".to_owned(),
        }),
    }
}

/// 得到 PC 端的 rtmp
async fn get_rtmp(id: String, client: Client) -> Result<Value, Box<dyn std::error::Error>> {
    let url = String::from("https://www.douyu.com/") + &id;
    let response = client
        .get(url)
        .header(reqwest::header::USER_AGENT, useag())
        .send()
        .await?
        .text()
        .await?;
    let result = re_result(
        r#"(vdwdae325w_64we[\s\S]*function ub98484234[\s\S]*?)function"#,
        response,
    )
    .unwrap();

    let fun = regex::Regex::new(r"eval.*?;}")
        .unwrap()
        .replace_all(&result, "strc;}")
        .to_string();
    let res = eval(&format!("{fun}ub98484234(0,0,0)")).await;
    let v = re_result(r"v=(\d+)", res.clone()).unwrap();
    let tt = Local::now().timestamp_millis().to_string();
    let tt = &tt.as_str()[0..10];
    let rb = {
        let mut h = Md5::new();
        h.update(format!("{id}10000000000000000000000000001501{tt}{v}"));
        hex::encode(h.finalize())
    };
    let func_sign = regex::Regex::new(r"return rt;}\);?")
        .unwrap()
        .replace_all(&res, "return rt;}")
        .to_string();

    let func_sign = func_sign.replace(r#""(function "#, "function sign");
    let func_sign = func_sign.replace(r#";""#, ";");
    let func_sign = func_sign.replace(r#"}""#, "};");
    let func_sign = func_sign.replace(
        "CryptoJS.MD5(cb).toString()",
        format!(r#""{}""#, &rb).as_str(),
    );
    let sign = eval(&format!(
        r#"{func_sign};sign({id},10000000000000000000000000001501,{tt})"#
    ))
    .await
    .trim_matches('"')
    .to_string();
    let sign = sign.rsplit_once("=").unwrap().1.to_string();
    let mut params = std::collections::HashMap::new();
    params.insert("v", v);
    params.insert("did", "10000000000000000000000000001501".to_string());
    params.insert("tt", tt.to_string());
    params.insert("sign", sign);
    params.insert("cdn", "scdncthubyc".to_string());
    params.insert("rate", 0.to_string());

    let rtmp = client
        .post(format!("https://www.douyu.com/lapi/live/getH5Play/{id}"))
        .form(&params)
        .header(reqwest::header::USER_AGENT, useag())
        .send()
        .await?
        .json::<Value>()
        .await?;

    return Ok(rtmp);
}
