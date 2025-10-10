pub(crate) trait DrainInvalidInputs {
	type TInvalidInput;

	fn drain_invalid_inputs(&mut self) -> impl Iterator<Item = Self::TInvalidInput>;
}
