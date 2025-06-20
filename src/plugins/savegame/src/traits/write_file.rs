pub(crate) trait WriteFile {
	type TError;

	fn write(&self, string: &str) -> Result<(), Self::TError>;
}
