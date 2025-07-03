pub(crate) trait WriteFile {
	type TWriteError;

	fn write(&self, string: &str) -> Result<(), Self::TWriteError>;
}
