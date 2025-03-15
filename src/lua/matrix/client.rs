use super::room::Room;
use matrix_sdk::{
    Client as MatrixClient,
    ruma::{RoomId, UserId},
};
use mlua::{Error, UserData, UserDataFields, UserDataMethods};
use std::sync::Arc;

pub struct Client(pub Arc<MatrixClient>);

impl UserData for Client {
    fn add_fields<F: UserDataFields<Self>>(f: &mut F) {
        f.add_field_method_get("invited_rooms", |_, this| {
            Ok(this
                .0
                .invited_rooms()
                .into_iter()
                .map(Room)
                .collect::<Vec<_>>())
        });
        f.add_field_method_get("joined_rooms", |_, this| {
            Ok(this
                .0
                .joined_rooms()
                .into_iter()
                .map(Room)
                .collect::<Vec<_>>())
        });
        f.add_field_method_get("left_rooms", |_, this| {
            Ok(this
                .0
                .left_rooms()
                .into_iter()
                .map(Room)
                .collect::<Vec<_>>())
        });
        f.add_field_method_get("rooms", |_, this| {
            Ok(this.0.rooms().into_iter().map(Room).collect::<Vec<_>>())
        });
        f.add_field_method_get("user_id", |_, this| {
            Ok(this.0.user_id().map(ToString::to_string))
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        m.add_async_method("create_dm", async |_, this, user_id: String| {
            this.0
                .create_dm(&UserId::parse(user_id).map_err(Error::external)?)
                .await
                .map_err(Error::external)
                .map(Room)
        });
        m.add_async_method("join_room_by_id", async |_, this, room_id: String| {
            this.0
                .join_room_by_id(&RoomId::parse(room_id).map_err(Error::external)?)
                .await
                .map_err(Error::external)
                .map(Room)
        });
    }
}
