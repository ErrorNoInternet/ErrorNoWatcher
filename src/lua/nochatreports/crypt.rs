#[macro_export]
macro_rules! crypt {
    ($op:ident, $options:expr, $text:expr) => {{
        macro_rules! crypt_with {
            ($algo:ident) => {{
                let encoding = $options.get("encoding").unwrap_or_default();
                let key = &$options.get::<UserDataRef<AesKey>>("key")?.0;
                match encoding {
                    1 => $algo::<Base64Encoding>::$op($text, &key),
                    2 => $algo::<Base64rEncoding>::$op($text, &key),
                    _ => $algo::<NewBase64rEncoding>::$op($text, &key),
                }
                .map_err(|error| Error::external(error.to_string()))?
            }};
        }

        match $options.get("encryption").unwrap_or_default() {
            1 => CaesarEncryption::$op(&$text, &$options.get("key")?)
                .map_err(|error| Error::external(error.to_string()))?,
            2 => crypt_with!(EcbEncryption),
            3 => crypt_with!(GcmEncryption),
            _ => crypt_with!(Cfb8Encryption),
        }
    }};
}
