use crate::{
	assets::ftl::Ftl,
	traits::{current_locale::CurrentLocaleMut, requested_language::UpdateCurrentLocaleMut},
};
use bevy::{asset::LoadedFolder, prelude::*};
use common::traits::handles_localization::SetLocalization;
use fluent::{FluentResource, concurrent::FluentBundle};
use unic_langid::LanguageIdentifier;

#[derive(Resource)]
pub struct FtlServer {
	fallback: Locale,
	current: Option<Locale>,
	update: bool,
}

pub(crate) struct Locale {
	pub(crate) ln: LanguageIdentifier,
	pub(crate) file: Option<Handle<Ftl>>,
	pub(crate) folder: Option<Handle<LoadedFolder>>,
	pub(crate) bundle: Option<FluentBundle<FluentResource>>,
}

impl From<LanguageIdentifier> for FtlServer {
	fn from(index: LanguageIdentifier) -> Self {
		Self {
			fallback: Locale {
				ln: index,
				file: None,
				folder: None,
				bundle: None,
			},
			current: None,
			update: true,
		}
	}
}

impl CurrentLocaleMut for FtlServer {
	fn current_locale_mut(&mut self) -> &mut Locale {
		self.current.as_mut().unwrap_or(&mut self.fallback)
	}
}

impl SetLocalization for FtlServer {
	fn set_localization(&mut self, language: LanguageIdentifier) {
		self.update = true;

		self.current = match language == self.fallback.ln {
			true => None,
			false => Some(Locale {
				ln: language,
				file: None,
				folder: None,
				bundle: None,
			}),
		};
	}
}

impl UpdateCurrentLocaleMut for FtlServer {
	fn update_current_locale(&mut self) -> &mut bool {
		&mut self.update
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use unic_langid::langid;

	#[test]
	fn current_locale_mut_only_fallback() {
		let mut server = FtlServer {
			fallback: Locale {
				ln: langid!("en"),
				file: None,
				folder: None,
				bundle: None,
			},
			current: None,
			update: false,
		};

		let current = server.current_locale_mut();

		assert_eq!(langid!("en"), current.ln);
	}

	#[test]
	fn current_locale_mut_with_current_field_set() {
		let mut server = FtlServer {
			fallback: Locale {
				ln: langid!("en"),
				file: None,
				folder: None,
				bundle: None,
			},
			current: Some(Locale {
				ln: langid!("fr"),
				file: None,
				folder: None,
				bundle: None,
			}),
			update: false,
		};

		let current = server.current_locale_mut();

		assert_eq!(langid!("fr"), current.ln);
	}

	#[test]
	fn set_localization() {
		let mut server = FtlServer {
			fallback: Locale {
				ln: langid!("en"),
				file: None,
				folder: None,
				bundle: None,
			},
			current: None,
			update: false,
		};

		server.set_localization(langid!("jp"));

		assert_eq!(
			(true, &langid!("jp")),
			(
				*server.update_current_locale(),
				&server.current_locale_mut().ln
			)
		);
	}

	#[test]
	fn set_localization_to_fallback() {
		let mut server = FtlServer {
			fallback: Locale {
				ln: langid!("en"),
				file: None,
				folder: None,
				bundle: None,
			},
			current: Some(Locale {
				ln: langid!("jp"),
				file: None,
				folder: None,
				bundle: None,
			}),
			update: false,
		};

		server.set_localization(langid!("en"));

		assert_eq!(
			(true, &langid!("en")),
			(
				*server.update_current_locale(),
				&server.current_locale_mut().ln
			)
		);
	}
}
