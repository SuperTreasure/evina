use core::{get_user_agent, live::douyu::get_real_id};
use std::{
    collections::HashMap,
    error::Error,
    fs::{create_dir_all, remove_dir_all, write},
    path::Path,
    time::{self},
};

use chrono::{DateTime, Duration as OtherDuration, Local, NaiveDate, NaiveDateTime, NaiveTime};
use chrono_tz::Asia::Shanghai;
use clap::Parser;
use logger_rust::{log_error, log_warn};
use md5::{Digest, Md5};
use reqwest::{header::USER_AGENT, Client};
use serde_json::Value;
use tokio_retry::{strategy::FixedInterval, Retry};
// use tokio_retry::strategy::FixedInterval;

pub async fn down_his(rid: Option<String>, date: Option<NaiveDate>) {
    let cli = core::Cli::parse();
    match rid {
        Some(id) => match date {
            Some(date) => match get_real_id(id.clone()).await {
                Ok(map) => {
                    let mut url = String::from("https://www.douyu.com/betard/") + &map["id"];
                    match Client::builder().connect_timeout(time::Duration::from_secs(20)).build() {
                        Ok(client) => match client.get(url).header(USER_AGENT, get_user_agent()).send().await {
                            Ok(response) => match response.json::<Value>().await {
                                Ok(result) => {
                                    let room = &result["room"];
                                    let up_id = room["up_id"].to_string().replace(r#"""#, "");
                                    let show_status = room["show_status"].as_i64().unwrap();
                                    let room_id = room["room_id"].as_i64().unwrap();
                                    let vipid = room["vipId"].as_i64().unwrap();
                                    let nickname = room["nickname"].to_string().replace(r#"""#, "");
                                    let room_name = &room["room_name"].to_string().replace(r#"""#, "");
                                    let second_lvl_name = &room["second_lvl_name"].to_string().replace(r#"""#, "");
                                    let show_time: DateTime<Local> = DateTime::from_timestamp(room["show_time"].as_i64().unwrap(), 0).unwrap().into();
                                    let end_time: DateTime<Local> =
                                        DateTime::from_timestamp(room["end_time"].to_string().replace(r#"""#, "").parse::<i64>().unwrap(), 0).unwrap().into();
                                    let nowtime: DateTime<Local> = DateTime::from_timestamp(room["nowtime"].as_i64().unwrap(), 0).unwrap().into();
                                    let club_org_name = &room["room_biz_all"]["clubOrgName"].to_string().replace(r#"""#, "");
                                    match show_status {
                                        1 => {
                                            println!("{}", "正在直播：");
                                            println!("    主播名：            {}", nickname);
                                            println!("    标题：              {}", room_name);
                                            println!("    房间号/靓号：       {}/{}", room_id, vipid);
                                            println!("    分类：              {}", second_lvl_name);
                                            println!("    公会：              {}", club_org_name);
                                            println!("    开播时间：          {}", show_time);
                                            println!("    查询时间：          {}", nowtime);
                                        }
                                        _ => {
                                            println!("{}", "未开播：");
                                            println!("    主播名：            {}", nickname);
                                            println!("    标题：              {}", room_name);
                                            println!("    房间号/靓号：       {}/{}", room_id, vipid,);
                                            println!("    分类：              {}", second_lvl_name);
                                            println!("    公会：              {}", club_org_name);
                                            println!("    开播时间：          {}", show_time);
                                            println!("    下播时间：          {}", end_time);
                                            println!("    查询时间：          {}", nowtime);
                                        }
                                    };
                                    let start_time =
                                        NaiveDateTime::new(date, NaiveTime::from_hms_opt(0, 0, 0).unwrap()).and_local_timezone(Shanghai).unwrap().timestamp();
                                    let end = date + OtherDuration::days(1);
                                    let end_time =
                                        NaiveDateTime::new(end, NaiveTime::from_hms_opt(0, 0, 0).unwrap()).and_local_timezone(Shanghai).unwrap().timestamp();
                                    url = format!(
                                        "https://v.douyu.com/wgapi/vod/center/authorShowVideoList?page=1&limit=5&up_id={}&start_time={}&end_time={}",
                                        up_id, start_time, end_time
                                    );
                                    match client.get(url).header(USER_AGENT, get_user_agent()).send().await {
                                        Ok(response) => {
                                            match response.json::<Value>().await {
                                                Ok(result) => {
                                                    match result["data"]["list"].as_array().unwrap().is_empty() {
                                                        true => log_error!("{} {}", date, "没有上传录播"),
                                                        false => {
                                                            let mut tasks = vec![];
                                                            for list in result["data"]["list"].as_array().unwrap() {
                                                                let video_list = &list["video_list"];
                                                                for list2 in video_list.as_array().unwrap() {
                                                                    let hash_id = &list2["hash_id"].to_string().replace(r#"""#, "");
                                                                    let up_id = std::sync::Arc::new(up_id.clone());
                                                                    match client
                                                                        .get(format!(
                                                                            "https://v.douyu.com/wgapi/vod/center/getShowReplayList?vid={}&up_id={}",
                                                                            hash_id, up_id
                                                                        ))
                                                                        .header(USER_AGENT, get_user_agent())
                                                                        .send()
                                                                        .await
                                                                    {
                                                                        Ok(response) => match response.json::<Value>().await {
                                                                            Ok(result) => {
                                                                                for list in result["data"]["list"].as_array().unwrap().clone() {
                                                                                    let hash_id = &list["hash_id"].to_string().replace(r#"""#, "");
                                                                                    let vid = &list["vid"].to_string().replace(r#"""#, "");
                                                                                    let show_remark = &list["show_remark"].to_string().replace(r#"""#, "");
                                                                                    match get_sign(hash_id.clone(), vid.clone()).await {
                                                                                        Ok(mut params) => {
                                                                                            params.insert("vid", hash_id.to_string());
                                                                                            match client.post("https://v.douyu.com/wgapi/vodnc/front/stream/getStreamUrlWeb").form(&params).header(USER_AGENT, get_user_agent()).send().await {
                                                                                        Ok(response) => match response.json::<Value>().await {
                                                                                            Ok(result) => {
                                                                                                let download_dir = Some(cli.download_dir.clone());
                                                                                                let nickname_clone = nickname.clone();
                                                                                                let show_remark_clone = show_remark.clone();
                                                                                                let client_clone = client.clone();

                                                                                                // 创建异步任务
                                                                                                let task = tokio::task::spawn(async move {
                                                                                                    match download_dir {
                                                                                                        Some(down_dir) => {
                                                                                                            let path = Path::new(&down_dir).join(format!("{}--录播下载", nickname_clone)).join(date.to_string())
                                                                                                                .join(show_remark_clone.clone());
                                                                                                            let super_url =
                                                                                                                result["data"]["thumb_video"]["super"]["url"].to_string().replace(r#"""#, "");
                                                                                                            let start_spuer_url = super_url.split_once("playlist.m3u8").unwrap().0;
                                                                                                            let _ = match path.exists() {
                                                                                                                true => {
                                                                                                                    let _ = remove_dir_all(&path.clone());
                                                                                                                    create_dir_all(path.clone())
                                                                                                                }
                                                                                                                false => create_dir_all(path.clone()),
                                                                                                            };
                                                                                                            match client_clone.get(super_url.clone()).header(USER_AGENT, get_user_agent()).send().await {
                                                                                                                Ok(response) => match response.text().await {
                                                                                                                    Ok(text) => {
                                                                                                                        for line in text.lines() {
                                                                                                                            if !line.trim().is_empty() && !line.trim().starts_with('#') {
                                                                                                                                let strategy = FixedInterval::from_millis(10000).take(cli.retry);
                                                                                                                                let down_spuer_url = format!("{}{}", start_spuer_url, line);
                                                                                                                                let name = core::re_match(r"_([0-9]+)\.ts", &down_spuer_url).unwrap();
                                                                                                                                let down_path = path.join(format!("{}--{}.ts", show_remark_clone.clone(), name));
                                                                                                                                let retry_result = Retry::spawn(strategy.clone(), || async {
                                                                                                                                    match client_clone
                                                                                                                                        .get(down_spuer_url.clone())
                                                                                                                                        .header(USER_AGENT, get_user_agent())
                                                                                                                                        .send()
                                                                                                                                        .await
                                                                                                                                    {
                                                                                                                                        Ok(response) => match response.bytes().await {
                                                                                                                                            Ok(bytes) => {
                                                                                                                                                let _ = write(down_path.clone(), bytes);
                                                                                                                                                Ok(format!("{}--{}",show_remark_clone.clone(),name.clone()))
                                                                                                                                            }
                                                                                                                                            Err(e) => Err(e),
                                                                                                                                        },
                                                                                                                                        Err(e) => {
                                                                                                                                            log_error!("{}",e);
                                                                                                                                            Err(e)
                                                                                                                                        },
                                                                                                                                    }
                                                                                                                                });
                                                                                                                                match retry_result.await {
                                                                                                                                    Ok(name) => println!("{}  下载完成",name),
                                                                                                                                    Err(e) => log_error!("{}", e),
                                                                                                                                }
                                                                                                                            }
                                                                                                                        }
                                                                                                                    },
                                                                                                                    Err(e) => log_error!("{}", e),
                                                                                                                },
                                                                                                                Err(e) => log_error!("{}", e),
                                                                                                            }
                                                                                                        },
                                                                                                        None => todo!(),
                                                                                                    }
                                                                                                });
                                                                                                tasks.push(task)
                                                                                            },
                                                                                            Err(e) => log_error!("{}", e),
                                                                                        },
                                                                                        Err(e) => log_error!("{}", e),
                                                                                    }
                                                                                        }
                                                                                        Err(e) => log_error!("{}", e),
                                                                                    }
                                                                                }
                                                                            }
                                                                            Err(e) => log_error!("{}", e),
                                                                        },
                                                                        Err(e) => log_error!("{}", e),
                                                                    }
                                                                }
                                                            }
                                                            for task in tasks {
                                                                let _ = task.await;
                                                            }
                                                        }
                                                    }
                                                }
                                                Err(e) => log_error!("{}", e),
                                            }
                                        }
                                        Err(e) => log_error!("{}", e),
                                    }
                                }
                                Err(e) => log_error!("{}", e),
                            },
                            Err(e) => log_error!("{}", e),
                        },
                        Err(e) => log_error!("{}", e),
                    }
                }
                Err(e) => log_error!("{}", e),
            },
            None => log_warn!("{}", "请输入需要下载的日期"),
        },
        None => log_warn!("{}", "ID 不能为空"),
    };
}

// https://v.douyu.com/wgapi/vodnc/front/stream/getStreamUrlWeb
// v=220320240116&did=0571372163a8f1d435b50f6e00031701&tt=1705336620&sign=83539e88d98cf595e1304456dedb864e&vid=NVm0WKN302wMJeBP

async fn get_sign(hash_id: String, vid: String) -> Result<HashMap<&'static str, String>, Box<dyn Error>> {
    let mut params = HashMap::new();
    match Client::builder().connect_timeout(time::Duration::from_secs(20)).build() {
        Ok(client) => {
            let url = format!("https://v.douyu.com/show/{}?ap=1:formatted", hash_id);
            match client.get(url).header(reqwest::header::USER_AGENT, get_user_agent()).send().await {
                Ok(response) => match response.text().await {
                    Ok(text) => {
                        let result = match core::re_match(r#"(vdwdae325w_64we[\s\S]*function ub98484234[\s\S]*?)function"#, &text) {
                            Some(data) => data,
                            None => return Err(format!("{} js代码获取错误", hash_id).into()),
                        };
                        let fun = regex::Regex::new(r"eval.*?<script>!").unwrap().replace_all(&result, "strc;}").to_string();
                        let res = core::run_js(&format!("{fun}ub98484234(0,0,0)"));
                        let v = core::re_match(r"v=(\d+)", &res.clone()).unwrap();
                        let tt = Local::now().timestamp_millis().to_string();
                        let tt = &tt.as_str()[0..10];
                        let rb = {
                            let mut h = Md5::new();
                            h.update(format!("{}10000000000000000000000000001501{}{}", vid, tt, v));
                            hex::encode(h.finalize())
                        };
                        let func_sign = regex::Regex::new(r"return rt;}\);?").unwrap().replace_all(&res, "return rt;}").to_string();

                        let func_sign = func_sign.replace(r#""(function "#, "function sign");
                        let func_sign = func_sign.replace(r#";""#, ";");
                        let func_sign = func_sign.replace(r#"}""#, "};");
                        let func_sign = func_sign.replace("CryptoJS.MD5(cb).toString()", format!(r#""{}""#, &rb).as_str());
                        let data = core::run_js(&format!(r#"{func_sign};sign({},10000000000000000000000000001501,{})"#, vid, tt));
                        let sign = data.trim_matches('"').to_string();
                        let sign = sign.rsplit_once("=").unwrap().1.to_string();
                        params.insert("v", v);
                        params.insert("did", "10000000000000000000000000001501".to_string());
                        params.insert("tt", tt.to_string());
                        params.insert("sign", sign);
                        return Ok(params);
                    }
                    Err(e) => return Err(e.into()),
                },
                Err(e) => return Err(e.into()),
            }
        }
        Err(e) => return Err(e.into()),
    }
}
