use mlua::{UserData, UserDataFields};

pub struct AesKey(pub ncr::AesKey);

impl UserData for AesKey {
    fn add_fields<F: UserDataFields<Self>>(f: &mut F) {
        f.add_field_method_get("base64", |_, this| Ok(this.0.encode_base64()));
        f.add_field_method_get("bytes", |_, this| Ok(this.0.as_ref().to_vec()));
    }
}
