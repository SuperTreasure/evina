pub mod live;

use std::{
    cell::RefCell,
    path::Path,
    process::{exit, Command},
    time,
};

use boa_engine::Context;
use chrono::{Duration, Local, NaiveDate};
use clap::{Parser, Subcommand};
use fake::{faker::internet::raw::*, locales::*, Fake};
use live::{douyin, douyu, Information};
use logger_rust::{log_error, log_warn};
use regex::Regex;
use tokio::runtime::Runtime;

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
    /// 需要录制的平台 | 支持的平台： 斗鱼(仅支持房间号)、抖音
    #[arg(long,short = 'l', value_parser=["douyu", "douyin"])]
    pub live: Option<String>,
    /// 直播间号或链接
    #[arg(long, short = 'i')]
    pub id: Option<String>,
    /// 是否开启录播，默认不开启
    #[arg(long, short = 'm')]
    pub ffm: bool,
    /// 是否开启播放功能，默认不开启
    #[arg(long, short = 'p')]
    pub ffp: bool,
    /// 尝试重试的次数
    #[arg(long, default_value_t = 0)]
    pub retry: usize,
    /// 播放器的分辨率，默认为 1366x768
    #[arg(long,default_value_t = String::from("1366x768"))]
    pub resolution: String,
    /// 自定义录制的目录，默认保存到根目录下的download
    #[arg(long,default_value_t = {
            // 获取当前工作目录
    let mut current_dir = std::env::current_dir().unwrap();
    current_dir.push("/download");
    current_dir.display().to_string()
    })]
    pub download_dir: String,
    /// 使用配置文件配置的id
    #[arg(long)]
    pub config: bool,
    #[cfg(target_os = "windows")]
    /// 自定义配置文件的路径
    #[arg(long,default_value_t={
    let root = std::env::var("USERPROFILE").ok().unwrap();
    format!("{}\\.evina\\config",root)
})]
    pub config_file: String,
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    /// 自定义配置文件的路径
    #[arg(long,default_value_t={
    let root = std::env::var("HOME").ok().unwrap();
    format!("{}/.evina/config",root)
})]
    pub config_file: String,
    #[command(subcommand)]
    pub sub: Option<Sub>,
}

#[derive(Debug, Subcommand)]
pub enum Sub {
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
    /// 下载历史直播的录屏
    History {
        /// 需要下载的平台,支持斗鱼
        #[arg(short='l',long,value_parser=["douyu"])]
        live: Option<String>,
        /// 需要下载的直播间号或链接
        #[arg(short = 'i', long)]
        id: Option<String>,
        /// 需要下载的日期，注意格式为 2024-01-13
        #[arg(short = 'd', long,default_value_t = {
            let current_time = Local::now();
            // 计算昨天的日期
            let yesterday = current_time - Duration::days(1);
            let naive_date = yesterday.date_naive();
            naive_date
        })]
        date: NaiveDate,
    },
}

pub fn retries(num: RefCell<i32>) {
    let cli = Cli::parse();
    match cli.retry {
        i if i > 0 => {
            if *num.borrow_mut() == 1 {
                log_warn!("获取失败,正在尝试重新连接,最大重新连接次数为 {}", i);
                log_warn!("第 {} 次重新连接", *num.borrow_mut());
            } else if *num.borrow_mut() < (i + 1).try_into().unwrap() {
                log_warn!("第 {} 次重新连接", *num.borrow_mut());
            } else {
                log_error!("获取失败");
            }
        }
        _ => log_error!("获取失败"),
    };
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
                    key if key.contains("DOUYIN") && value != "xxxxxx" => match douyin::get_rtmp_url(Some(value)).await {
                        Ok(info) => Information::print_information(&info).await,
                        Err(e) => log_error!("{}: {}", key, e),
                    },
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

pub async fn check_env(key: &str) {
    use reqwest::Client;
    match key {
        "ffmpeg" => match Command::new(key).output() {
            Ok(_) => {}
            Err(_) => {
                #[cfg(target_os = "windows")]
                let url = "https://mirror.ghproxy.com/https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip";
                #[cfg(target_os = "linux")]
                let url = "https://mirror.ghproxy.com/https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-linux64-gpl.tar.xz";
                let exe_path = std::env::current_exe();
                match exe_path {
                    Ok(mut exe_path) => {
                        exe_path.pop();
                        let check = Path::exists(&Path::new(&exe_path).join("depend/ffmpeg"));
                        match check {
                            true => {}
                            false => {
                                let name = url.rsplit_once("/").unwrap().1;
                                match download(url, name).await {
                                    Ok(_) => {
                                        #[cfg(target_os = "windows")]
                                        Command::new("powershell")
                                            .args(["Expand-Archive", "-Path", name, "-DestinationPath", Path::new(&exe_path).join("depend").to_str().unwrap()])
                                            .output()
                                            .expect("解压失败");
                                        #[cfg(target_os = "linux")]
                                        {
                                            let _ = std::fs::create_dir_all(Path::new(&exe_path).join("depend"));
                                            Command::new("tar")
                                                .args(["-xvf", name, "-C", Path::new(&exe_path).join("depend").to_str().unwrap()])
                                                .output()
                                                .expect("解压失败");
                                        }
                                        let dir_name = name.split_once(".").unwrap().0;
                                        // let _ = std::fs::rename(Path::new(&exe_path).join(dir_name), Path::new(&exe_path).join("depend"));
                                        let _ = std::fs::rename(
                                            Path::new(&exe_path).join("depend").join(dir_name),
                                            Path::new(&exe_path).join("depend").join("ffmpeg"),
                                        );
                                        let _ = std::fs::remove_file(name);
                                        std::env::set_var("Path", Path::new(&exe_path).join("depend").join("ffmpeg").join("bin"));
                                    }
                                    Err(_) => {
                                        log_error!("下载失败，请尝试手动下载，并添加环境变量\n{}", url);
                                        exit(0)
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => {
                        log_error!("下载失败，请尝试手动下载，并添加环境变量\n{}", url);
                        exit(0)
                    }
                }
            }
        },
        "yt-dlp" => match Command::new(key).output() {
            Ok(_) => {}
            Err(_) => {
                let exe_path = std::env::current_exe();
                #[cfg(target_os = "windows")]
                let url = "https://mirror.ghproxy.com/https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe";
                #[cfg(target_os = "linux")]
                let url = "https://mirror.ghproxy.com/https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp";
                match exe_path {
                    Ok(mut exe_path) => {
                        exe_path.pop();
                        let check = Path::exists(&Path::new(&exe_path).join("depend").join("yt-dlp"));
                        match check {
                            true => {}
                            false => {
                                let _ = std::fs::create_dir_all(Path::new(&exe_path).join("depend/yt-dlp"));
                                let name = url.rsplit_once("/").unwrap().1;
                                match download(url, name).await {
                                    Ok(_) => {
                                        let _ = std::fs::rename(name, Path::new(&exe_path).join("depend").join("yt-dlp").join(name));
                                        std::env::set_var("Path", Path::new(&exe_path).join("depend").join("yt-dlp"));
                                    }
                                    Err(_) => {
                                        log_error!("下载失败，请尝试手动下载，并添加环境变量\n{}", url);
                                        exit(0)
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => {
                        log_error!("下载失败，请尝试手动下载，并添加环境变量\n{}", url);
                        exit(0)
                    }
                }
            }
        },
        _ => todo!(),
    }
    async fn download(url: &str, name: &str) -> Result<(), reqwest::Error> {
        match Client::builder().connect_timeout(time::Duration::from_secs(20)).build() {
            Ok(client) => match client.get(url).send().await {
                Ok(response) => match response.bytes().await {
                    Ok(bytes) => {
                        let _ = std::fs::write(format!("./{}", name), bytes);
                        Ok(())
                    }
                    Err(e) => return Err(e),
                },
                Err(e) => return Err(e),
            },
            Err(e) => return Err(e),
        }
    }
}
