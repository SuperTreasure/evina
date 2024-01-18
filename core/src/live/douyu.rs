use std::{collections::HashMap, error::Error, time};

use super::{
    super::{get_user_agent, re_match, run_js},
    Information,
};
use chrono::Local;
use logger_rust::log_warn;
use md5::{Digest, Md5};
use reqwest::Client;
use serde_json::Value;
use std::process::exit;

pub async fn get_real_id(rid: String) -> Result<HashMap<&'static str, String>, Box<dyn Error>> {
    let client = Client::builder()
        .connect_timeout(time::Duration::from_secs(20))
        .build()?;
    let mut map = HashMap::new();
    match rid.as_str() {
        i if i.contains("http") => Err(format!("ID格式错误, 仅支持房间号的方式").into()),
        _ => {
            let id = String::from("https://m.douyu.com/") + &rid.clone();
            let response = match client
                .get(id.clone())
                .header(reqwest::header::USER_AGENT, get_user_agent())
                .send()
                .await
            {
                Ok(response) => response,
                Err(e) => return Err(e.into()),
            };
            match response.text().await {
                Ok(text) => {
                    // 得到真实的直播间id。
                    let real_id = match re_match(r#"rid":(\d{1,8}),"vipId"#, &text.clone()) {
                        Some(data) => data,
                        None => {
                            return Err(
                                format!("斗鱼 | {}: 真实ID获取失败或找不到直播间", rid).into()
                            )
                        }
                    };
                    // 得到直播间名称。
                    let re_name = match re_match(r#"nickname":"(.*?)","#, &text.clone()) {
                        Some(data) => data,
                        None => return Err(format!("斗鱼 | {}: 直播间名称获取失败", rid).into()),
                    };
                    map.insert("id", real_id);
                    map.insert("name", re_name);
                    return Ok(map);
                }
                Err(e) => return Err(e.into()),
            };
        }
    }
}

pub async fn get_rtmp_url(id: Option<String>) -> Result<Information, Box<dyn Error>> {
    let client = Client::builder()
        .connect_timeout(time::Duration::from_secs(20))
        .build()
        .unwrap();
    match id {
        Some(id) => match get_real_id(id.clone()).await {
            Ok(map) => {
                let url = String::from("https://www.douyu.com/") + &map["id"];
                let response = match client
                    .get(url)
                    .header(reqwest::header::USER_AGENT, get_user_agent())
                    .send()
                    .await
                {
                    Ok(response) => response,
                    Err(e) => return Err(e.into()),
                };
                match response.text().await {
                    Ok(text) => {
                        let result = match re_match(
                            r#"(vdwdae325w_64we[\s\S]*function ub98484234[\s\S]*?)function"#,
                            &text,
                        ) {
                            Some(data) => data,
                            None => {
                                return Err(format!("斗鱼 | {}: 未找到直播间,请检查ID", id).into())
                            }
                        };
                        let fun = regex::Regex::new(r"eval.*?;}")
                            .unwrap()
                            .replace_all(&result, "strc;}")
                            .to_string();
                        let res = run_js(&format!("{fun}ub98484234(0,0,0)"));
                        let v = re_match(r"v=(\d+)", &res.clone()).unwrap();
                        let tt = Local::now().timestamp_millis().to_string();
                        let tt = &tt.as_str()[0..10];
                        let rb = {
                            let mut h = Md5::new();
                            h.update(format!(
                                "{}10000000000000000000000000001501{}{}",
                                map["id"], tt, v
                            ));
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
                        let data = run_js(&format!(
                            r#"{func_sign};sign({},10000000000000000000000000001501,{})"#,
                            map["id"], tt
                        ));
                        let sign = data.trim_matches('"').to_string();
                        let sign = sign.rsplit_once("=").unwrap().1.to_string();
                        let mut params = HashMap::new();
                        params.insert("v", v);
                        params.insert("did", "10000000000000000000000000001501".to_string());
                        params.insert("tt", tt.to_string());
                        params.insert("sign", sign);
                        params.insert("cdn", "scdncthubyc".to_string());
                        params.insert("rate", 0.to_string());
                        let response = match client
                            .post(format!(
                                "https://www.douyu.com/lapi/live/getH5Play/{}",
                                map["id"]
                            ))
                            .form(&params)
                            .header(reqwest::header::USER_AGENT, get_user_agent())
                            .send()
                            .await
                        {
                            Ok(resp) => resp,
                            Err(e) => return Err(e.into()),
                        };

                        match response.json::<Value>().await {
                            Ok(rtmp_json) => {
                                let rtmp_url = rtmp_json["data"]["rtmp_url"].as_str();
                                match rtmp_url {
                                    Some(rtmp) => Ok(Information {
                                        platform: "斗鱼".to_owned(),
                                        rid: map["id"].clone(),
                                        name: map["name"].clone(),
                                        rtmp: rtmp.to_owned()
                                            + "/"
                                            + rtmp_json["data"]["rtmp_live"].as_str().unwrap(),
                                    }),
                                    None => Ok(Information {
                                        platform: "斗鱼".to_owned(),
                                        rid: map["id"].clone(),
                                        name: map["name"].clone(),
                                        rtmp: "直播间未开播".to_owned(),
                                    }),
                                }
                            }
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
