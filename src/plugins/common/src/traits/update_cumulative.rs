pub trait CumulativeUpdate<TValue> {
	fn update_cumulative(&mut self, value: TValue);
}
