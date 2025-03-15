use super::member::Member;
use matrix_sdk::{
    RoomMemberships,
    room::Room as MatrixRoom,
    ruma::{EventId, UserId, events::room::message::RoomMessageEventContent},
};
use mlua::{Error, UserData, UserDataFields, UserDataMethods};

pub struct Room(pub MatrixRoom);

impl UserData for Room {
    fn add_fields<F: UserDataFields<Self>>(f: &mut F) {
        f.add_field_method_get("id", |_, this| Ok(this.0.room_id().to_string()));
        f.add_field_method_get("name", |_, this| Ok(this.0.name()));
        f.add_field_method_get("topic", |_, this| Ok(this.0.topic()));
        f.add_field_method_get("type", |_, this| {
            Ok(this.0.room_type().map(|room_type| room_type.to_string()))
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        m.add_async_method(
            "ban_user",
            async |_, this, (user_id, reason): (String, Option<String>)| {
                this.0
                    .ban_user(
                        &UserId::parse(user_id).map_err(Error::external)?,
                        reason.as_deref(),
                    )
                    .await
                    .map_err(Error::external)
            },
        );
        m.add_async_method("get_members", async |_, this, (): ()| {
            this.0
                .members(RoomMemberships::all())
                .await
                .map_err(Error::external)
                .map(|members| {
                    members
                        .into_iter()
                        .map(|member| Member(member.clone()))
                        .collect::<Vec<_>>()
                })
        });
        m.add_async_method(
            "kick_user",
            async |_, this, (user_id, reason): (String, Option<String>)| {
                this.0
                    .kick_user(
                        &UserId::parse(user_id).map_err(Error::external)?,
                        reason.as_deref(),
                    )
                    .await
                    .map_err(Error::external)
            },
        );
        m.add_async_method("leave", async |_, this, (): ()| {
            this.0.leave().await.map_err(Error::external)
        });
        m.add_async_method(
            "redact",
            async |_, this, (event_id, reason): (String, Option<String>)| {
                this.0
                    .redact(
                        &EventId::parse(event_id).map_err(Error::external)?,
                        reason.as_deref(),
                        None,
                    )
                    .await
                    .map_err(Error::external)
                    .map(|response| response.event_id.to_string())
            },
        );
        m.add_async_method("send", async |_, this, body: String| {
            this.0
                .send(RoomMessageEventContent::text_plain(body))
                .await
                .map_err(Error::external)
                .map(|response| response.event_id.to_string())
        });
    }
}
