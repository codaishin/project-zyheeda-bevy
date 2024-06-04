pub trait TryInsert<TKey, TValue> {
	type Error;

	fn try_insert(&mut self, key: TKey, value: TValue) -> Result<(), Self::Error>;
}
