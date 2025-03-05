macro_rules! crypt_with {
    ($op:ident, $encoding:expr, $key:expr, $text:expr, $algo:ident) => {
        match $encoding {
            1 => $algo::<Base64Encoding>::$op($text, $key),
            2 => $algo::<Base64rEncoding>::$op($text, $key),
            _ => $algo::<NewBase64rEncoding>::$op($text, $key),
        }
        .map_err(|error| Error::external(error.to_string()))?
    };
}

#[macro_export]
macro_rules! crypt {
    ($op:ident, $encoding:expr, $options:expr, $text:expr) => {
        match $options.get("encryption").unwrap_or_default() {
            1 => CaesarEncryption::$op(&$text, &$options.get("key")?)
                .map_err(|error| Error::external(error.to_string()))?,
            2 => crypt_with!(
                $op,
                $encoding,
                &$options.get::<UserDataRef<AesKey>>("key")?.inner,
                &$text,
                EcbEncryption
            ),
            3 => crypt_with!(
                $op,
                $encoding,
                &$options.get::<UserDataRef<AesKey>>("key")?.inner,
                &$text,
                GcmEncryption
            ),
            _ => crypt_with!(
                $op,
                $encoding,
                &$options.get::<UserDataRef<AesKey>>("key")?.inner,
                &$text,
                Cfb8Encryption
            ),
        }
    };
}
