mod live;

use clap::Parser;
use live::util;
use logger_rust::log_error;
use std::{cell::RefCell, process::exit};
use tokio_retry::{strategy::FixedInterval, Retry};

#[tokio::main]
async fn main() {
    // 解析命令行参数
    let cli = live::util::Cli::parse();
    // 设置重试策略
    let strategy = FixedInterval::from_millis(1000).take(8);
    // 判断cli的值
    match cli.live {
        Some(data) => {
            let retries: RefCell<i32> = RefCell::new(0);
            let result = Retry::spawn(strategy, || async {
                match data.as_str() {
                    "douyu" => live::douyu::get_rtmp_url(cli.id.clone()).await,
                    "douyin" => live::douyin::get_rtmp_url(cli.id.clone()).await,
                    _ => todo!(),
                }
            });
            let info = match result.await {
                Ok(info) => Ok(info),
                Err(e) => {
                    *retries.borrow_mut() += 1;
                    log_error!("Error: {}", e);
                    live::util::retries(retries.clone());
                    Err(e)
                }
            };
            match info {
                Ok(info) => live::Information::print_information(&info).await,
                Err(_) => exit(0),
            }
        }
        None => match cli.read {
            true => {
                let list = vec!["DOUYU", "DOUYIN"];
                match cookie::read_config(cli.file.clone(), list) {
                    Ok(map) => util::thread_run(map).await,
                    Err(e) => log_error!("{}", e),
                };
            }
            false => subcommand(cli),
        },
    }
}

fn subcommand(cli: util::Cli) {
    match cli.sub {
        Some(util::Config::Config { reload, add, del, list, symlink }) => match reload {
            true => cookie::live::reload(cli.file.clone()),
            false => match list {
                true => cookie::live::list(cli.file.clone()),
                false => match add {
                    Some(data) => cookie::live::add(cli.file, data),
                    None => match del {
                        Some(data) => cookie::live::del(cli.file, data),
                        None => match symlink {
                            Some(data) => cookie::live::symlink(cli.file, data),
                            None => std::process::exit(0),
                        },
                    },
                },
            },
        },
        None => exit(0),
    }
}
