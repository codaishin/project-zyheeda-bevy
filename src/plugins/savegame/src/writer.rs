use crate::traits::write_to_file::WriteToFile;

#[derive(Debug, PartialEq)]
pub struct FileWriter;

impl WriteToFile for FileWriter {
	fn write(&self, _: String) {}
}
