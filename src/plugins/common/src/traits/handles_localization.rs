pub mod localized;

mod key_code;
mod mouse_button;

use crate::traits::{accessors::get::ViewField, thread_safe::ThreadSafe};
use bevy::{ecs::system::SystemParam, prelude::*};
use localized::Localized;
use macros::serde_model;
use std::{fmt::Display, ops::Deref, sync::Arc};
use unic_langid::LanguageIdentifier;

pub trait HandlesLocalization {
	type TLocalizationServer: ThreadSafe + for<'w, 's> SystemParam<Item<'w, 's>: Localize>;
	type TLocalizationServerMut: ThreadSafe + for<'w, 's> SystemParam<Item<'w, 's>: SetLocalization>;
}

pub trait SetLocalization {
	fn set_localization(&mut self, language: LanguageIdentifier);
}

pub trait Localize {
	fn localize(&self, token: &Token) -> LocalizationResult;
}

impl<T> Localize for T
where
	T: Deref<Target: Localize>,
{
	fn localize(&self, token: &Token) -> LocalizationResult {
		self.deref().localize(token)
	}
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
pub struct Token(Vec<TokenItem<Arc<str>>>);

impl Token {
	pub fn failed(&self) -> FailedToken {
		FailedToken(self.0.clone())
	}

	pub fn iter(&self) -> TokenIterator<'_> {
		TokenIterator { it: self.0.iter() }
	}

	pub fn items(&self) -> TokenIterator<'_> {
		self.iter()
	}
}

impl Display for Token {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for item in self {
			write!(f, "{item}")?
		}

		Ok(())
	}
}

impl From<&str> for Token {
	fn from(value: &str) -> Self {
		Token(vec![TokenItem::Key(Arc::from(value))])
	}
}

impl From<String> for Token {
	fn from(value: String) -> Self {
		Token(vec![TokenItem::Key(Arc::from(value))])
	}
}

impl<const N: usize, T> From<[TokenItem<T>; N]> for Token
where
	T: Deref<Target = str>,
{
	fn from(values: [TokenItem<T>; N]) -> Self {
		Token(
			values
				.into_iter()
				.map(|item| TokenItem::from_item(item.as_deref()))
				.collect(),
		)
	}
}

impl<S> FromIterator<TokenItem<S>> for Token
where
	S: Deref<Target = str>,
{
	fn from_iter<T: IntoIterator<Item = TokenItem<S>>>(iter: T) -> Self {
		Self(
			iter.into_iter()
				.map(|item| TokenItem::<Arc<str>>::from_item(item.as_deref()))
				.collect(),
		)
	}
}

impl ViewField for Token {
	type TValue<'a> = &'a Self;
}

impl<'a> IntoIterator for &'a Token {
	type Item = TokenItem<&'a str>;
	type IntoIter = TokenIterator<'a>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

pub struct TokenIterator<'a> {
	it: std::slice::Iter<'a, TokenItem<Arc<str>>>,
}

impl<'a> Iterator for TokenIterator<'a> {
	type Item = TokenItem<&'a str>;

	fn next(&mut self) -> Option<Self::Item> {
		self.it.next().map(|item| match item {
			TokenItem::Key(key) => TokenItem::Key(key.deref()),
			TokenItem::Raw(raw) => TokenItem::Raw(raw.deref()),
		})
	}
}

#[serde_model]
#[derive(Debug, PartialEq, Clone)]
pub enum TokenItem<T> {
	Key(T),
	Raw(T),
}

impl<T> TokenItem<T> {
	pub fn from_item<U>(value: TokenItem<U>) -> Self
	where
		T: From<U>,
	{
		match value {
			TokenItem::Key(key) => Self::Key(T::from(key)),
			TokenItem::Raw(raw) => Self::Raw(T::from(raw)),
		}
	}
}

impl<T> TokenItem<T>
where
	T: Deref<Target = str>,
{
	fn as_deref(&self) -> TokenItem<&str> {
		match self {
			TokenItem::Key(key) => TokenItem::Key(key.deref()),
			TokenItem::Raw(raw) => TokenItem::Raw(raw.deref()),
		}
	}
}

impl<T> Display for TokenItem<T>
where
	T: Display,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			TokenItem::Key(key) => write!(f, "Token({key})"),
			TokenItem::Raw(raw) => write!(f, "{raw}"),
		}
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct FailedToken(Vec<TokenItem<Arc<str>>>);

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
			Self::Error(FailedToken(t)) => Localized(Arc::from(Token(t).to_string())),
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

impl From<Localized> for LocalizationResult {
	fn from(localized: Localized) -> Self {
		Self::Ok(localized)
	}
}

impl From<FailedToken> for LocalizationResult {
	fn from(token: FailedToken) -> Self {
		Self::Error(token)
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
			result.or(|failed| format!("FAILED: {failed:?}"))
		)
	}

	#[test]
	fn localize_result_or_err() {
		let failed = FailedToken(vec![TokenItem::Key(Arc::from("my token"))]);
		let result = LocalizationResult::Error(failed.clone());

		assert_eq!(
			Localized::from(format!("FAILED: {failed:?}")),
			result.or(|failed| format!("FAILED: {failed:?}"))
		)
	}

	#[test]
	fn localize_result_or_token_ok() {
		let result = LocalizationResult::Ok(Localized::from("my string"));

		assert_eq!(Localized::from("my string"), result.or_token())
	}

	#[test]
	fn localize_result_or_token_err() {
		let result =
			LocalizationResult::Error(FailedToken(vec![TokenItem::Key(Arc::from("my token"))]));

		assert_eq!(
			Localized::from(Token::from("my token").to_string()),
			result.or_token()
		)
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
		let result =
			LocalizationResult::Error(FailedToken(vec![TokenItem::Key(Arc::from("my token"))]));

		assert_eq!(
			Localized::from("my fallback"),
			result.or_string(|| "my fallback")
		)
	}

	#[test]
	fn iterate_token() {
		let token = Token::from([
			TokenItem::Key("a"),
			TokenItem::Raw(", "),
			TokenItem::Key("b"),
		]);

		let items = token.iter().collect::<Vec<_>>();

		assert_eq!(
			vec![
				TokenItem::Key("a"),
				TokenItem::Raw(", "),
				TokenItem::Key("b")
			],
			items
		);
	}
}
