use crate::{
	assets::ftl::Ftl,
	traits::{
		current_locale::CurrentLocaleMut,
		get_errors_mut::GetErrorsMut,
		requested_language::UpdateCurrentLocaleMut,
	},
};
use bevy::{asset::LoadedFolder, prelude::*};
use common::{
	errors::{Error, Level},
	traits::{handles_load_tracking::Loaded, handles_localization::SetLocalization},
};
use fluent::{FluentError, FluentResource, concurrent::FluentBundle};
use std::fmt::Display;
use unic_langid::LanguageIdentifier;

#[derive(Resource)]
pub struct FtlServer {
	fallback: Locale,
	current: Option<Locale>,
	update: bool,
	translate_errors: Vec<TranslateError>,
}

impl FtlServer {
	pub(crate) fn all_fallback_files_loaded(ftl_server: Res<Self>) -> Loaded {
		if ftl_server.fallback.bundle.is_none() {
			return Loaded(false);
		}

		Loaded(ftl_server.fallback.file.is_none() && ftl_server.fallback.folder.is_none())
	}
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
			translate_errors: vec![],
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
		(self.current, self.update) = match language == self.fallback.ln {
			true => (None, false),
			false => (
				Some(Locale {
					ln: language,
					file: None,
					folder: None,
					bundle: None,
				}),
				true,
			),
		};
	}
}

impl UpdateCurrentLocaleMut for FtlServer {
	fn update_current_locale(&mut self) -> &mut bool {
		&mut self.update
	}
}

impl GetErrorsMut for FtlServer {
	type TError = TranslateError;

	fn errors_mut(&mut self) -> &mut Vec<Self::TError> {
		&mut self.translate_errors
	}
}

pub(crate) struct Locale {
	pub(crate) ln: LanguageIdentifier,
	pub(crate) file: Option<Handle<Ftl>>,
	pub(crate) folder: Option<Handle<LoadedFolder>>,
	pub(crate) bundle: Option<FluentBundle<FluentResource>>,
}

#[derive(Debug)]
pub enum TranslateError {
	NoBundle(LanguageIdentifier),
	NoMessageFor(Token),
	NoPatternFor(Token),
	FallbackAttempt {
		token: Token,
		fallback: LanguageIdentifier,
	},
	FluentErrors {
		token: Token,
		errors: Vec<FluentError>,
	},
}

impl From<TranslateError> for Error {
	fn from(error: TranslateError) -> Self {
		match error {
			TranslateError::NoBundle(ln) => Error {
				msg: format!("no `FluentBundle` for {ln}"),
				lvl: Level::Error,
			},
			TranslateError::NoMessageFor(token) => Error {
				msg: format!("no message found for {token}"),
				lvl: Level::Error,
			},
			TranslateError::NoPatternFor(token) => Error {
				msg: format!("no pattern found for {token}"),
				lvl: Level::Error,
			},
			TranslateError::FallbackAttempt { token, fallback } => Error {
				msg: format!("fallback attempted for {token} -> {fallback}"),
				lvl: Level::Warning,
			},
			TranslateError::FluentErrors { token, errors } => {
				let errors = errors
					.iter()
					.map(FluentError::to_string)
					.collect::<Vec<_>>()
					.join(", ");
				Error {
					msg: format!("errors for {token}: {errors}"),
					lvl: Level::Error,
				}
			}
		}
	}
}

#[derive(Debug)]
pub struct Token {
	value: String,
	language: LanguageIdentifier,
}

impl Display for Token {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} ({})", self.value, self.language)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::test_tools::utils::{SingleThreadedApp, new_handle};
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
			translate_errors: vec![],
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
			translate_errors: vec![],
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
			translate_errors: vec![],
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
			translate_errors: vec![],
			update: false,
		};

		server.set_localization(langid!("en"));

		assert_eq!(
			(false, &langid!("en")),
			(
				*server.update_current_locale(),
				&server.current_locale_mut().ln
			)
		);
	}

	fn setup(server: FtlServer) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(server);

		app
	}

	#[test]
	fn fallback_loaded_if_bundle_present() -> Result<(), RunSystemError> {
		let mut app = setup(FtlServer {
			fallback: Locale {
				ln: langid!("en"),
				file: None,
				folder: None,
				bundle: Some(FluentBundle::new_concurrent(vec![langid!("en")])),
			},
			current: None,
			translate_errors: vec![],
			update: false,
		});

		let Loaded(loaded) = app
			.world_mut()
			.run_system_once(FtlServer::all_fallback_files_loaded)?;

		assert!(loaded);
		Ok(())
	}

	#[test]
	fn fallback_not_loaded_if_no_bundle_present() -> Result<(), RunSystemError> {
		let mut app = setup(FtlServer {
			fallback: Locale {
				ln: langid!("en"),
				file: None,
				folder: None,
				bundle: None,
			},
			current: None,
			translate_errors: vec![],
			update: false,
		});

		let Loaded(loaded) = app
			.world_mut()
			.run_system_once(FtlServer::all_fallback_files_loaded)?;

		assert!(!loaded);
		Ok(())
	}

	#[test]
	fn fallback_not_loaded_if_bundle_file_and_folder_handle_present() -> Result<(), RunSystemError>
	{
		let mut app = setup(FtlServer {
			fallback: Locale {
				ln: langid!("en"),
				file: Some(new_handle()),
				folder: Some(new_handle()),
				bundle: Some(FluentBundle::new_concurrent(vec![langid!("en")])),
			},
			current: None,
			translate_errors: vec![],
			update: false,
		});

		let Loaded(loaded) = app
			.world_mut()
			.run_system_once(FtlServer::all_fallback_files_loaded)?;

		assert!(!loaded);
		Ok(())
	}

	#[test]
	fn fallback_not_loaded_if_only_bundle_and_file_present() -> Result<(), RunSystemError> {
		let mut app = setup(FtlServer {
			fallback: Locale {
				ln: langid!("en"),
				file: Some(new_handle()),
				folder: None,
				bundle: Some(FluentBundle::new_concurrent(vec![langid!("en")])),
			},
			current: None,
			translate_errors: vec![],
			update: false,
		});

		let Loaded(loaded) = app
			.world_mut()
			.run_system_once(FtlServer::all_fallback_files_loaded)?;

		assert!(!loaded);
		Ok(())
	}

	#[test]
	fn fallback_not_loaded_if_only_bundle_and_folder_present() -> Result<(), RunSystemError> {
		let mut app = setup(FtlServer {
			fallback: Locale {
				ln: langid!("en"),
				file: None,
				folder: Some(new_handle()),
				bundle: Some(FluentBundle::new_concurrent(vec![langid!("en")])),
			},
			current: None,
			translate_errors: vec![],
			update: false,
		});

		let Loaded(loaded) = app
			.world_mut()
			.run_system_once(FtlServer::all_fallback_files_loaded)?;

		assert!(!loaded);
		Ok(())
	}
}
