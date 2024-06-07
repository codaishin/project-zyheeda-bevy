pub trait Cache<TKey, TValue> {
	fn cached(&mut self, key: TKey, new: impl FnOnce() -> TValue) -> TValue;
}
