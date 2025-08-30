use crate::{
	assets::ftl::Ftl,
	tools::list_string,
	traits::{current_locale::CurrentLocaleMut, requested_language::UpdateCurrentLocaleMut},
};
use bevy::{asset::LoadedFolder, prelude::*};
use common::traits::{
	handles_load_tracking::Loaded,
	handles_localization::{
		LocalizationResult,
		LocalizeToken,
		SetLocalization,
		Token,
		localized::Localized,
	},
};
use fluent::{FluentError, FluentResource, concurrent::FluentBundle};
use std::fmt::Display;
use unic_langid::LanguageIdentifier;
use zyheeda_core::logger::{Log, Logger};

#[derive(Resource)]
pub struct FtlServer<TLogger = Logger>
where
	TLogger: Log,
{
	fallback: Locale,
	current: Option<Locale>,
	update: bool,
	logger: TLogger,
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
			logger: Logger,
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
		if language == self.fallback.ln {
			self.current = None;
			self.update = false;
			return;
		}

		if matches!(self.current.as_ref(), Some(current) if current.ln == language) {
			return;
		}

		self.current = Some(Locale {
			ln: language,
			file: None,
			folder: None,
			bundle: None,
		});
		self.update = true;
	}
}

impl<TLogger> LocalizeToken for FtlServer<TLogger>
where
	TLogger: Log,
{
	fn localize_token<TToken>(&self, token: TToken) -> LocalizationResult
	where
		TToken: Into<Token>,
	{
		let (current, locales) = match self.current.as_ref() {
			Some(current) => (current, vec![current, &self.fallback]),
			None => (&self.fallback, vec![&self.fallback]),
		};
		let str = &*token.into();
		let localize = |locale: &&Locale| {
			if locale.ln != current.ln {
				self.logger.log_warning(FtlError::FallbackAttempt {
					token: current.ln_token(str),
					fallback: locale.ln.clone(),
				});
			}

			let Some(bundle) = locale.bundle.as_ref() else {
				self.logger.log_error(FtlError::NoBundle(locale.ln.clone()));
				return None;
			};

			let Some(msg) = bundle.get_message(str) else {
				self.logger
					.log_error(FtlError::NoMessageFor(locale.ln_token(str)));
				return None;
			};

			let Some(pattern) = msg.value() else {
				self.logger
					.log_error(FtlError::NoPatternFor(locale.ln_token(str)));
				return None;
			};

			let mut fluent_errors = vec![];
			let localized = bundle.format_pattern(pattern, None, &mut fluent_errors);

			if !fluent_errors.is_empty() {
				self.logger.log_error(FtlError::FluentErrors {
					token: locale.ln_token(str),
					errors: fluent_errors,
				});
			}

			Some(String::from(localized))
		};

		match locales.iter().find_map(localize) {
			Some(localized) => LocalizationResult::Ok(Localized::from(localized)),
			None => LocalizationResult::Error(Token::from(str).failed()),
		}
	}
}

impl UpdateCurrentLocaleMut for FtlServer {
	fn update_current_locale(&mut self) -> &mut bool {
		&mut self.update
	}
}

pub(crate) struct Locale {
	pub(crate) ln: LanguageIdentifier,
	pub(crate) file: Option<Handle<Ftl>>,
	pub(crate) folder: Option<Handle<LoadedFolder>>,
	pub(crate) bundle: Option<FluentBundle<FluentResource>>,
}

impl Locale {
	fn ln_token(&self, value: &str) -> LnToken {
		LnToken {
			value: String::from(value),
			language: self.ln.clone(),
		}
	}
}

#[derive(Debug, PartialEq)]
pub enum FtlError {
	NoBundle(LanguageIdentifier),
	NoMessageFor(LnToken),
	NoPatternFor(LnToken),
	FluentErrors {
		token: LnToken,
		errors: Vec<FluentError>,
	},
	FallbackAttempt {
		token: LnToken,
		fallback: LanguageIdentifier,
	},
}

impl Display for FtlError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			FtlError::NoBundle(ln) => write!(f, "no `FluentBundle` for {ln}"),
			FtlError::NoMessageFor(token) => write!(f, "no message found for {token}"),
			FtlError::NoPatternFor(token) => write!(f, "no pattern found for {token}"),
			FtlError::FluentErrors { token, errors } => {
				write!(f, "errors for {token}:\n{}", list_string(errors))
			}
			FtlError::FallbackAttempt { token, fallback } => {
				write!(f, "fallback attempted for {token} -> {fallback}")
			}
		}
	}
}

#[derive(Debug, PartialEq)]
pub struct LnToken {
	value: String,
	language: LanguageIdentifier,
}

impl Display for LnToken {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} ({})", self.value, self.language)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use fluent::resolver::{ResolverError, errors::ReferenceKind};
	use mockall::{mock, predicate::eq};
	use testing::{Mock, SingleThreadedApp, new_handle, simple_init};
	use unic_langid::langid;

	mock! {
		_Logger {}
		impl Log for _Logger {
			fn log_warning<TError>(&self, value: TError) where TError: 'static;
			fn log_error<TError>(&self, value: TError) where TError: 'static;
		}
	}

	simple_init!(Mock_Logger);

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
			logger: Logger,
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
			logger: Logger,
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
			logger: Logger,
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
			logger: Logger,
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

	#[test]
	fn do_nothing_when_setting_to_current_localization() {
		let file = new_handle();
		let folder = new_handle();
		let mut server = FtlServer {
			fallback: Locale {
				ln: langid!("en"),
				file: None,
				folder: None,
				bundle: None,
			},
			current: Some(Locale {
				ln: langid!("jp"),
				file: Some(file.clone()),
				folder: Some(folder.clone()),
				bundle: Some(FluentBundle::new_concurrent(vec![langid!("jp")])),
			}),
			logger: Logger,
			update: false,
		};

		server.set_localization(langid!("jp"));

		let update = *server.update_current_locale();
		let locale = &server.current_locale_mut();
		assert_eq!(
			(false, &langid!("jp"), &Some(file), &Some(folder), true),
			(
				update,
				&locale.ln,
				&locale.file,
				&locale.folder,
				locale.bundle.is_some(),
			)
		);
	}

	#[test]
	fn override_current_localization() {
		let mut server = FtlServer {
			fallback: Locale {
				ln: langid!("en"),
				file: None,
				folder: None,
				bundle: None,
			},
			current: Some(Locale {
				ln: langid!("fr"),
				file: Some(new_handle()),
				folder: Some(new_handle()),
				bundle: Some(FluentBundle::new_concurrent(vec![langid!("jp")])),
			}),
			logger: Logger,
			update: false,
		};

		server.set_localization(langid!("jp"));

		let update = *server.update_current_locale();
		let locale = &server.current_locale_mut();
		assert_eq!(
			(true, &langid!("jp"), &None, &None, false),
			(
				update,
				&locale.ln,
				&locale.file,
				&locale.folder,
				locale.bundle.is_some(),
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
			logger: Logger,
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
			logger: Logger,
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
			logger: Logger,
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
			logger: Logger,
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
			logger: Logger,
			update: false,
		});

		let Loaded(loaded) = app
			.world_mut()
			.run_system_once(FtlServer::all_fallback_files_loaded)?;

		assert!(!loaded);
		Ok(())
	}

	#[test]
	fn localize_from_fallback() {
		let mut bundle = FluentBundle::new_concurrent(vec![langid!("en")]);
		let res = match FluentResource::try_new(String::from("a = A!")) {
			Ok(res) => res,
			Err((res, ..)) => res,
		};
		_ = bundle.add_resource(res);
		let server = FtlServer {
			fallback: Locale {
				ln: langid!("en"),
				file: None,
				folder: None,
				bundle: Some(bundle),
			},
			current: None,
			logger: Mock_Logger::new_mock(|mock| {
				mock.expect_log_error::<FtlError>().never();
				mock.expect_log_warning::<FtlError>().never();
			}),
			update: false,
		};

		assert_eq!(
			LocalizationResult::Ok(Localized::from("A!")),
			server.localize_token("a")
		);
	}

	#[test]
	fn localize_from_current() {
		let mut bundle = FluentBundle::new_concurrent(vec![langid!("en")]);
		let res = match FluentResource::try_new(String::from("a = A!")) {
			Ok(res) => res,
			Err((res, ..)) => res,
		};
		_ = bundle.add_resource(res);
		let server = FtlServer {
			fallback: Locale {
				ln: langid!("jp"),
				file: None,
				folder: None,
				bundle: None,
			},
			current: Some(Locale {
				ln: langid!("en"),
				file: None,
				folder: None,
				bundle: Some(bundle),
			}),
			logger: Mock_Logger::new_mock(|mock| {
				mock.expect_log_error::<FtlError>().never();
				mock.expect_log_warning::<FtlError>().never();
			}),
			update: false,
		};

		assert_eq!(
			LocalizationResult::Ok(Localized::from("A!")),
			server.localize_token("a")
		);
	}

	#[test]
	fn no_bundle_error() {
		let server = FtlServer {
			fallback: Locale {
				ln: langid!("en"),
				file: None,
				folder: None,
				bundle: None,
			},
			current: None,
			logger: Mock_Logger::new_mock(|mock| {
				mock.expect_log_error::<FtlError>()
					.times(1)
					.with(eq(FtlError::NoBundle(langid!("en"))))
					.return_const(());
				mock.expect_log_warning::<FtlError>().never();
			}),
			update: false,
		};

		assert_eq!(
			LocalizationResult::Error(Token::from("a").failed()),
			server.localize_token("a")
		);
	}

	#[test]
	fn no_msg_error() {
		let mut bundle = FluentBundle::new_concurrent(vec![langid!("en")]);
		let res = match FluentResource::try_new(String::from("a = A!")) {
			Ok(res) => res,
			Err((res, ..)) => res,
		};
		_ = bundle.add_resource(res);
		let server = FtlServer {
			fallback: Locale {
				ln: langid!("en"),
				file: None,
				folder: None,
				bundle: Some(bundle),
			},
			current: None,
			logger: Mock_Logger::new_mock(|mock| {
				mock.expect_log_error::<FtlError>()
					.times(1)
					.with(eq(FtlError::NoMessageFor(LnToken {
						value: String::from("b"),
						language: langid!("en"),
					})))
					.return_const(());
				mock.expect_log_warning::<FtlError>().never();
			}),
			update: false,
		};

		assert_eq!(
			LocalizationResult::Error(Token::from("b").failed()),
			server.localize_token("b")
		);
	}

	#[test]
	fn no_patter_error() {
		let mut bundle = FluentBundle::new_concurrent(vec![langid!("en")]);
		let res = match FluentResource::try_new(String::from("a = \n  .description = my item")) {
			Ok(res) => res,
			Err((res, ..)) => res,
		};
		_ = bundle.add_resource(res);
		let server = FtlServer {
			fallback: Locale {
				ln: langid!("en"),
				file: None,
				folder: None,
				bundle: Some(bundle),
			},
			current: None,
			logger: Mock_Logger::new_mock(|mock| {
				mock.expect_log_error::<FtlError>()
					.times(1)
					.with(eq(FtlError::NoPatternFor(LnToken {
						value: String::from("a"),
						language: langid!("en"),
					})))
					.return_const(());
				mock.expect_log_warning::<FtlError>().never();
			}),
			update: false,
		};

		assert_eq!(
			LocalizationResult::Error(Token::from("a").failed()),
			server.localize_token("a"),
		);
	}

	#[test]
	fn fluent_error() {
		let mut bundle = FluentBundle::new_concurrent(vec![langid!("en")]);
		let res = match FluentResource::try_new(String::from("a = { $a }")) {
			Ok(res) => res,
			Err((res, ..)) => res,
		};
		_ = bundle.add_resource(res);
		let server = FtlServer {
			fallback: Locale {
				ln: langid!("en"),
				file: None,
				folder: None,
				bundle: Some(bundle),
			},
			current: None,
			logger: Mock_Logger::new_mock(|mock| {
				mock.expect_log_error::<FtlError>()
					.times(1)
					.with(eq(FtlError::FluentErrors {
						token: LnToken {
							value: String::from("a"),
							language: langid!("en"),
						},
						errors: vec![FluentError::ResolverError(ResolverError::Reference(
							ReferenceKind::Variable {
								id: String::from("a"),
							},
						))],
					}))
					.return_const(());
				mock.expect_log_warning::<FtlError>().never();
			}),
			update: false,
		};

		assert_eq!(
			LocalizationResult::Ok(Localized::from("{$a}")),
			server.localize_token("a"),
		);
	}

	#[test]
	fn fallback_attempt_on_bundle_error() {
		let mut fallback = FluentBundle::new_concurrent(vec![langid!("en")]);
		let res = match FluentResource::try_new(String::from("a = A!")) {
			Ok(res) => res,
			Err((res, ..)) => res,
		};
		_ = fallback.add_resource(res);
		let server = FtlServer {
			fallback: Locale {
				ln: langid!("en"),
				file: None,
				folder: None,
				bundle: Some(fallback),
			},
			current: Some(Locale {
				ln: langid!("jp"),
				file: None,
				folder: None,
				bundle: None,
			}),
			logger: Mock_Logger::new_mock(|mock| {
				mock.expect_log_error::<FtlError>()
					.times(1)
					.with(eq(FtlError::NoBundle(langid!("jp"))))
					.return_const(());
				mock.expect_log_warning::<FtlError>()
					.times(1)
					.with(eq(FtlError::FallbackAttempt {
						token: LnToken {
							value: String::from("a"),
							language: langid!("jp"),
						},
						fallback: langid!("en"),
					}))
					.return_const(());
			}),
			update: false,
		};

		assert_eq!(
			LocalizationResult::Ok(Localized::from("A!")),
			server.localize_token("a"),
		);
	}

	#[test]
	fn fallback_attempt_on_msg_error() {
		let mut fallback = FluentBundle::new_concurrent(vec![langid!("en")]);
		let res = match FluentResource::try_new(String::from("a = A!")) {
			Ok(res) => res,
			Err((res, ..)) => res,
		};
		_ = fallback.add_resource(res);
		let mut current = FluentBundle::new_concurrent(vec![langid!("en")]);
		let res = match FluentResource::try_new(String::from("b = B!")) {
			Ok(res) => res,
			Err((res, ..)) => res,
		};
		_ = current.add_resource(res);
		let server = FtlServer {
			fallback: Locale {
				ln: langid!("en"),
				file: None,
				folder: None,
				bundle: Some(fallback),
			},
			current: Some(Locale {
				ln: langid!("jp"),
				file: None,
				folder: None,
				bundle: Some(current),
			}),
			logger: Mock_Logger::new_mock(|mock| {
				mock.expect_log_error::<FtlError>()
					.times(1)
					.with(eq(FtlError::NoMessageFor(LnToken {
						value: String::from("a"),
						language: langid!("jp"),
					})))
					.return_const(());
				mock.expect_log_warning::<FtlError>()
					.times(1)
					.with(eq(FtlError::FallbackAttempt {
						token: LnToken {
							value: String::from("a"),
							language: langid!("jp"),
						},
						fallback: langid!("en"),
					}))
					.return_const(());
			}),
			update: false,
		};

		assert_eq!(
			LocalizationResult::Ok(Localized::from("A!")),
			server.localize_token("a"),
		);
	}

	#[test]
	fn fallback_attempt_on_pattern_error() {
		let mut fallback = FluentBundle::new_concurrent(vec![langid!("en")]);
		let res = match FluentResource::try_new(String::from("a = A!")) {
			Ok(res) => res,
			Err((res, ..)) => res,
		};
		_ = fallback.add_resource(res);
		let mut current = FluentBundle::new_concurrent(vec![langid!("en")]);
		let res = match FluentResource::try_new(String::from("a = \n  .description = my item")) {
			Ok(res) => res,
			Err((res, ..)) => res,
		};
		_ = current.add_resource(res);
		let server = FtlServer {
			fallback: Locale {
				ln: langid!("en"),
				file: None,
				folder: None,
				bundle: Some(fallback),
			},
			current: Some(Locale {
				ln: langid!("jp"),
				file: None,
				folder: None,
				bundle: Some(current),
			}),
			logger: Mock_Logger::new_mock(|mock| {
				mock.expect_log_error::<FtlError>()
					.times(1)
					.with(eq(FtlError::NoPatternFor(LnToken {
						value: String::from("a"),
						language: langid!("jp"),
					})))
					.return_const(());
				mock.expect_log_warning::<FtlError>()
					.times(1)
					.with(eq(FtlError::FallbackAttempt {
						token: LnToken {
							value: String::from("a"),
							language: langid!("jp"),
						},
						fallback: langid!("en"),
					}))
					.return_const(());
			}),
			update: false,
		};

		assert_eq!(
			LocalizationResult::Ok(Localized::from("A!")),
			server.localize_token("a"),
		);
	}

	#[test]
	fn no_fallback_attempt_on_fluent_error() {
		let mut fallback = FluentBundle::new_concurrent(vec![langid!("en")]);
		let res = match FluentResource::try_new(String::from("a = A!")) {
			Ok(res) => res,
			Err((res, ..)) => res,
		};
		_ = fallback.add_resource(res);
		let mut current = FluentBundle::new_concurrent(vec![langid!("jp")]);
		let res = match FluentResource::try_new(String::from("a = { $a }")) {
			Ok(res) => res,
			Err((res, ..)) => res,
		};
		_ = current.add_resource(res);
		let server = FtlServer {
			fallback: Locale {
				ln: langid!("en"),
				file: None,
				folder: None,
				bundle: Some(fallback),
			},
			current: Some(Locale {
				ln: langid!("jp"),
				file: None,
				folder: None,
				bundle: Some(current),
			}),
			logger: Mock_Logger::new_mock(|mock| {
				mock.expect_log_error::<FtlError>()
					.times(1)
					.with(eq(FtlError::FluentErrors {
						token: LnToken {
							value: String::from("a"),
							language: langid!("jp"),
						},
						errors: vec![FluentError::ResolverError(ResolverError::Reference(
							ReferenceKind::Variable {
								id: String::from("a"),
							},
						))],
					}))
					.return_const(());
				mock.expect_log_warning::<FtlError>().never();
			}),
			update: false,
		};

		assert_eq!(
			LocalizationResult::Ok(Localized::from("{$a}")),
			server.localize_token("a"),
		);
	}
}
