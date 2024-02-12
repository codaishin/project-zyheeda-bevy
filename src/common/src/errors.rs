#[derive(Debug, PartialEq, Clone)]
pub enum Level {
	Warning,
	Error,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Error {
	pub msg: String,
	pub lvl: Level,
}
