pub(crate) trait ReadFile {
	type TError;

	fn read(&self) -> Result<String, Self::TError>;
}
