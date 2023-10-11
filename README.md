<div style="width: 100%;height: 300px;">
    <img src="./.github/image.jpg" style="width: 100%;object-fit:cover">
</div>


<div align="center">
    <hr/>
    <a href="https://github.com/soft-cute/test">
        <img src="https://img.shields.io/badge/github-soft--cute%2Ftest-1707320?logo=github">
    </a>
    <img src="https://img.shields.io/github/last-commit/soft-cute/test/master?logo=github">
    <a href="https://github.com/soft-cute/test/releases">
        <img src="https://img.shields.io/github/v/release/soft-cute/test?logo=github">
    </a>
    <img src="https://img.shields.io/github/release-date/soft-cute/test">
    <img src="https://img.shields.io/github/license/soft-cute/test">
</div><br><br>

# 目录
- [目录](#目录)
- [说明](#说明)
    - [命令行使用方法](#命令行使用方法)
    - [是否开启录制或播放](#是否开启录制或播放)
    - [使用配置文件](#使用配置文件)
- [注意](#注意)

# 说明

```bash
____________________________________________________

                        _
          ___  __   __ (_)  _ __     __ _
         / _ \ \ \ / / | | | '_ \   / _` |
        |  __/  \ V /  | | | | | | | (_| |
         \___|   \_/   |_| |_| |_|  \__,_|

____________________________________________________


Usage: evina.exe [OPTIONS]
       evina.exe [OPTIONS] <COMMAND>

Commands:
  config  编辑配置文件
  help    Print this message or the help of the given subcommand(s)

Options:
  -l, --live <LIVE>  需要录制的平台 [possible values: douyu, douyin]
  -i, --id <ID>      直播间号或链接
  -x <X>             播放器的宽度 [default: 1366]
  -y <Y>             播放器的高度 [default: 768]
  -d <DOWNLOAD>      自定义录制的目录，默认保存到根目录下的download [default: /download]
  -m, --ffm          是否开启录播，默认不开启
  -p, --ffp          是否开启播放功能，默认不开启
  -r, --read         使用配置文件配置的id
  -f, --file <FILE>  自定义配置文件的路径 [default: C:\Users\softcute\.evina\config]
  -h, --help         Print help
  -V, --version      Print version

```



### 命令行使用方法
```shell
# 斗鱼平台仅支持输入房间ID
evina --live douyu --id 123456
或者
evina -l douyu -i 123456

# 抖音平台支持输入链接和房间ID
# 如果使用手机分享的短链接，必须配置抖音账号的 cookie
evina --live douyin --id https://v.douyin.com/abcdefg/
evina --live douyin --id abcdefg/
evina --live douyin --id abcdefg
evina --live douyin --id https://live.douyin.com/123456789
evina --live douyin --id 123456789
或者
evina -l douyin -i https://v.douyin.com/abcdefg/
evina -l douyin -i abcdefg/
evina -l douyin -i abcdefg
evina -l douyin -i https://live.douyin.com/123456789
evina -l douyin -i 123456789
```
### 是否开启录制或播放
```shell
# 开启录制功能
evina -l douyu -i 123456 -m
# 开启录制功能并设置保存目录
# 目录格式默认为 自定义目录/直播间名字/日期时间/录制的视频
evina -l douyu -i 123456 -m -d 自定义目录
# 开启播放功能
evina -l douyu -i 123456 -p
# 开启播放功能并设置分辨率大小
evina -l douyu -i 123456 -p -x 1080 -y 768
# 同时开启录制和播放功能
evina -l douyu -i 123456 -mp
```
### 使用配置文件
```shell
# 创建配置文件或者格式化
evina config -r
# 自定义配置文件的路径
evina -f /aa/aa.txt config -r
# 添加参数到文件
evina config -a DOUYU_URL__1=123456
# 删除文件的参数
evina config -d DOUYU_URL__1
# 创建配置文件的软连接
evina config -s aaa/config
# 列出配置文件所有的参数
evina config -l
```

# 注意
 1. 录播需要安装FFMPEG
 2. 在线播放需要安装FFPLAY


