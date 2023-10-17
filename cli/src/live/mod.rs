pub mod douyin;
pub mod douyu;
use boa_engine::Context;
use chrono::Local;
use clap::{Parser, Subcommand};
use fake::locales::*;
use fake::Fake;
use regex::Regex;
use reqwest;
use std::{process::exit, process::Command, thread};

use self::douyin::douyin;
use self::douyu::douyu;

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

#[derive(Debug)]
pub struct Information {
    platform: String,
    id: String,
    name: String,
    rtmp: String,
}

impl Information {
    pub fn to_print(&self) {
        let name = if self.platform == "斗鱼" {
            "斗鱼主播名: ".to_string() + &self.name
        } else {
            "抖音主播名: ".to_string() + &self.name
        };
        println!("{}{} | 房间号ID: {}", "\n", name, self.id);
        println!("{}{}", "\n", self.rtmp);
        if !self.rtmp.contains("未开播") {
            Information::ff(self);
        }
    }

    fn ff(&self) {
        let cli = Cli::parse();
        let mut tasks = Vec::new();

        let ffm: Option<Result<thread::JoinHandle<()>, std::io::Error>> = if cli.ffm == true {
            Some(Information::ffm(self))
        } else {
            None
        };

        let ffp: Option<Result<thread::JoinHandle<()>, std::io::Error>> = if cli.ffp == true {
            Some(Information::ffp(self))
        } else {
            None
        };

        tasks.push(ffm);
        tasks.push(ffp);
        for task in tasks {
            let _ = match task {
                Some(t) => t.unwrap().join(),
                None => Ok(()),
            };
        }
    }

    fn ffm(&self) -> Result<thread::JoinHandle<()>, std::io::Error> {
        let cli = Cli::parse();
        let fmt = "%Y年%m月%d日-%H时%M分%S秒";
        let now = Local::now().format(fmt);
        let path = std::path::Path::new(&cli.download).join(format!("{}/{}/{now}", self.platform,self.name));
        let save = format!("{}/%Y-%m-%d-%H-%M-%S.mp4", path.display());
        let _ = std::fs::create_dir_all(path);

        let ffmpeg = format!(
            r#"ffmpeg -t 18000 -i "{}" -c:a copy -c:v copy -f segment -segment_time 3600 -strftime 1 "{save}""#,
            self.rtmp
        );
        let ffmpeg = shell_words::split(&ffmpeg).unwrap();
        let ffm = thread::Builder::new()
            .name("ffm".to_owned())
            .spawn(move || {
                if cfg!(target_os = "windows") {
                    Command::new(ffmpeg[0].clone())
                        .args(&ffmpeg[1..])
                        .output()
                        .expect("录播程序错误")
                } else {
                    Command::new(ffmpeg[0].clone())
                        .args(&ffmpeg[1..])
                        .output()
                        .expect("录播程序错误")
                };
            });
        return ffm;
    }

    fn ffp(&self) -> Result<thread::JoinHandle<()>, std::io::Error> {
        let cli = Cli::parse();
        let info: Information = if self.platform == "斗鱼" {
            douyu::douyu(self.id.clone()).unwrap()
        } else if self.platform == "抖音" {
            douyin::douyin(self.id.clone()).unwrap()
        } else {
            exit(0x0100)
        };
        
        let ffplay = format!(r#"ffplay -x {} -y {} -i "{}""#, cli.x, cli.y, info.rtmp);
        let ffplay = shell_words::split(&ffplay).unwrap();

        let ffp = thread::Builder::new()
            .name("ffp".to_owned())
            .spawn(move || {
                if cfg!(target_os = "windows") {
                    Command::new(ffplay[0].clone())
                        .args(&ffplay[1..])
                        .output()
                        .expect("播放程序错误")
                } else {
                    Command::new(ffplay[0].clone())
                        .args(&ffplay[1..])
                        .output()
                        .expect("录播程序错误")
                };
            });
        return ffp;
    }
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

pub fn useag() -> String {
    use fake::faker::internet::raw::*;
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

pub fn re_result(re: &str, data: String) -> Result<String, std::io::Error> {
    let result = Regex::new(re)
        .unwrap()
        .captures(&data)
        .map(|caps| caps.get(1).unwrap().as_str().to_owned());
    match result {
        Some(result) => Ok(result),
        None => {
            println!("{}", "未找到直播间");
            std::process::exit(0x0100)
        }
    }
}
pub async fn eval(js: &str) -> String {
    let mut context = Context::default();
    match context.eval(js) {
        Ok(result) => result.display().to_string(),
        Err(e) => e.display().to_string(),
    }
}

pub fn thread_run(hashmap: std::collections::HashMap<String, Option<String>>) {
    let mut tasks = Vec::new();
    for (key, value) in hashmap.iter() {
        let key = key.clone();
        let value = value.clone().unwrap();
        let run = thread::Builder::new().name(value.clone()).spawn(move || {
            if key.contains("DOUYU") && value != "xxxxxx" {
                Information::to_print(&douyu(value).unwrap())
            } else if key.contains("DOUYIN") && value != "xxxxxx" {
                Information::to_print(&douyin(value).unwrap())
            }
        });
        tasks.push(run);
    }

    for task in tasks {
        let _ = match task {
            Ok(t) => t.join(),
            Err(_) => todo!(),
        };
    }
}
