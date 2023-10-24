use std::cell::RefCell;

use boa_engine::Context;
use clap::{Parser, Subcommand};
use fake::{faker::internet::raw::*, locales::*, Fake};
use logger_rust::{log_error, log_warn};
use regex::Regex;
use tokio::runtime::Runtime;

use super::{douyin, douyu, Information};

#[derive(Parser)]
#[command(author = env!("CARGO_PKG_NAME"),version = env!("CARGO_PKG_VERSION"),
about=r"
____________________________________________________

                        _                 
          ___  __   __ (_)  _ __     __ _ 
         / _ \ \ \ / / | | | '_ \   / _` |
        |  __/  \ V /  | | | | | | | (_| |
         \___|   \_/   |_| |_| |_|  \__,_|

____________________________________________________
                                   ",
long_about = None,)]
#[command(subcommand_negates_reqs = true)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    /// 需要录制的平台
    #[arg(short = 'l', long,value_parser=["douyu", "douyin"])]
    pub live: Option<String>,
    /// 直播间号或链接
    #[arg(short = 'i', long)]
    pub id: Option<String>,
    /// 播放器的宽度
    #[arg(short = 'x',default_value_t = String::from("1366"))]
    pub x: String,
    /// 播放器的高度
    #[arg(short = 'y',default_value_t = String::from("768"))]
    pub y: String,
    /// 自定义录制的目录，默认保存到根目录下的download
    #[arg(short = 'd',default_value_t = String::from("/download"))]
    pub download: String,
    /// 是否开启录播，默认不开启
    #[arg(short = 'm', long)]
    pub ffm: bool,
    /// 是否开启播放功能，默认不开启
    #[arg(short = 'p', long)]
    pub ffp: bool,
    /// 使用配置文件配置的id
    #[arg(short = 'r', long)]
    pub read: bool,
    #[cfg(target_os = "windows")]
    /// 自定义配置文件的路径
    #[arg(short='f',long,default_value_t={
    let root = std::env::var("USERPROFILE").ok().unwrap();
    format!("{}\\.evina\\config",root)
})]
    pub file: String,
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    /// 自定义配置文件的路径
    #[arg(short='f',long,default_value_t={
    let root = std::env::var("HOME").ok().unwrap();
    format!("{}/.evina/config",root)
})]
    pub file: String,
    #[command(subcommand)]
    pub sub: Option<Config>,
}

#[derive(Debug, Subcommand)]
pub enum Config {
    /// 编辑配置文件
    Config {
        /// 格式化配置文件
        #[arg(short = 'r', long)]
        reload: bool,
        /// 列出配置文件的所有配置
        #[arg(short = 'l', long)]
        list: bool,
        /// 添加变量到配置文件
        #[arg(short = 'a', long)]
        add: Option<String>,
        /// 删除配置文件的变量
        #[arg(short = 'd', long)]
        del: Option<String>,
        /// 创建配置文件的软链接
        #[arg(short = 's', long)]
        symlink: Option<String>,
    },
}

pub fn retries(num: RefCell<i32>) {
    if *num.borrow_mut() == 1 {
        log_warn!("获取失败,正在尝试重新连接,最大重新连接次数为 8");
        log_warn!("第 {} 次重新连接", *num.borrow_mut());
    } else if *num.borrow_mut() < 9 {
        log_warn!("第 {} 次重新连接", *num.borrow_mut());
    } else {
        log_error!("获取失败");
    }
}

pub fn get_user_agent() -> String {
    loop {
        let add = UserAgent(ZH_CN).fake::<String>();
        if !add.contains("iPhone")
            && !add.contains("Windows CE")
            && !add.contains("Android")
            && !add.contains("WIndows NT")
            && !add.contains("Windows NT")
            && !add.contains("Linux i686")
            && !add.contains("iPad")
        {
            return add;
        }
    }
}

pub fn re_match(re: &str, data: &str) -> Option<String> {
    let result = Regex::new(re).unwrap().captures(&data).map(|caps| caps.get(1).unwrap().as_str().to_owned());
    match result {
        Some(result) => Some(result),
        None => None,
    }
}

pub fn run_js(js: &str) -> String {
    let mut context = Context::default();
    return context.eval(js).unwrap().display().to_string();
}

// 判断 id 是否包含字母
pub fn is_alphabetic(id: String) -> bool {
    let mut res = false;
    for c in id.chars() {
        if c.is_alphabetic() {
            res = true;
            break;
        }
    }
    return res;
}

pub async fn thread_run(hashmap: std::collections::HashMap<String, Option<String>>) {
    let mut tasks = Vec::new();

    for (key, value) in hashmap.iter() {
        let key = key.clone();
        let value = value.clone().unwrap();

        let run = std::thread::spawn(move || {
            // 创建新的运行时实例
            let runtime = Runtime::new().unwrap();

            // 使用运行时执行异步代码
            runtime.block_on(async {
                match key.as_str() {
                    key if key.contains("DOUYU") && value != "xxxxxx" => match douyu::get_rtmp_url(Some(value)).await {
                        Ok(info) => Information::print_information(&info).await,
                        Err(e) => log_error!("{}: {}", key, e),
                    },
                    key if key.contains("DOUYIN") && value != "xxxxxx" => {
                        match douyin::get_rtmp_url(Some(value)).await {
                            Ok(info) => Information::print_information(&info).await,
                            Err(e) => log_error!("{}: {}", key, e),
                        }
                    }
                    _ => log_warn!("请检查配置: {}", key),
                }
            })
        });
        tasks.push(run);
    }

    for task in tasks {
        let _ = task.join();
    }
}
