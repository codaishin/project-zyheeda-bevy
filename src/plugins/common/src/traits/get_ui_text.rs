pub mod key_code;

pub struct English;

pub struct Japanese;

#[derive(Debug, PartialEq)]
pub enum UIText {
	String(String),
	Unmapped,
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
	fn ui_text_for(&self, value: &TValue) -> UIText
	where
		Japanese: GetUiText<TValue>,
		English: GetUiText<TValue>;
}
