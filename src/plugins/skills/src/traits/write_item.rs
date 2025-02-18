pub trait WriteItem<TKey, TValue> {
	fn write_item(&mut self, key: &TKey, value: TValue);
}
