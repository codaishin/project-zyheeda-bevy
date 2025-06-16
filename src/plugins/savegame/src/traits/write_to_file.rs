pub(crate) trait WriteToFile {
	type TError;

	fn write(&self, string: &str) -> Result<(), Self::TError>;
}
