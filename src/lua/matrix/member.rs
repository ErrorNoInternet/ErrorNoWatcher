use matrix_sdk::room::RoomMember;
use mlua::UserData;

pub struct Member(pub RoomMember);

impl UserData for Member {
    fn add_fields<F: mlua::UserDataFields<Self>>(f: &mut F) {
        f.add_field_method_get("id", |_, this| Ok(this.0.user_id().to_string()));
        f.add_field_method_get("name", |_, this| Ok(this.0.name().to_owned()));
        f.add_field_method_get("power_level", |_, this| Ok(this.0.power_level()));
    }
}
