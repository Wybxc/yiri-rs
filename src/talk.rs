use anyhow::Result;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Response {
    result: String,
    score: f64,
}

pub async fn get_reponse(talk_server: &str, message: &str) -> Result<String> {
    let accept = std::env::var("YIRI_ACCEPT")
        .unwrap_or("-100".to_string())
        .parse::<f64>()
        .unwrap_or(-100.0);

    let response = reqwest::get(format!("{talk_server}?msg={message}"))
        .await?
        .text()
        .await?;
    let response: Response = serde_json::from_str(&response)?;
    Ok(if response.score > accept {
        response.result
    } else if rand::random::<f64>() > 0.5 {
        "?".to_string()
    } else {
        message.to_string()
    })
}
