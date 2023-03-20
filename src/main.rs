mod device;
mod handler;
mod login;
mod talk;
mod utils;

use anyhow::Result;
use requestty::Question;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let talk_server =
        std::env::var("YIRI_TALK_SERVER").unwrap_or("http://localhost:6000".to_string());
    let data_folder = std::env::var("YIRI_DATA_FOLDER").unwrap_or("./bots".to_string());

    let uin = Question::int("uin").message("请输入账号").build();
    let uin = requestty::prompt_one(uin)?.as_int().unwrap();

    let login_method = Question::select("login_method")
        .message("请选择登录方式：")
        .choice("密码登录")
        .choice("二维码登录")
        .build();
    let login_method = requestty::prompt_one(login_method)?
        .as_list_item()
        .unwrap()
        .index;

    let handler = handler::YiriHandler { uin, talk_server };

    let (_, alive) = match login_method {
        0 => {
            // 密码登录
            let protocol = Question::select("protocol")
                .message("请选择客户端协议：")
                .choice("IPad")
                .choice("Android Phone")
                .choice("Android Watch")
                .choice("MacOS")
                .choice("企点")
                .default(0)
                .build();
            let protocol = requestty::prompt_one(protocol)?
                .as_list_item()
                .unwrap()
                .index;
            let protocol = match protocol {
                0 => login::Protocol::IPad,
                1 => login::Protocol::AndroidPhone,
                2 => login::Protocol::AndroidWatch,
                3 => login::Protocol::MacOS,
                4 => login::Protocol::QiDian,
                _ => unreachable!(),
            };

            let password = Question::password("password")
                .message("请输入密码")
                .mask('*')
                .build();
            let password = requestty::prompt_one(password)?.try_into_string().unwrap();

            login::login_with_password(uin, &password, protocol, data_folder, handler).await?
        }
        1 => {
            // 二维码登录
            login::login_with_qrcode(
                uin,
                |img| {
                    println!("{}", utils::qrcode_text(&img)?);
                    Ok(())
                },
                data_folder,
                handler,
            )
            .await?
        }
        _ => unreachable!(),
    };

    alive.auto_reconnect().await?;
    Ok(())
}
