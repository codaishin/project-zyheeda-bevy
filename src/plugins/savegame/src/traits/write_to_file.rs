pub(crate) trait WriteToFile {
	type TError;

	fn write(&self, string: String) -> Result<(), Self::TError>;
}
