use crate::{
	assets::ftl::Ftl,
	tools::list_string,
	traits::{
		current_locale::CurrentLocaleMut,
		update_current_locale::{UpdateCurrentLocaleFromFile, UpdateCurrentLocaleFromFolder},
	},
};
use bevy::{
	asset::LoadedFolder,
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::{
	error_logger::{ErrorLogger, Log},
	errors::{ErrorData, Level},
	traits::{
		handles_load_tracking::Loaded,
		handles_localization::{
			LocalizationResult,
			Localize,
			SetLocalization,
			Token,
			localized::Localized,
		},
		thread_safe::ThreadSafe,
	},
};
use fluent::{FluentError, FluentResource, concurrent::FluentBundle};
use std::fmt::Display;
use unic_langid::LanguageIdentifier;

#[derive(Resource)]
pub struct FtlServer {
	fallback: Locale,
	current: Option<Locale>,
	update_file: bool,
	update_folder: bool,
}

impl FtlServer {
	pub(crate) fn all_fallback_files_loaded(ftl_server: Res<Self>) -> Loaded {
		if ftl_server.fallback.bundle.is_none() {
			return Loaded(false);
		}

		Loaded(ftl_server.fallback.file.is_none() && ftl_server.fallback.folder.is_none())
	}

	fn get_current(&mut self) -> &mut Locale {
		self.current.as_mut().unwrap_or(&mut self.fallback)
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
			update_file: true,
			update_folder: true,
		}
	}
}

#[derive(SystemParam)]
pub struct FtlServerParam<'w, 's, TLogger = ErrorLogger>
where
	TLogger: SystemParam + ThreadSafe,
{
	server: ResMut<'w, FtlServer>,
	logger: StaticSystemParam<'w, 's, TLogger>,
}

impl<TLogger> CurrentLocaleMut for FtlServerParam<'_, '_, TLogger>
where
	TLogger: SystemParam + ThreadSafe,
{
	fn current_locale_mut(&mut self) -> &mut Locale {
		self.server.get_current()
	}
}

impl<'w, 's, TLogger> SetLocalization for FtlServerParam<'w, 's, TLogger>
where
	TLogger: for<'w2, 's2> SystemParam<Item<'w2, 's2>: Log> + ThreadSafe,
{
	fn set_localization(&mut self, language: LanguageIdentifier) {
		if language == self.server.fallback.ln {
			self.server.current = None;
			self.server.update_file = false;
			self.server.update_folder = false;
			return;
		}

		if matches!(self.server.current.as_ref(), Some(current) if current.ln == language) {
			return;
		}

		self.server.current = Some(Locale {
			ln: language,
			file: None,
			folder: None,
			bundle: None,
		});
		self.server.update_file = true;
		self.server.update_folder = true;
	}
}

impl<'w, 's, TLogger> Localize for FtlServerParam<'w, 's, TLogger>
where
	TLogger: for<'w2, 's2> SystemParam<Item<'w2, 's2>: Log> + ThreadSafe,
{
	fn localize(&self, token: &Token) -> LocalizationResult {
		let (current, locales) = match self.server.current.as_ref() {
			Some(current) => (current, vec![current, &self.server.fallback]),
			None => (&self.server.fallback, vec![&self.server.fallback]),
		};
		let str = &**token;
		let localize = |locale: &&Locale| {
			if locale.ln != current.ln {
				self.logger.log(FtlError::FallbackAttempt {
					token: current.ln_token(str),
					fallback: locale.ln.clone(),
				});
			}

			let Some(bundle) = locale.bundle.as_ref() else {
				self.logger.log(FtlError::NoBundle(locale.ln.clone()));
				return None;
			};

			let Some(msg) = bundle.get_message(str) else {
				self.logger
					.log(FtlError::NoMessageFor(locale.ln_token(str)));
				return None;
			};

			let Some(pattern) = msg.value() else {
				self.logger
					.log(FtlError::NoPatternFor(locale.ln_token(str)));
				return None;
			};

			let mut fluent_errors = vec![];
			let localized = bundle.format_pattern(pattern, None, &mut fluent_errors);

			if !fluent_errors.is_empty() {
				self.logger.log(FtlError::FluentErrors {
					token: locale.ln_token(str),
					errors: fluent_errors,
				});
			}

			Some(Localized::from(localized))
		};

		match locales.iter().find_map(localize) {
			Some(localized) => LocalizationResult::from(localized),
			None => LocalizationResult::from(token.failed()),
		}
	}
}

impl<TLogger> UpdateCurrentLocaleFromFile for FtlServerParam<'_, '_, TLogger>
where
	TLogger: SystemParam + ThreadSafe,
{
	fn update_current_locale_from_file(&mut self) -> &mut bool {
		&mut self.server.update_file
	}
}

impl<TLogger> UpdateCurrentLocaleFromFolder for FtlServerParam<'_, '_, TLogger>
where
	TLogger: SystemParam + ThreadSafe,
{
	fn update_current_locale_from_folder(&mut self) -> &mut bool {
		&mut self.server.update_folder
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

impl ErrorData for FtlError {
	fn level(&self) -> Level {
		match self {
			FtlError::FallbackAttempt { .. } => const { Level::Warning },
			_ => const { Level::Error },
		}
	}

	fn label() -> impl Display {
		const { "Localization error" }
	}

	fn into_details(self) -> impl Display {
		self
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
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp, new_handle};
	use unic_langid::langid;

	#[derive(Resource, NestedMocks)]
	struct _Logger {
		mock: Mock_Logger,
	}

	#[automock]
	impl Log for _Logger {
		fn log<TError>(&self, error: TError)
		where
			TError: ErrorData,
		{
			self.mock.log(error)
		}
	}

	impl Default for _Logger {
		fn default() -> Self {
			Self::new().with_mock(|mock| {
				mock.expect_log::<FtlError>().return_const(());
			})
		}
	}

	type _LoggerParam = Res<'static, _Logger>;

	fn setup(server: FtlServer, logger: _Logger) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(server);
		app.insert_resource(logger);

		app
	}

	#[test]
	fn current_locale_mut_only_fallback() -> Result<(), RunSystemError> {
		let mut app = setup(
			FtlServer {
				fallback: Locale {
					ln: langid!("en"),
					file: None,
					folder: None,
					bundle: None,
				},
				current: None,
				update_file: false,
				update_folder: false,
			},
			default(),
		);

		let current = app
			.world_mut()
			.run_system_once(|mut f: FtlServerParam<_LoggerParam>| {
				f.current_locale_mut().ln.clone()
			})?;

		assert_eq!(langid!("en"), current);
		Ok(())
	}

	#[test]
	fn current_locale_mut_with_current_field_set() -> Result<(), RunSystemError> {
		let mut app = setup(
			FtlServer {
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
				update_file: false,
				update_folder: false,
			},
			default(),
		);

		let current = app
			.world_mut()
			.run_system_once(|mut f: FtlServerParam<_LoggerParam>| {
				f.current_locale_mut().ln.clone()
			})?;

		assert_eq!(langid!("fr"), current);
		Ok(())
	}

	#[test]
	fn set_localization() -> Result<(), RunSystemError> {
		let mut app = setup(
			FtlServer {
				fallback: Locale {
					ln: langid!("en"),
					file: None,
					folder: None,
					bundle: None,
				},
				current: None,
				update_file: false,
				update_folder: false,
			},
			default(),
		);

		let current = app
			.world_mut()
			.run_system_once(|mut f: FtlServerParam<_LoggerParam>| {
				f.set_localization(langid!("jp"));
				f.current_locale_mut().ln.clone()
			})?;

		assert_eq!(langid!("jp"), current);
		Ok(())
	}

	#[test]
	fn set_localization_to_fallback() -> Result<(), RunSystemError> {
		let mut app = setup(
			FtlServer {
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
				update_file: false,
				update_folder: false,
			},
			default(),
		);

		let current = app
			.world_mut()
			.run_system_once(|mut f: FtlServerParam<_LoggerParam>| {
				f.set_localization(langid!("en"));
				(
					*f.update_current_locale_from_folder(),
					f.current_locale_mut().ln.clone(),
				)
			})?;

		assert_eq!((false, langid!("en")), current);
		Ok(())
	}

	#[test]
	fn do_nothing_when_setting_to_current_localization() -> Result<(), RunSystemError> {
		let file = new_handle();
		let folder = new_handle();
		let mut app = setup(
			FtlServer {
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
				update_file: false,
				update_folder: false,
			},
			default(),
		);

		let current = app
			.world_mut()
			.run_system_once(|mut f: FtlServerParam<_LoggerParam>| {
				f.set_localization(langid!("jp"));
				let update = *f.update_current_locale_from_folder();
				let current = f.current_locale_mut();

				(
					update,
					current.ln.clone(),
					current.file.clone(),
					current.folder.clone(),
					current.bundle.is_some(),
				)
			})?;

		assert_eq!(
			(false, langid!("jp"), Some(file), Some(folder), true),
			current
		);
		Ok(())
	}

	#[test]
	fn override_current_localization() -> Result<(), RunSystemError> {
		let mut app = setup(
			FtlServer {
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
				update_file: false,
				update_folder: false,
			},
			default(),
		);

		let current = app
			.world_mut()
			.run_system_once(|mut f: FtlServerParam<_LoggerParam>| {
				f.set_localization(langid!("jp"));
				let update = *f.update_current_locale_from_folder();
				let current = f.current_locale_mut();

				(
					update,
					current.ln.clone(),
					current.file.clone(),
					current.folder.clone(),
					current.bundle.is_some(),
				)
			})?;

		assert_eq!((true, langid!("jp"), None, None, false), current);
		Ok(())
	}

	#[test]
	fn fallback_loaded_if_bundle_present() -> Result<(), RunSystemError> {
		let mut app = setup(
			FtlServer {
				fallback: Locale {
					ln: langid!("en"),
					file: None,
					folder: None,
					bundle: Some(FluentBundle::new_concurrent(vec![langid!("en")])),
				},
				current: None,
				update_file: false,
				update_folder: false,
			},
			default(),
		);

		let Loaded(loaded) = app
			.world_mut()
			.run_system_once(FtlServer::all_fallback_files_loaded)?;

		assert!(loaded);
		Ok(())
	}

	#[test]
	fn fallback_not_loaded_if_no_bundle_present() -> Result<(), RunSystemError> {
		let mut app = setup(
			FtlServer {
				fallback: Locale {
					ln: langid!("en"),
					file: None,
					folder: None,
					bundle: None,
				},
				current: None,
				update_file: false,
				update_folder: false,
			},
			default(),
		);

		let Loaded(loaded) = app
			.world_mut()
			.run_system_once(FtlServer::all_fallback_files_loaded)?;

		assert!(!loaded);
		Ok(())
	}

	#[test]
	fn fallback_not_loaded_if_bundle_file_and_folder_handle_present() -> Result<(), RunSystemError>
	{
		let mut app = setup(
			FtlServer {
				fallback: Locale {
					ln: langid!("en"),
					file: Some(new_handle()),
					folder: Some(new_handle()),
					bundle: Some(FluentBundle::new_concurrent(vec![langid!("en")])),
				},
				current: None,
				update_file: false,
				update_folder: false,
			},
			default(),
		);

		let Loaded(loaded) = app
			.world_mut()
			.run_system_once(FtlServer::all_fallback_files_loaded)?;

		assert!(!loaded);
		Ok(())
	}

	#[test]
	fn fallback_not_loaded_if_only_bundle_and_file_present() -> Result<(), RunSystemError> {
		let mut app = setup(
			FtlServer {
				fallback: Locale {
					ln: langid!("en"),
					file: Some(new_handle()),
					folder: None,
					bundle: Some(FluentBundle::new_concurrent(vec![langid!("en")])),
				},
				current: None,
				update_file: false,
				update_folder: false,
			},
			default(),
		);

		let Loaded(loaded) = app
			.world_mut()
			.run_system_once(FtlServer::all_fallback_files_loaded)?;

		assert!(!loaded);
		Ok(())
	}

	#[test]
	fn fallback_not_loaded_if_only_bundle_and_folder_present() -> Result<(), RunSystemError> {
		let mut app = setup(
			FtlServer {
				fallback: Locale {
					ln: langid!("en"),
					file: None,
					folder: Some(new_handle()),
					bundle: Some(FluentBundle::new_concurrent(vec![langid!("en")])),
				},
				current: None,
				update_file: false,
				update_folder: false,
			},
			default(),
		);

		let Loaded(loaded) = app
			.world_mut()
			.run_system_once(FtlServer::all_fallback_files_loaded)?;

		assert!(!loaded);
		Ok(())
	}

	#[test]
	fn localize_from_fallback() -> Result<(), RunSystemError> {
		let mut bundle = FluentBundle::new_concurrent(vec![langid!("en")]);
		let res = match FluentResource::try_new(String::from("a = A!")) {
			Ok(res) => res,
			Err((res, ..)) => res,
		};
		_ = bundle.add_resource(res);
		let mut app = setup(
			FtlServer {
				fallback: Locale {
					ln: langid!("en"),
					file: None,
					folder: None,
					bundle: Some(bundle),
				},
				current: None,
				update_file: false,
				update_folder: false,
			},
			_Logger::new().with_mock(|mock| {
				mock.expect_log::<FtlError>().never();
			}),
		);

		let result = app
			.world_mut()
			.run_system_once(|f: FtlServerParam<_LoggerParam>| f.localize(&Token::from("a")))?;

		assert_eq!(LocalizationResult::Ok(Localized::from("A!")), result);
		Ok(())
	}

	#[test]
	fn localize_from_current() -> Result<(), RunSystemError> {
		let mut bundle = FluentBundle::new_concurrent(vec![langid!("en")]);
		let res = match FluentResource::try_new(String::from("a = A!")) {
			Ok(res) => res,
			Err((res, ..)) => res,
		};
		_ = bundle.add_resource(res);
		let mut app = setup(
			FtlServer {
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
				update_file: false,
				update_folder: false,
			},
			_Logger::new().with_mock(|mock| {
				mock.expect_log::<FtlError>().never();
			}),
		);

		let result = app
			.world_mut()
			.run_system_once(|f: FtlServerParam<_LoggerParam>| f.localize(&Token::from("a")))?;

		assert_eq!(LocalizationResult::Ok(Localized::from("A!")), result);
		Ok(())
	}

	#[test]
	fn no_bundle_error() -> Result<(), RunSystemError> {
		let mut app = setup(
			FtlServer {
				fallback: Locale {
					ln: langid!("en"),
					file: None,
					folder: None,
					bundle: None,
				},
				current: None,
				update_file: false,
				update_folder: false,
			},
			_Logger::new().with_mock(|mock| {
				mock.expect_log::<FtlError>()
					.once()
					.with(eq(FtlError::NoBundle(langid!("en"))))
					.return_const(());
			}),
		);

		let result = app
			.world_mut()
			.run_system_once(|f: FtlServerParam<_LoggerParam>| f.localize(&Token::from("a")))?;

		assert_eq!(LocalizationResult::Error(Token::from("a").failed()), result);
		Ok(())
	}

	#[test]
	fn no_msg_error() -> Result<(), RunSystemError> {
		let mut bundle = FluentBundle::new_concurrent(vec![langid!("en")]);
		let res = match FluentResource::try_new(String::from("a = A!")) {
			Ok(res) => res,
			Err((res, ..)) => res,
		};
		_ = bundle.add_resource(res);
		let mut app = setup(
			FtlServer {
				fallback: Locale {
					ln: langid!("en"),
					file: None,
					folder: None,
					bundle: Some(bundle),
				},
				current: None,
				update_file: false,
				update_folder: false,
			},
			_Logger::new().with_mock(|mock| {
				mock.expect_log::<FtlError>()
					.once()
					.with(eq(FtlError::NoMessageFor(LnToken {
						value: String::from("b"),
						language: langid!("en"),
					})))
					.return_const(());
			}),
		);

		let result = app
			.world_mut()
			.run_system_once(|f: FtlServerParam<_LoggerParam>| f.localize(&Token::from("b")))?;

		assert_eq!(LocalizationResult::Error(Token::from("b").failed()), result);
		Ok(())
	}

	#[test]
	fn no_patter_error() -> Result<(), RunSystemError> {
		let mut bundle = FluentBundle::new_concurrent(vec![langid!("en")]);
		let res = match FluentResource::try_new(String::from("a = \n  .description = my item")) {
			Ok(res) => res,
			Err((res, ..)) => res,
		};
		_ = bundle.add_resource(res);
		let mut app = setup(
			FtlServer {
				fallback: Locale {
					ln: langid!("en"),
					file: None,
					folder: None,
					bundle: Some(bundle),
				},
				current: None,
				update_file: false,
				update_folder: false,
			},
			_Logger::new().with_mock(|mock| {
				mock.expect_log::<FtlError>()
					.once()
					.with(eq(FtlError::NoPatternFor(LnToken {
						value: String::from("a"),
						language: langid!("en"),
					})))
					.return_const(());
			}),
		);

		let result = app
			.world_mut()
			.run_system_once(|f: FtlServerParam<_LoggerParam>| f.localize(&Token::from("a")))?;

		assert_eq!(LocalizationResult::Error(Token::from("a").failed()), result);
		Ok(())
	}

	#[test]
	fn fluent_error() -> Result<(), RunSystemError> {
		let mut bundle = FluentBundle::new_concurrent(vec![langid!("en")]);
		let res = match FluentResource::try_new(String::from("a = { $a }")) {
			Ok(res) => res,
			Err((res, ..)) => res,
		};
		_ = bundle.add_resource(res);
		let mut app = setup(
			FtlServer {
				fallback: Locale {
					ln: langid!("en"),
					file: None,
					folder: None,
					bundle: Some(bundle),
				},
				current: None,
				update_file: false,
				update_folder: false,
			},
			_Logger::new().with_mock(|mock| {
				mock.expect_log::<FtlError>()
					.once()
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
			}),
		);

		let result = app
			.world_mut()
			.run_system_once(|f: FtlServerParam<_LoggerParam>| f.localize(&Token::from("a")))?;

		assert_eq!(LocalizationResult::Ok(Localized::from("{$a}")), result);
		Ok(())
	}

	#[test]
	fn fallback_attempt_on_bundle_error() -> Result<(), RunSystemError> {
		let mut fallback = FluentBundle::new_concurrent(vec![langid!("en")]);
		let res = match FluentResource::try_new(String::from("a = A!")) {
			Ok(res) => res,
			Err((res, ..)) => res,
		};
		_ = fallback.add_resource(res);
		let mut app = setup(
			FtlServer {
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
				update_file: false,
				update_folder: false,
			},
			_Logger::new().with_mock(|mock| {
				mock.expect_log::<FtlError>()
					.once()
					.with(eq(FtlError::NoBundle(langid!("jp"))))
					.return_const(());
				mock.expect_log::<FtlError>()
					.once()
					.with(eq(FtlError::FallbackAttempt {
						token: LnToken {
							value: String::from("a"),
							language: langid!("jp"),
						},
						fallback: langid!("en"),
					}))
					.return_const(());
			}),
		);

		let result = app
			.world_mut()
			.run_system_once(|f: FtlServerParam<_LoggerParam>| f.localize(&Token::from("a")))?;

		assert_eq!(LocalizationResult::Ok(Localized::from("A!")), result);
		Ok(())
	}

	#[test]
	fn fallback_attempt_on_msg_error() -> Result<(), RunSystemError> {
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
		let mut app = setup(
			FtlServer {
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
				update_file: false,
				update_folder: false,
			},
			_Logger::new().with_mock(|mock| {
				mock.expect_log::<FtlError>()
					.once()
					.with(eq(FtlError::NoMessageFor(LnToken {
						value: String::from("a"),
						language: langid!("jp"),
					})))
					.return_const(());
				mock.expect_log::<FtlError>()
					.once()
					.with(eq(FtlError::FallbackAttempt {
						token: LnToken {
							value: String::from("a"),
							language: langid!("jp"),
						},
						fallback: langid!("en"),
					}))
					.return_const(());
			}),
		);

		let result = app
			.world_mut()
			.run_system_once(|f: FtlServerParam<_LoggerParam>| f.localize(&Token::from("a")))?;

		assert_eq!(LocalizationResult::Ok(Localized::from("A!")), result);
		Ok(())
	}

	#[test]
	fn fallback_attempt_on_pattern_error() -> Result<(), RunSystemError> {
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
		let mut app = setup(
			FtlServer {
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
				update_file: false,
				update_folder: false,
			},
			_Logger::new().with_mock(|mock| {
				mock.expect_log::<FtlError>()
					.once()
					.with(eq(FtlError::NoPatternFor(LnToken {
						value: String::from("a"),
						language: langid!("jp"),
					})))
					.return_const(());
				mock.expect_log::<FtlError>()
					.once()
					.with(eq(FtlError::FallbackAttempt {
						token: LnToken {
							value: String::from("a"),
							language: langid!("jp"),
						},
						fallback: langid!("en"),
					}))
					.return_const(());
			}),
		);

		let result = app
			.world_mut()
			.run_system_once(|f: FtlServerParam<_LoggerParam>| f.localize(&Token::from("a")))?;

		assert_eq!(LocalizationResult::Ok(Localized::from("A!")), result);
		Ok(())
	}

	#[test]
	fn no_fallback_attempt_on_fluent_error() -> Result<(), RunSystemError> {
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
		let mut app = setup(
			FtlServer {
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
				update_file: false,
				update_folder: false,
			},
			_Logger::new().with_mock(|mock| {
				mock.expect_log::<FtlError>()
					.once()
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
			}),
		);

		let result = app
			.world_mut()
			.run_system_once(|f: FtlServerParam<_LoggerParam>| f.localize(&Token::from("a")))?;

		assert_eq!(LocalizationResult::Ok(Localized::from("{$a}")), result);
		Ok(())
	}
}
