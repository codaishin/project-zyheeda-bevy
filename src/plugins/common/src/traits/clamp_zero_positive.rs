pub trait ClampZeroPositive {
	fn new(value: f32) -> Self;
	fn value(&self) -> f32;
}
