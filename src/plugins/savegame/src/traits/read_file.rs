pub(crate) trait ReadFile {
	type TReadError;

	fn read(&self) -> Result<String, Self::TReadError>;
}
