pub trait SwapKeys<TKey1, TKey2> {
	fn swap(&mut self, a: TKey1, b: TKey2);
}
