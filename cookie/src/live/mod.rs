
use regex::Regex;
use std::{io::{Read, Write},process::exit};
pub mod douyin;
use dotenv_rs::get_vars;


pub fn add(file:String,data:String) {
    let context = open_file(file.clone());
    // 切割需要修改的字符串
    let data_split = data.split_once("=").unwrap();
    // 判断需要修改的字符串是否包含特殊字符或空格
    let data = if data.contains(" ") || data.contains("&") {
        format!(r#"{} = "{}""#,data_split.0,data_split.1)
    } else {
        format!(r#"{} = {}"#,data_split.0,data_split.1)
    };
    create_file(data_split.0, context, data, file)
}

pub fn del(file:String,key:String) {
    let context = open_file(file.clone());
    create_file(&key, context, "".to_string(), file)
}

pub fn list(path:String) {
    let maps = get_vars(path);
    match maps {
        Ok(map) => for (key,value) in map.iter() {
            println!("{} = {}",key,value.clone().unwrap())
        } ,
        Err(err) => {
            println!("{}",err.to_string());
            exit(0)
        },
    }
}

#[allow(deprecated)]
pub fn symlink(file:String,data:String) {
    std::fs::soft_link(file, data).unwrap();
}

pub fn reload(file: String) {
    let config_txt: Vec<u8> = vec![
        10, 10, 68, 79, 85, 89, 85, 95, 85, 82, 76, 95, 95, 49, 32, 61, 32, 120, 120, 120, 120,
        120, 120, 10, 68, 79, 85, 89, 85, 95, 85, 82, 76, 95, 95, 50, 32, 61, 32, 120, 120, 120,
        120, 120, 120, 10, 68, 79, 85, 89, 73, 78, 95, 85, 82, 76, 95, 95, 49, 32, 61, 32, 120,
        120, 120, 120, 120, 120, 10, 68, 79, 85, 89, 73, 78, 95, 85, 82, 76, 95, 95, 50, 32, 61,
        32, 120, 120, 120, 120, 120, 120, 10, 68, 79, 85, 89, 73, 78, 95, 85, 82, 76, 95, 95, 51,
        32, 61, 32, 120, 120, 120, 120, 120, 120, 10, 10, 10, 10, 99, 111, 111, 107, 105, 101, 95,
        100, 111, 117, 121, 105, 110, 32, 61, 32, 34, 120, 120, 120, 120, 120, 120, 34, 10, 10,
    ];
    let file_rs = match file.rsplit_once("\\"){
        Some(file) => file,
        None => file.rsplit_once("/").unwrap(),
    };
    let _ = std::fs::create_dir_all(file_rs.0);
    let mut file = std::fs::File::create(file).unwrap();
    let _ = file.write_all(&config_txt.clone());
}

fn open_file(file:String) -> String {
    // 创建空字符串
    let mut context = String::new();
    // 打开文件
    let open_file = std::fs::File::open(file);
    match open_file {
        Ok(mut open_file) => {
            // 文件内容赋值给context
            open_file.read_to_string(&mut context).unwrap();
        },
        Err(err) => {
            println!("{}",err.to_string());
            exit(0)
        },
    }
    return context;
}


/// ## Parse
/// *re 需要匹配的key*
/// 
/// *context 正则匹配的文本*
/// 
/// *data 需要替换的字符串*
/// 
/// *file 文件的路径*
fn create_file(re:&str,mut context:String,data:String,file:String) {
    // 通过re替换字符串
    let result = Regex::new(&format!("{}(.*?)\n",re))
        .unwrap()
        .replace_all(&context, &format!("{}\n",data));
    match result {
        std::borrow::Cow::Borrowed(text) => context = format!("{}{}",text,data),
        std::borrow::Cow::Owned(text) => context = format!("{}",text),
    }
    // 创建一个新的文本
    let create_file = std::fs::File::create(file);
    match create_file {
        Ok(mut create_file) => create_file.write_all(context.as_bytes()).unwrap(),
        Err(_) => todo!(),
    }
    
}