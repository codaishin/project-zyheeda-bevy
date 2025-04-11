pub trait GetKeyCode<TKey, TKeyCode> {
	fn get_key_code(&self, value: TKey) -> TKeyCode;
}

pub trait TryGetKey<TKeyCode, TKey> {
	fn try_get_key(&self, value: TKeyCode) -> Option<TKey>;
}
