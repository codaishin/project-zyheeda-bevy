use crate::{assets::ftl::Ftl, traits::current_locale::CurrentLocaleMut};
use bevy::{asset::LoadedFolder, prelude::*};
use fluent::{FluentResource, concurrent::FluentBundle};
use unic_langid::LanguageIdentifier;

#[derive(Resource)]
pub struct FtlServer {
	fallback: Locale,
	current: Option<Locale>,
	requested_current: Option<LanguageIdentifier>,
}

pub(crate) struct Locale {
	pub(crate) ln: LanguageIdentifier,
	pub(crate) file: Option<Handle<Ftl>>,
	pub(crate) folder: Option<Handle<LoadedFolder>>,
	pub(crate) bundle: Option<FluentBundle<FluentResource>>,
}

impl From<(LanguageIdentifier, Handle<Ftl>, Handle<LoadedFolder>)> for FtlServer {
	fn from(
		(index, file, folder): (LanguageIdentifier, Handle<Ftl>, Handle<LoadedFolder>),
	) -> Self {
		Self {
			requested_current: None,
			fallback: Locale {
				ln: index,
				file: Some(file),
				folder: Some(folder),
				bundle: None,
			},
			current: None,
		}
	}
}

impl CurrentLocaleMut for FtlServer {
	fn current_locale_mut(&mut self) -> &mut Locale {
		self.current.as_mut().unwrap_or(&mut self.fallback)
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
			requested_current: None,
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
			requested_current: None,
		};

		let current = server.current_locale_mut();

		assert_eq!(langid!("fr"), current.ln);
	}
}
