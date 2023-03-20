use anyhow::Result;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Response {
    result: String,
    score: f64,
}

pub async fn get_reponse(talk_server: &str, message: &str) -> Result<String> {
    let response = reqwest::get(format!("{talk_server}?msg={message}"))
        .await?
        .text()
        .await?;
    let response: Response = serde_json::from_str(&response)?;
    Ok(if response.score > -0.5 {
        response.result
    } else if rand::random::<f64>() > 0.5 {
        "?".to_string()
    } else {
        message.to_string()
    })
}
