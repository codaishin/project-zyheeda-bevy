pub mod key_code;

pub struct English;

pub struct Japanese;

#[derive(Debug, PartialEq, Clone)]
pub enum UIText {
	String(String),
	Unmapped,
}

impl UIText {
	pub fn ok(self) -> Option<String> {
		match self {
			UIText::String(value) => Some(value),
			_ => None,
		}
	}
}

impl From<&'static str> for UIText {
	fn from(value: &'static str) -> Self {
		UIText::String(value.into())
	}
}

pub trait GetUiText<TValue>
where
	Self: Sized,
{
	fn ui_text(value: &TValue) -> UIText;
}

pub trait GetUiTextFor<TValue> {
	fn ui_text_for(&self, value: &TValue) -> UIText;
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn ui_text_ok() {
		let text = UIText::String("my string".to_owned());

		assert_eq!(Some("my string".to_owned()), text.ok());
	}

	#[test]
	fn ui_text_not_ok() {
		let text = UIText::Unmapped;

		assert_eq!(None, text.ok());
	}
}
