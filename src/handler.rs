use async_trait::async_trait;
use ricq::{
    client::event::{FriendMessageEvent, GroupMessageEvent, NewFriendRequestEvent},
    handler::{Handler, QEvent},
    msg::{elem::RQElem, MessageChainBuilder},
};

pub struct YiriHandler {
    pub uin: i64,
    pub talk_server: String,
}

#[async_trait]
impl Handler for YiriHandler {
    async fn handle(&self, event: QEvent) {
        tracing::debug!("收到事件：{:?}", event);

        match event {
            QEvent::GroupMessage(GroupMessageEvent { client, inner }) => {
                let mut reply = rand::random::<f64>() > 0.96;
                let mut at_me = false;

                let quote = inner.elements.reply();

                let message = inner
                    .elements
                    .into_iter()
                    .filter_map(|e| {
                        if let RQElem::At(ref at) = e {
                            if at.target == self.uin {
                                reply = true;
                                at_me = true;
                            }
                        }
                        if let RQElem::Text(t) = e {
                            Some(t.to_string())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<String>>();
                let message = message.join("").trim().to_string();

                tracing::info!(
                    "Group {}[{}] -> {}",
                    inner.group_name,
                    inner.group_code,
                    message
                );

                if !reply {
                    return;
                }

                if message.is_empty() && at_me {
                    let mut builder = MessageChainBuilder::new();
                    builder.push_str("你在叫我吗？");
                    let mut response = builder.build();

                    if let Some(reply) = quote {
                        response.with_reply(reply);
                    }

                    if let Err(e) = client.send_group_message(inner.group_code, response).await {
                        tracing::error!("发送消息失败：{}", e);
                    }
                    return;
                }

                let response = match crate::talk::get_reponse(&self.talk_server, &message).await {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::error!("获取回复失败：{}", e);
                        return;
                    }
                };

                tracing::info!(
                    "Group {}[{}] <- {}",
                    inner.group_name,
                    inner.group_code,
                    response
                );

                let mut builder = MessageChainBuilder::new();
                builder.push_str(&response);
                let response = builder.build();

                if let Err(e) = client.send_group_message(inner.group_code, response).await {
                    tracing::error!("发送消息失败：{}", e);
                }
            }
            QEvent::FriendMessage(FriendMessageEvent { client, inner }) => {
                let message = inner
                    .elements
                    .into_iter()
                    .filter_map(|e| {
                        if let RQElem::Text(t) = e {
                            Some(t.to_string())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<String>>();
                let message = message.join("");

                tracing::info!(
                    "Friend {}[{}] <- {}",
                    inner.from_nick,
                    inner.from_uin,
                    message
                );

                let response = match crate::talk::get_reponse(&self.talk_server, &message).await {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::error!("获取回复失败：{}", e);
                        return;
                    }
                };

                tracing::info!(
                    "Friend {}[{}] <- {}",
                    inner.from_nick,
                    inner.from_uin,
                    response
                );

                let mut builder = MessageChainBuilder::new();
                builder.push_str(&response);
                let response = builder.build();

                if let Err(e) = client.send_friend_message(inner.from_uin, response).await {
                    tracing::error!("发送消息失败：{}", e);
                }
            }
            QEvent::NewFriendRequest(NewFriendRequestEvent { client, inner }) => {
                if let Err(e) = client
                    .solve_friend_system_message(inner.msg_seq, inner.req_uin, true)
                    .await
                {
                    tracing::error!("处理好友请求失败：{}", e);
                }
            }
            _ => {}
        }
    }
}
