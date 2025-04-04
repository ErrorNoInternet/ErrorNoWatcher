#[macro_use]
pub mod crypt;
pub mod key;

use key::AesKey;
use mlua::{Error, Lua, Result, Table, UserDataRef};
use ncr::{
    encoding::{Base64Encoding, Base64rEncoding, NewBase64rEncoding},
    encryption::{CaesarEncryption, Cfb8Encryption, EcbEncryption, Encryption, GcmEncryption},
    utils::{prepend_header, trim_header},
};

pub fn register_globals(lua: &Lua, globals: &Table) -> Result<()> {
    globals.set(
        "ncr_aes_key_from_passphrase",
        lua.create_function(|_, passphrase: Vec<u8>| {
            Ok(AesKey(ncr::AesKey::gen_from_passphrase(&passphrase)))
        })?,
    )?;

    globals.set(
        "ncr_aes_key_from_base64",
        lua.create_function(|_, base64: String| {
            Ok(AesKey(
                ncr::AesKey::decode_base64(&base64)
                    .map_err(|error| Error::external(error.to_string()))?,
            ))
        })?,
    )?;

    globals.set(
        "ncr_generate_random_aes_key",
        lua.create_function(|_, (): ()| Ok(AesKey(ncr::AesKey::gen_random_key())))?,
    )?;

    globals.set(
        "ncr_encrypt",
        lua.create_function(|_, (options, plaintext): (Table, String)| {
            Ok(crypt!(encrypt, options, &plaintext))
        })?,
    )?;

    globals.set(
        "ncr_decrypt",
        lua.create_function(|_, (options, ciphertext): (Table, String)| {
            Ok(crypt!(decrypt, options, &ciphertext))
        })?,
    )?;

    globals.set(
        "ncr_prepend_header",
        lua.create_function(|_, text: String| Ok(prepend_header(&text)))?,
    )?;

    globals.set(
        "ncr_trim_header",
        lua.create_function(|_, text: String| {
            Ok(trim_header(&text)
                .map_err(|error| Error::external(error.to_string()))?
                .to_owned())
        })?,
    )?;

    Ok(())
}
