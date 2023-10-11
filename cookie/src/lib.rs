pub mod live;
use std::collections::HashMap;
pub fn local_cookie(path:String,key: String) -> Result<String, std::io::Error> {
    let cookie = live::douyin::local_cookie(path,&key);
    return cookie;
}

pub async fn auto_cookie(url: String) -> Result<String, Box<dyn std::error::Error>> {
    let cookie = live::douyin::auto_cookie(url).await;
    return cookie;
}

pub fn read_config(path:String,prefix_list: Vec<&str>) -> HashMap<String, Option<String>> {
    let mut map = HashMap::new();
    for platform in prefix_list {
        let hashmap = dotenv_rs::get_vars_with_prefix(path.clone(), platform);
        match hashmap {
            Ok(hashmap) => map.extend(hashmap),
            Err(err) => {
                println!("{}",err.to_string());
                std::process::exit(0)
            },
        }
    };
    return map;

}


#[cfg(test)]
mod tests {
    use std::env;

    #[test]
    fn main() {
        // 获取命令行参数
        let args: Vec<String> = env::args().collect();

        // 打印程序名称
        let program_name = &args[0];
        println!("Program name: {}", program_name);

        // 打印所有命令行参数
        println!("Command line arguments:");
        for arg in args.iter().skip(1) {
            println!("{}", arg);
        }

        // 检查命令行参数数量
        if args.len() < 3 {
            println!("Usage: {} <arg1> <arg2>", program_name);
        } else {
            // 获取命令行参数
            let arg1 = &args[1];
            let arg2 = &args[2];

            // 在这里处理命令行参数
            println!("Argument 1: {}", arg1);
            println!("Argument 2: {}", arg2);
        }
    }
}
