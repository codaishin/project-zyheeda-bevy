pub(crate) trait KeyString<TKey> {
	fn key_string(key: &TKey) -> &'static str;
}
