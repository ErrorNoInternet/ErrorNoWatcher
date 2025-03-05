use mlua::UserData;

pub struct AesKey {
    pub inner: ncr::AesKey,
}

impl UserData for AesKey {
    fn add_fields<F: mlua::UserDataFields<Self>>(f: &mut F) {
        f.add_field_method_get("base64", |_, this| Ok(this.inner.encode_base64()));
        f.add_field_method_get("bytes", |_, this| Ok(this.inner.as_ref().to_vec()));
    }
}
