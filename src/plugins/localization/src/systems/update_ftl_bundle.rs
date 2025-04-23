use crate::{
	assets::ftl::Ftl,
	resources::ftl_server::Locale,
	traits::current_locale::CurrentLocaleMut,
};
use bevy::{asset::LoadedFolder, prelude::*};
use common::errors::{Error, Level};
use fluent::{FluentError, FluentResource, concurrent::FluentBundle};
use fluent_syntax::parser::ParserError;
use unic_langid::LanguageIdentifier;

impl<T> UpdateFtlBundle for T where T: Resource + CurrentLocaleMut {}

pub(crate) trait UpdateFtlBundle: Resource + CurrentLocaleMut {
	fn update_ftl_bundle(
		mut server: ResMut<Self>,
		mut events: EventReader<AssetEvent<Ftl>>,
		files: Res<Assets<Ftl>>,
		mut folders: ResMut<Assets<LoadedFolder>>,
	) -> Vec<Result<(), SetBundleError>> {
		let locale = server.current_locale_mut();

		events
			.read()
			.filter_map(added_id)
			.map(update_bundle(locale, &files, &mut folders))
			.collect()
	}
}

fn added_id(event: &AssetEvent<Ftl>) -> Option<&AssetId<Ftl>> {
	match event {
		AssetEvent::Added { id } => Some(id),
		_ => None,
	}
}

fn update_bundle(
	locale: &mut Locale,
	assets: &Assets<Ftl>,
	folders: &mut Assets<LoadedFolder>,
) -> impl FnMut(&AssetId<Ftl>) -> Result<(), SetBundleError> {
	move |id| {
		let Some(id) = removed_handle_id(id, locale, folders) else {
			return Ok(());
		};

		let Ftl(file) = get_ftl_file(assets, id, &locale.ln)?;
		let (res, parse_errors) = new_resource(file);
		let bundle = locale
			.bundle
			.get_or_insert_with(|| FluentBundle::new_concurrent(vec![locale.ln.clone()]));
		let fluent_errors = bundle.add_resource(res).err().unwrap_or_default();

		if !parse_errors.is_empty() || !fluent_errors.is_empty() {
			return Err(SetBundleError::fluent_errors(
				&locale.ln,
				parse_errors,
				fluent_errors,
			));
		}

		Ok(())
	}
}

fn removed_handle_id(
	id: &AssetId<Ftl>,
	locale: &mut Locale,
	folders: &mut Assets<LoadedFolder>,
) -> Option<AssetId<Ftl>> {
	let id = match locale.file.as_ref() {
		Some(file) if &file.id() == id => {
			return locale.file.take().map(|handle| handle.id());
		}
		_ => id.untyped(),
	};

	let folder = locale.folder.as_ref()?;
	let LoadedFolder { handles } = folders.get_mut(folder)?;
	let i = handles.iter().position(|handle| handle.id() == id)?;
	let removed = handles.remove(i).id().typed();

	if handles.is_empty() {
		locale.folder = None;
	}

	Some(removed)
}

fn get_ftl_file<'a>(
	assets: &'a Assets<Ftl>,
	id: AssetId<Ftl>,
	ln: &LanguageIdentifier,
) -> Result<&'a Ftl, SetBundleError> {
	let Some(file) = assets.get(id) else {
		return Err(SetBundleError::no_ftl_file(ln));
	};

	Ok(file)
}

fn new_resource(file: &str) -> (FluentResource, Vec<ParserError>) {
	match FluentResource::try_new(file.to_owned()) {
		Err((res, errors)) => (res, errors),
		Ok(res) => (res, vec![]),
	}
}

#[derive(Debug, PartialEq)]
pub(crate) struct SetBundleError {
	ln: LanguageIdentifier,
	kind: SetBundleErrorKind,
}

impl SetBundleError {
	fn no_ftl_file(ln: &LanguageIdentifier) -> Self {
		Self {
			ln: ln.clone(),
			kind: SetBundleErrorKind::NoFtlFile,
		}
	}

	fn fluent_errors(
		ln: &LanguageIdentifier,
		parse_errors: Vec<ParserError>,
		fluent_errors: Vec<FluentError>,
	) -> Self {
		Self {
			ln: ln.clone(),
			kind: SetBundleErrorKind::FluentError(parse_errors, fluent_errors),
		}
	}
}

#[derive(Debug, PartialEq)]
pub(crate) enum SetBundleErrorKind {
	NoFtlFile,
	FluentError(Vec<ParserError>, Vec<FluentError>),
}

impl From<SetBundleError> for Error {
	fn from(SetBundleError { ln, kind }: SetBundleError) -> Self {
		match kind {
			SetBundleErrorKind::NoFtlFile => Error {
				msg: format!("no file found for {ln:?}"),
				lvl: Level::Error,
			},
			SetBundleErrorKind::FluentError(parse_errors, fluent_errors) => Error {
				msg: format!("fluent errors for {ln:?}: {parse_errors:?}, {fluent_errors:?}"),
				lvl: Level::Error,
			},
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::resources::ftl_server::Locale;
	use common::{
		assert_count,
		test_tools::utils::{SingleThreadedApp, new_handle},
	};
	use fluent::FluentError;
	use unic_langid::langid;

	#[derive(Resource)]
	struct _FtlServer(Locale);

	impl CurrentLocaleMut for _FtlServer {
		fn current_locale_mut(&mut self) -> &mut Locale {
			&mut self.0
		}
	}

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(Vec<Result<(), SetBundleError>>);

	fn setup<const N_FILES: usize, const N_FOLDERS: usize>(
		added: [(AssetEvent<Ftl>, Option<Ftl>); N_FILES],
		folders: [(Handle<LoadedFolder>, Vec<UntypedHandle>); N_FOLDERS],
	) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut file_assets = Assets::default();
		let mut folder_assets = Assets::default();

		for (handle, handles) in folders {
			folder_assets.insert(&handle, LoadedFolder { handles });
		}

		app.add_event::<AssetEvent<Ftl>>();
		for (event, ftl) in added {
			let id = match event {
				AssetEvent::Added { id } => id,
				AssetEvent::Modified { id } => id,
				AssetEvent::Removed { id } => id,
				AssetEvent::Unused { id } => id,
				AssetEvent::LoadedWithDependencies { id } => id,
			};
			if let Some(ftl) = ftl {
				file_assets.insert(id, ftl);
			}
			app.world_mut().send_event(event);
		}
		app.insert_resource(file_assets);
		app.insert_resource(folder_assets);
		app.add_systems(
			Update,
			_FtlServer::update_ftl_bundle.pipe(|In(result), mut commands: Commands| {
				commands.insert_resource(_Result(result))
			}),
		);

		app
	}

	#[derive(Debug, PartialEq)]
	enum _Error {
		NoBundle,
		NoMsg,
		NoPattern,
		Errors(Vec<FluentError>),
	}

	fn get_localization(locale: &Locale, token: &str) -> Result<String, _Error> {
		let Some(bundle) = &locale.bundle else {
			return Err(_Error::NoBundle);
		};
		let Some(msg) = bundle.get_message(token) else {
			return Err(_Error::NoMsg);
		};
		let Some(pattern) = msg.value() else {
			return Err(_Error::NoPattern);
		};
		let mut errors = vec![];
		let localized = bundle.format_pattern(pattern, None, &mut errors);
		if !errors.is_empty() {
			return Err(_Error::Errors(errors));
		}

		Ok(localized.to_string())
	}

	#[test]
	fn set_bundle() {
		let file = new_handle();
		let mut app = setup(
			[(
				AssetEvent::Added { id: file.id() },
				Some(Ftl(String::from("hello-world = Hello, World!"))),
			)],
			[],
		);
		app.insert_resource(_FtlServer(Locale {
			ln: langid!("en"),
			file: Some(file),
			folder: None,
			bundle: None,
		}));

		app.update();

		let locale = &app.world().resource::<_FtlServer>().0;
		assert_eq!(
			Ok(String::from("Hello, World!")),
			get_localization(locale, "hello-world")
		);
	}

	#[test]
	fn set_multiple_bundles() {
		let files = [new_handle(), new_handle()];
		let folders = [(new_handle(), vec![files[1].clone().untyped()])];
		let mut app = setup(
			[
				(
					AssetEvent::Added { id: files[0].id() },
					Some(Ftl(String::from("hello-world = Hello, World!"))),
				),
				(
					AssetEvent::Added { id: files[1].id() },
					Some(Ftl(String::from("bye-world = Bye, World!"))),
				),
			],
			folders.clone(),
		);
		app.insert_resource(_FtlServer(Locale {
			ln: langid!("en"),
			file: Some(files[0].clone()),
			folder: Some(folders[0].0.clone()),
			bundle: None,
		}));

		app.update();

		let locale = &app.world().resource::<_FtlServer>().0;
		assert_eq!(
			(
				Ok(String::from("Hello, World!")),
				Ok(String::from("Bye, World!")),
			),
			(
				get_localization(locale, "hello-world"),
				get_localization(locale, "bye-world"),
			)
		);
	}

	#[test]
	fn remove_file_handle_for_added_bundle() {
		let file = new_handle();
		let mut app = setup(
			[(
				AssetEvent::Added { id: file.id() },
				Some(Ftl(String::from("hello-world = Hello, World!"))),
			)],
			[],
		);
		app.insert_resource(_FtlServer(Locale {
			ln: langid!("en"),
			file: Some(file),
			folder: None,
			bundle: None,
		}));

		app.update();

		let locale = &app.world().resource::<_FtlServer>().0;
		assert!(locale.file.is_none());
	}

	#[test]
	fn remove_file_handles_for_added_bundle_in_folder() {
		let files = [new_handle(), new_handle()];
		let folders = [(
			new_handle(),
			files.iter().map(|f| f.clone().untyped()).collect(),
		)];
		let mut app = setup(
			[(
				AssetEvent::Added { id: files[1].id() },
				Some(Ftl(String::from("hello-world = Hello, World!"))),
			)],
			folders.clone(),
		);
		app.insert_resource(_FtlServer(Locale {
			ln: langid!("en"),
			file: None,
			folder: Some(folders[0].0.clone()),
			bundle: None,
		}));

		app.update();

		let folder = &app
			.world()
			.resource::<Assets<LoadedFolder>>()
			.get(&folders[0].0)
			.unwrap();
		assert_eq!(vec![files[0].clone()], folder.handles);
	}

	#[test]
	fn remove_folder_handle_if_no_files_left() {
		let files = [new_handle(), new_handle()];
		let folders = [(
			new_handle(),
			files.iter().map(|f| f.clone().untyped()).collect(),
		)];
		let mut app = setup(
			[
				(
					AssetEvent::Added { id: files[0].id() },
					Some(Ftl(String::from("hello-world = Hello, World!"))),
				),
				(
					AssetEvent::Added { id: files[1].id() },
					Some(Ftl(String::from("bye-world = Bye, World!"))),
				),
			],
			folders.clone(),
		);
		app.insert_resource(_FtlServer(Locale {
			ln: langid!("en"),
			file: None,
			folder: Some(folders[0].0.clone()),
			bundle: None,
		}));

		app.update();

		let ftl_server = &app.world().resource::<_FtlServer>();
		assert_eq!(None, ftl_server.0.folder);
	}

	#[test]
	fn do_nothing_if_not_added() {
		let file = new_handle();
		let mut app = setup(
			[
				(
					AssetEvent::LoadedWithDependencies { id: file.id() },
					Some(Ftl(String::from("hello-world = Hello, World!"))),
				),
				(
					AssetEvent::Modified { id: file.id() },
					Some(Ftl(String::from("hello-world = Hello, World!"))),
				),
				(
					AssetEvent::Removed { id: file.id() },
					Some(Ftl(String::from("hello-world = Hello, World!"))),
				),
				(
					AssetEvent::Unused { id: file.id() },
					Some(Ftl(String::from("hello-world = Hello, World!"))),
				),
			],
			[],
		);
		app.insert_resource(_FtlServer(Locale {
			ln: langid!("en"),
			file: Some(file),
			folder: None,
			bundle: None,
		}));

		app.update();

		let locale = &app.world().resource::<_FtlServer>().0;
		assert!(locale.bundle.is_none());
	}

	#[test]
	fn ignore_not_matching_handles() {
		let files = [new_handle(), new_handle(), new_handle(), new_handle()];
		let folders = [(
			new_handle(),
			files[1..=2].iter().map(|f| f.clone().untyped()).collect(),
		)];
		let mut app = setup(
			[
				(
					AssetEvent::Added { id: files[0].id() },
					Some(Ftl(String::from("a = A!"))),
				),
				(
					AssetEvent::Added { id: files[1].id() },
					Some(Ftl(String::from("b = B!"))),
				),
				(
					AssetEvent::Added { id: files[2].id() },
					Some(Ftl(String::from("c = C!"))),
				),
				(
					AssetEvent::Added { id: files[3].id() },
					Some(Ftl(String::from("d = D!"))),
				),
			],
			folders.clone(),
		);
		app.insert_resource(_FtlServer(Locale {
			ln: langid!("en"),
			file: Some(new_handle()),
			folder: Some(folders[0].0.clone()),
			bundle: None,
		}));

		app.update();

		let locale = &app.world().resource::<_FtlServer>().0;
		assert_eq!(
			(
				Err(_Error::NoMsg),
				Ok(String::from("B!")),
				Ok(String::from("C!")),
				Err(_Error::NoMsg),
			),
			(
				get_localization(locale, "a"),
				get_localization(locale, "b"),
				get_localization(locale, "c"),
				get_localization(locale, "d"),
			)
		);
	}

	#[test]
	fn no_ftl_asset_error() {
		let file = new_handle();
		let mut app = setup([(AssetEvent::Added { id: file.id() }, None)], []);
		app.insert_resource(_FtlServer(Locale {
			ln: langid!("en"),
			file: Some(file),
			folder: None,
			bundle: None,
		}));

		app.update();

		assert_eq!(
			&_Result(vec![Err(SetBundleError {
				ln: langid!("en"),
				kind: SetBundleErrorKind::NoFtlFile
			})]),
			app.world().resource::<_Result>()
		);
	}

	#[test]
	fn parse_error() {
		let file = new_handle();
		let mut app = setup(
			[(
				AssetEvent::Added { id: file.id() },
				Some(Ftl(String::from("hello-world ? Hello, World!"))),
			)],
			[],
		);
		app.insert_resource(_FtlServer(Locale {
			ln: langid!("en"),
			file: Some(file),
			folder: None,
			bundle: None,
		}));

		app.update();

		assert_eq!(
			&_Result(vec![Err(SetBundleError {
				ln: langid!("en"),
				kind: SetBundleErrorKind::FluentError(
					vec![ParserError {
						pos: 12..13,
						slice: Some(0..27),
						kind: fluent_syntax::parser::ErrorKind::ExpectedToken('='),
					}],
					vec![]
				)
			})]),
			app.world().resource::<_Result>()
		);
	}

	#[test]
	fn still_add_bundle_when_parse_error() {
		let file = new_handle();
		let mut app = setup(
			[(
				AssetEvent::Added { id: file.id() },
				Some(Ftl(String::from(
					"other = Other!\nhello-world ? Hello, World!",
				))),
			)],
			[],
		);
		app.insert_resource(_FtlServer(Locale {
			ln: langid!("en"),
			file: Some(file),
			folder: None,
			bundle: None,
		}));

		app.update();

		let locale = &app.world().resource::<_FtlServer>().0;
		assert_eq!(
			Ok(String::from("Other!")),
			get_localization(locale, "other")
		);
	}

	macro_rules! assert_singular_override_error {
		($result:expr, $lang_id:expr, $index:expr) => {
			let error = match $result {
				Err(error) => error,
				Ok(_) => panic!("NO ERROR"),
			};
			let (ln, errors) = match error {
				SetBundleError {
					ln,
					kind: SetBundleErrorKind::FluentError(_, errors),
				} => (ln, errors),
				_ => panic!("WRONG ERROR KIND"),
			};
			let [error, ..] = assert_count!(1, errors.iter());
			assert_eq!(ln, &$lang_id);
			assert!(matches!(
				error,
				FluentError::Overriding {
					id,
					.. // We cannot import EntryKind, this is the reason for the painful step by step unpacking
				} if id == &String::from($index)
			))
		};
	}

	#[test]
	fn fluent_error() {
		let files = [new_handle(), new_handle()];
		let folders = [(new_handle(), vec![files[1].clone().untyped()])];
		let mut app = setup(
			[
				(
					AssetEvent::Added { id: files[0].id() },
					Some(Ftl(String::from("hello-world = Hello, World!"))),
				),
				(
					AssetEvent::Added { id: files[1].id() },
					Some(Ftl(String::from("hello-world = Hello, Override!"))),
				),
			],
			folders.clone(),
		);
		app.insert_resource(_FtlServer(Locale {
			ln: langid!("en"),
			file: Some(files[0].clone()),
			folder: Some(folders[0].0.clone()),
			bundle: None,
		}));

		app.update();

		let result = app.world().resource::<_Result>();
		let [_, result] = assert_count!(2, result.0.iter());
		assert_singular_override_error!(result, langid!("en"), "hello-world");
	}

	#[test]
	fn still_add_bundle_when_fluent_error() {
		let files = [new_handle(), new_handle()];
		let folders = [(new_handle(), vec![files[1].clone().untyped()])];
		let mut app = setup(
			[
				(
					AssetEvent::Added { id: files[0].id() },
					Some(Ftl(String::from(
						"hello-world = Hello, World!\nother-1 = Other1!",
					))),
				),
				(
					AssetEvent::Added { id: files[1].id() },
					Some(Ftl(String::from(
						"hello-world = Hello, Override!\nother-2 = Other2!",
					))),
				),
			],
			folders.clone(),
		);
		app.insert_resource(_FtlServer(Locale {
			ln: langid!("en"),
			file: Some(files[0].clone()),
			folder: Some(folders[0].0.clone()),
			bundle: None,
		}));

		app.update();

		let locale = &app.world().resource::<_FtlServer>().0;
		assert_eq!(
			(Ok(String::from("Other1!")), Ok(String::from("Other2!")),),
			(
				get_localization(locale, "other-1"),
				get_localization(locale, "other-2"),
			)
		);
	}
}
