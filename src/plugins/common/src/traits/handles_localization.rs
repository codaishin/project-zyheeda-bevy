use bevy::prelude::*;
use std::ops::Deref;
use unic_langid::LanguageIdentifier;

pub trait HandlesLocalization {
	type TLocalizationServer: Resource + SetLocalization;
}

pub trait SetLocalization {
	fn set_localization(&mut self, language: LanguageIdentifier);
}

pub trait LocalizeToken {
	fn localize_token<'a>(&mut self, token: Token<'a>) -> LocalizationResult<'a>;
}

pub type Token<'a> = &'a str;

#[derive(Debug, PartialEq)]
pub struct FailedToken<'a>(pub Token<'a>);

impl<'a> Deref for FailedToken<'a> {
	type Target = Token<'a>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[derive(Debug, PartialEq)]
pub enum LocalizationResult<'a> {
	Value(String),
	Error(FailedToken<'a>),
}

impl LocalizationResult<'_> {
	pub fn or<F, T>(self, fallback: F) -> String
	where
		F: Fn(FailedToken) -> T,
		T: Into<String>,
	{
		match self {
			Self::Value(string) => string,
			Self::Error(failed_token) => fallback(failed_token).into(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn localize_result_string() {
		let result = LocalizationResult::Value(String::from("my string"));

		assert_eq!(
			String::from("my string"),
			result.or(|FailedToken(token)| format!("FAILED: {token}"))
		)
	}

	#[test]
	fn localize_result_token() {
		let result = LocalizationResult::Error(FailedToken("my string"));

		assert_eq!(
			String::from("FAILED: my string"),
			result.or(|FailedToken(token)| format!("FAILED: {token}"))
		)
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LanguageFallback {
	Default,
	None,
}
