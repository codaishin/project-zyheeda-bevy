pub mod localized;

mod key_code;

use bevy::prelude::*;
use localized::Localized;
use std::{fmt::Display, ops::Deref};
use unic_langid::LanguageIdentifier;

pub trait HandlesLocalization {
	type TLocalizationServer: Resource + SetLocalization + LocalizeToken;
}

pub trait SetLocalization {
	fn set_localization(&mut self, language: LanguageIdentifier);
}

pub trait LocalizeToken {
	fn localize_token<TToken>(&mut self, token: TToken) -> LocalizationResult
	where
		TToken: Into<Token> + 'static;
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token(pub String);

impl Token {
	pub fn failed(self) -> FailedToken {
		FailedToken(self)
	}
}

impl Display for Token {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "token: {}", self.0)
	}
}

impl From<&str> for Token {
	fn from(value: &str) -> Self {
		Token(value.to_owned())
	}
}

impl From<String> for Token {
	fn from(value: String) -> Self {
		Token(value)
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct FailedToken(pub Token);

impl Deref for FailedToken {
	type Target = String;

	fn deref(&self) -> &Self::Target {
		&self.0.0
	}
}

#[derive(Debug, PartialEq, Clone)]
pub enum LocalizationResult {
	Ok(Localized),
	Error(FailedToken),
}

impl LocalizationResult {
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

	pub fn or_token(self) -> Localized {
		match self {
			Self::Ok(string) => string,
			Self::Error(FailedToken(Token(t))) => Localized::from_string(t),
		}
	}

	pub fn or_string<F, T>(self, string_fn: F) -> Localized
	where
		F: Fn() -> T,
		T: Into<String>,
	{
		match self {
			Self::Ok(string) => string,
			Self::Error(_) => Localized::from_string(string_fn()),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn localize_result_or_ok() {
		let result = LocalizationResult::Ok(Localized::from("my string"));

		assert_eq!(
			Localized::from("my string"),
			result.or(|failed_token| format!("FAILED: {}", *failed_token))
		)
	}

	#[test]
	fn localize_result_or_err() {
		let result = LocalizationResult::Error(FailedToken(Token::from("my token")));

		assert_eq!(
			Localized::from("FAILED: my token"),
			result.or(|failed_token| format!("FAILED: {}", *failed_token))
		)
	}

	#[test]
	fn localize_result_or_token_ok() {
		let result = LocalizationResult::Ok(Localized::from("my string"));

		assert_eq!(Localized::from("my string"), result.or_token())
	}

	#[test]
	fn localize_result_or_token_err() {
		let result = LocalizationResult::Error(FailedToken(Token::from("my token")));

		assert_eq!(Localized::from("my token"), result.or_token())
	}

	#[test]
	fn localize_result_or_string_ok() {
		let result = LocalizationResult::Ok(Localized::from("my string"));

		assert_eq!(
			Localized::from("my string"),
			result.or_string(|| "my fallback")
		)
	}

	#[test]
	fn localize_result_or_string_err() {
		let result = LocalizationResult::Error(FailedToken(Token::from("my token")));

		assert_eq!(
			Localized::from("my fallback"),
			result.or_string(|| "my fallback")
		)
	}
}
