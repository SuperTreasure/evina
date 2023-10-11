
use clap::Parser;
use live::{Cli, Config};
mod live;

fn main() {
    let cli = live::Cli::parse();
    match cli.live.clone() {
        Some(data) => {
            if data == "douyu" {
                live::Information::to_print(&live::douyu::douyu(cli.id.unwrap()).unwrap());
            } else if data == "douyin" {
                live::Information::to_print(&live::douyin::douyin(cli.id.unwrap()).unwrap());
            }
        }
        None => {
            if cli.read {
                let list = vec!["DOUYU","DOUYIN"];
                live::thread_run(cookie::read_config(cli.file.clone(), list));

            }
            else {
                subcommand(cli)
            }
        }
    }
}

fn subcommand(cli: Cli) {
    match cli.sub {
        Some(Config::Config { reload, add, del, list, symlink }) => {
            if reload == true {
                cookie::live::reload(cli.file.clone());
            }else if list == true {
                cookie::live::list(cli.file.clone())
                
            }
            match add {
                Some(data) => cookie::live::add(cli.file, data),
                None => match del {
                    Some(data) => cookie::live::del(cli.file, data),
                    None => match symlink {
                        Some(data) => cookie::live::symlink(cli.file,data),
                        None => std::process::exit(0),
                    },
                },
            }
        }
        None => std::process::exit(0),
    }
}
