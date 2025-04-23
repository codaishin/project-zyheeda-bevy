pub(crate) trait GetErrorsMut {
	type TError;

	fn errors_mut(&mut self) -> &mut Vec<Self::TError>;
}
