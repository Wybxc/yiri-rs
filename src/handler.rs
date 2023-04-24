use chocho::prelude::*;

use async_trait::async_trait;
use chocho::ricq::{
    client::event::{FriendMessageEvent, GroupMessageEvent, NewFriendRequestEvent},
    handler::PartlyHandler,
};
use chocho_msg::msg;

pub struct YiriHandler {
    pub talk_server: String,
}

impl YiriHandler {
    pub fn new() -> Self {
        let talk_server =
            std::env::var("YIRI_TALK_SERVER").unwrap_or("http://localhost:6000".to_string());
        Self { talk_server }
    }
}

#[async_trait]
impl PartlyHandler for YiriHandler {
    async fn handle_group_message(&self, GroupMessageEvent { client, inner }: GroupMessageEvent) {
        tracing::debug!("收到事件：{:?}", inner);

        let uin = client.uin().await;
        let group_name = inner.group_name;
        let group_code = inner.group_code;

        let mut reply = rand::random::<f64>() > 0.96;
        let mut at_me = false;

        let message: Message = inner.elements.into();
        let mut request = String::new();
        for elem in message.into_elems() {
            if let RQElem::At(ref at) = elem && at.target == uin {
                reply = true;
                at_me = true;
            }
            if let RQElem::Text(t) = elem {
                request.push_str(&t.content)
            }
        }
        let request = request.trim().to_string();

        tracing::info!("Group {}[{}] -> {}", group_name, group_code, request);

        if !reply {
            return;
        }

        if request.is_empty() && at_me {
            if let Err(e) = client.group(group_code).send(msg!["你在叫我吗？"]).await {
                tracing::error!("发送消息失败：{}", e);
            }
            return;
        }

        let response = match crate::talk::get_reponse(&self.talk_server, &request).await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("获取回复失败：{}", e);
                return;
            }
        };

        tracing::info!("Group {}[{}] <- {}", group_name, group_code, response);

        if let Err(e) = client.group(group_code).send(response).await {
            tracing::error!("发送消息失败：{}", e);
        }
    }

    async fn handle_friend_message(
        &self,
        FriendMessageEvent { client, inner }: FriendMessageEvent,
    ) {
        tracing::debug!("收到事件：{:?}", inner);

        let from_nick = inner.from_nick;
        let from_uin = inner.from_uin;

        let message: Message = inner.elements.into();
        let mut request = String::new();
        for elem in message.into_elems() {
            if let RQElem::Text(t) = elem {
                request.push_str(&t.content);
            }
        }

        tracing::info!("Friend {}[{}] <- {}", from_nick, from_uin, request);

        let response = match crate::talk::get_reponse(&self.talk_server, &request).await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("获取回复失败：{}", e);
                return;
            }
        };

        tracing::info!("Friend {}[{}] <- {}", from_nick, from_uin, response);

        if let Err(e) = client.friend(from_uin).send(response).await {
            tracing::error!("发送消息失败：{}", e);
        }
    }

    async fn handle_friend_request(
        &self,
        NewFriendRequestEvent { client, inner }: NewFriendRequestEvent,
    ) {
        tracing::debug!("收到事件：{:?}", inner);

        if let Err(e) = client
            .solve_friend_system_message(inner.msg_seq, inner.req_uin, true)
            .await
        {
            tracing::error!("处理好友请求失败：{}", e);
        }
    }
}
