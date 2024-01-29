use chrono::Local;
use clap::Parser;
use logger_rust::log_error;
use std::{
    process::{exit, Command},
    thread,
};
use tokio_retry::{strategy::FixedInterval, Retry};

use super::Cli;
pub mod douyin;
pub mod douyu;

pub struct Information {
    pub platform: String,
    pub rid: String,
    pub name: String,
    pub rtmp: String,
}

impl Information {
    pub async fn print_information(&self) {
        let name = if self.platform == "斗鱼" { "斗鱼主播名: ".to_string() + &self.name } else { "抖音主播名: ".to_string() + &self.name };
        println!("{}{} | 房间号ID: {}", "\n", name, self.rid);
        println!("{}{}", "\n", self.rtmp);
        if !self.rtmp.contains("未开播") {
            Information::ff(self).await;
        }
    }

    async fn ff(&self) {
        let cli = Cli::parse();
        let mut tasks = Vec::new();
        let ffm = match cli.ffm {
            true => Information::ffm(self).await.map(|handle| Some(handle)),
            false => Ok(None),
        };
        let ffp = match cli.ffp {
            true => Information::ffp(self).await.map(|handle| Some(handle)),
            false => Ok(None),
        };

        tasks.push(ffm);
        tasks.push(ffp);
        for task in tasks {
            match task {
                Ok(Some(t)) => t.join().unwrap(),
                Ok(None) => (),
                Err(e) => log_error!("{}", e),
            }
        }
    }

    async fn ffm(&self) -> Result<thread::JoinHandle<()>, std::io::Error> {
        super::check_env("ffmpeg").await;
        let cli = Cli::parse();
        // let fmt = "%Y年%m月%d日-%H时%M分%S秒";
        let fmt = "%Y-%m-%d";
        let now = Local::now().format(fmt);
        let path = std::path::Path::new(&cli.download_dir).join(format!("{}录播/{}/{now}", self.platform, self.name));
        let save = format!("{}/%Y-%m-%d-%H-%M-%S.mp4", path.display());
        // let _ = std::fs::create_dir_all(path);
        match std::fs::create_dir_all(path) {
            Ok(_) => {
                let ffmpeg = format!(r#"ffmpeg -i "{}" -c:a copy -c:v libx264 -b:v 3072k -f segment -segment_time 3600 -strftime 1 "{save}""#, self.rtmp);
                let ffmpeg = shell_words::split(&ffmpeg).unwrap();
                let ffm = thread::Builder::new().name("ffm".to_owned()).spawn(move || {
                    Command::new(ffmpeg[0].clone()).args(&ffmpeg[1..]).output().expect("录播程序错误");
                });
                return ffm;
            }
            Err(e) => return Err(e.into()),
        }
    }

    async fn ffp(&self) -> Result<thread::JoinHandle<()>, std::io::Error> {
        super::check_env("ffmpeg").await;
        let cli = Cli::parse();
        let resolution = cli.resolution.replace(" ", "");
        let resolution_vec = resolution.split_once("x");
        let width = resolution_vec.unwrap().0;
        let height = resolution_vec.unwrap().1;
        let strategy = FixedInterval::from_millis(1000).take(2);
        let result = Retry::spawn(strategy, || async {
            match self.platform.as_str() {
                "斗鱼" => douyu::get_rtmp_url(Some(self.rid.clone())).await,
                "抖音" => douyin::get_rtmp_url(Some(self.rid.clone())).await,
                _ => exit(0),
            }
        });
        let info = match result.await {
            Ok(info) => info,
            Err(e) => {
                log_error!("ffplay: {}", e);
                exit(0)
            }
        };
        let ffplay = format!(r#"ffplay -x {} -y {} -i "{}""#, width, height, info.rtmp);
        let ffplay = shell_words::split(&ffplay).unwrap();

        let ffp = thread::Builder::new().name("ffp".to_owned()).spawn(move || {
            Command::new(ffplay[0].clone()).args(&ffplay[1..]).output().expect("播放程序错误");
        });
        return ffp;
    }
}
