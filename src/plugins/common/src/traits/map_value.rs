pub trait MapForward<TFrom, TTo> {
	fn map_forward(&self, value: TFrom) -> TTo;
}

pub trait TryMapBackwards<TFrom, TTo> {
	fn try_map_backwards(&self, value: TFrom) -> Option<TTo>;
}
