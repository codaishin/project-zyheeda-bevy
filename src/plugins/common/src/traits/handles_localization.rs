pub mod localized;

mod key_code;
mod mouse_button;

use bevy::prelude::*;
use localized::Localized;
use std::{fmt::Display, ops::Deref, sync::Arc};
use unic_langid::LanguageIdentifier;

pub trait HandlesLocalization {
	type TLocalizationServer: Resource + SetLocalization + Localize;
}

pub trait SetLocalization {
	fn set_localization(&mut self, language: LanguageIdentifier);
}

pub trait Localize {
	fn localize(&self, token: &Token) -> LocalizationResult;
}

pub trait LocalizeToken {
	fn localize_token<TToken>(&self, token: TToken) -> LocalizationResult
	where
		TToken: Into<Token>;
}

impl<T> LocalizeToken for T
where
	T: Localize,
{
	fn localize_token<TToken>(&self, token: TToken) -> LocalizationResult
	where
		TToken: Into<Token>,
	{
		self.localize(&token.into())
	}
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct Token(Arc<str>);

impl Token {
	pub fn failed(&self) -> FailedToken {
		FailedToken(self.0.clone())
	}
}

impl Display for Token {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "token: {}", self.0)
	}
}

impl From<&str> for Token {
	fn from(value: &str) -> Self {
		Token(Arc::from(value))
	}
}

impl From<String> for Token {
	fn from(value: String) -> Self {
		Token(Arc::from(value))
	}
}

impl Deref for Token {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.0.as_ref()
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct FailedToken(Arc<str>);

impl Deref for FailedToken {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		&self.0
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
			Self::Error(failed_token) => Localized::from(fallback(failed_token).into()),
		}
	}

	pub fn or_token(self) -> Localized {
		match self {
			Self::Ok(string) => string,
			Self::Error(FailedToken(t)) => Localized(t),
		}
	}

	pub fn or_string<F, T>(self, string_fn: F) -> Localized
	where
		F: Fn() -> T,
		T: Into<String>,
	{
		match self {
			Self::Ok(string) => string,
			Self::Error(_) => Localized::from(string_fn()),
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
			result.or(|failed_token| format!("FAILED: {}", &*failed_token))
		)
	}

	#[test]
	fn localize_result_or_err() {
		let result = LocalizationResult::Error(FailedToken(Arc::from("my token")));

		assert_eq!(
			Localized::from("FAILED: my token"),
			result.or(|failed_token| format!("FAILED: {}", &*failed_token))
		)
	}

	#[test]
	fn localize_result_or_token_ok() {
		let result = LocalizationResult::Ok(Localized::from("my string"));

		assert_eq!(Localized::from("my string"), result.or_token())
	}

	#[test]
	fn localize_result_or_token_err() {
		let result = LocalizationResult::Error(FailedToken(Arc::from("my token")));

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
		let result = LocalizationResult::Error(FailedToken(Arc::from("my token")));

		assert_eq!(
			Localized::from("my fallback"),
			result.or_string(|| "my fallback")
		)
	}
}
