pub mod localized;

use bevy::prelude::*;
use localized::Localized;
use std::{fmt::Display, ops::Deref};
use unic_langid::LanguageIdentifier;

pub trait HandlesLocalization {
	type TLocalizationServer: Resource + SetLocalization;
}

pub trait SetLocalization {
	fn set_localization(&mut self, language: LanguageIdentifier);
}

pub trait LocalizeToken {
	fn localize_token<'a, TToken>(&mut self, token: TToken) -> LocalizationResult<'a>
	where
		TToken: Into<Token<'a>>;
}

#[derive(Debug, PartialEq)]
pub struct Token<'a>(pub &'a str);

impl<'a> Token<'a> {
	pub fn failed(self) -> FailedToken<'a> {
		FailedToken(self)
	}
}

impl Display for Token<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "token: {}", self.0)
	}
}

impl<'a> From<&'a str> for Token<'a> {
	fn from(value: &'a str) -> Self {
		Token(value)
	}
}

impl<'a> From<&'a String> for Token<'a> {
	fn from(value: &'a String) -> Self {
		Token(value)
	}
}

#[derive(Debug, PartialEq)]
pub struct FailedToken<'a>(pub Token<'a>);

impl<'a> Deref for FailedToken<'a> {
	type Target = &'a str;

	fn deref(&self) -> &Self::Target {
		&self.0.0
	}
}

#[derive(Debug, PartialEq)]
pub enum LocalizationResult<'a> {
	Ok(Localized),
	Error(FailedToken<'a>),
}

impl LocalizationResult<'_> {
	pub fn or<F, T>(self, fallback: F) -> Localized
	where
		F: Fn(FailedToken) -> T,
		T: Into<String>,
	{
		match self {
			Self::Ok(string) => string,
			Self::Error(failed_token) => Localized::from_string(fallback(failed_token)),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn localize_result_string() {
		let result = LocalizationResult::Ok(Localized::from("my string"));

		assert_eq!(
			Localized::from("my string"),
			result.or(|failed_token| format!("FAILED: {}", *failed_token))
		)
	}

	#[test]
	fn localize_result_token() {
		let result = LocalizationResult::Error(FailedToken(Token("my string")));

		assert_eq!(
			Localized::from("FAILED: my string"),
			result.or(|failed_token| format!("FAILED: {}", *failed_token))
		)
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LanguageFallback {
	Default,
	None,
}
