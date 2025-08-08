pub trait Mapper<TFrom, TTo> {
	fn map(&self, value: TFrom) -> TTo;
}
