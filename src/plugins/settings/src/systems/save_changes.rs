use crate::{
	resources::{
		asset_writer::{AssetWriter, WriteAsset, WriteError},
		key_map::InvalidInputWarning,
	},
	traits::drain_invalid_inputs::DrainInvalidInputs,
};
use bevy::prelude::*;
use common::{
	errors::{ErrorData, Level},
	tools::action_key::{ActionKey, user_input::UserInput},
	traits::load_asset::Path,
};
use serde::Serialize;
use std::{
	collections::{HashMap, HashSet},
	fmt::Display,
};

impl<T> SaveChanges for T where
	T: Resource + Clone + DrainInvalidInputs<TInvalidInput = (ActionKey, HashSet<UserInput>)>
{
}

pub(crate) trait SaveChanges:
	Resource + Clone + Sized + DrainInvalidInputs<TInvalidInput = (ActionKey, HashSet<UserInput>)>
{
	fn save_changes<TDto>(
		path: Path,
	) -> impl Fn(ResMut<Self>, Res<AssetWriter>) -> Result<(), SaveError>
	where
		TDto: Serialize + From<Self> + 'static,
	{
		save_changes::<Self, TDto, AssetWriter>(path)
	}
}

fn save_changes<TAsset, TDto, TWriter>(
	path: Path,
) -> impl Fn(ResMut<TAsset>, Res<TWriter>) -> Result<(), SaveError<TWriter::TError>>
where
	TAsset: Resource + Clone + DrainInvalidInputs<TInvalidInput = (ActionKey, HashSet<UserInput>)>,
	TDto: Serialize + From<TAsset> + 'static,
	TWriter: WriteAsset + Resource,
{
	move |mut resource, writer| {
		if resource.is_added() {
			return invalid_input_or_ok(resource.as_mut());
		}
		if !resource.is_changed() {
			return Ok(());
		}

		let dto = TDto::from(resource.clone());
		if let Err(err) = writer.write(dto, path.clone()) {
			return Err(SaveError::Writer(err));
		}

		invalid_input_or_ok(resource.as_mut())
	}
}

fn invalid_input_or_ok<TAsset, TError>(resource: &mut TAsset) -> Result<(), SaveError<TError>>
where
	TAsset: DrainInvalidInputs<TInvalidInput = (ActionKey, HashSet<UserInput>)>,
{
	let errors = resource.drain_invalid_inputs().collect::<HashMap<_, _>>();
	if !errors.is_empty() {
		return Err(SaveError::InvalidInput(InvalidInputWarning(errors)));
	}

	Ok(())
}

#[derive(Debug, PartialEq)]
pub(crate) enum SaveError<TWriteError = WriteError> {
	Writer(TWriteError),
	InvalidInput(InvalidInputWarning<ActionKey, UserInput>),
}

impl Display for SaveError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			SaveError::Writer(error) => write!(f, "{error}"),
			SaveError::InvalidInput(warning) => write!(f, "{warning}"),
		}
	}
}

impl ErrorData for SaveError {
	type TContext = Self;

	fn level(&self) -> Level {
		match self {
			SaveError::Writer(error) => error.level(),
			SaveError::InvalidInput(warning) => warning.level(),
		}
	}

	fn label() -> String {
		"Save error".to_owned()
	}

	fn context(&self) -> &Self::TContext {
		self
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::states::menu_state::MenuState;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Resource, Debug, PartialEq, Serialize, Clone, Default)]
	struct _Resource {
		invalid_inputs: Vec<(ActionKey, HashSet<UserInput>)>,
	}

	impl DrainInvalidInputs for _Resource {
		type TInvalidInput = (ActionKey, HashSet<UserInput>);

		fn drain_invalid_inputs(&mut self) -> impl Iterator<Item = Self::TInvalidInput> {
			// we fake the drain, so we do not have to repopulate this between frames
			self.invalid_inputs.iter().cloned()
		}
	}

	#[derive(Asset, TypePath, Debug, PartialEq, Serialize)]
	struct _ResourceDto(_Resource);

	impl From<_Resource> for _ResourceDto {
		fn from(resource: _Resource) -> Self {
			Self(resource)
		}
	}

	#[derive(Resource, NestedMocks)]
	struct _Writer {
		mock: Mock_Writer,
	}

	#[automock]
	impl WriteAsset for _Writer {
		type TError = _Error;

		fn write<TAsset>(&self, asset: TAsset, path: Path) -> Result<(), _Error>
		where
			TAsset: Serialize + 'static,
		{
			self.mock.write(asset, path)
		}
	}

	fn update_with_change(app: &mut App) {
		app.update();
		app.world_mut()
			.get_resource_mut::<_Resource>()
			.as_deref_mut();
		app.update();
	}

	fn update_without_change(app: &mut App) {
		app.update();
	}

	#[derive(Debug, PartialEq, Clone)]
	pub struct _Error;

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(Result<(), SaveError<_Error>>);

	fn setup(writer: _Writer, asset: _Resource, path: Path) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			save_changes::<_Resource, _ResourceDto, _Writer>(path).pipe(
				|In(r), mut commands: Commands| {
					commands.insert_resource(_Result(r));
				},
			),
		);
		app.insert_resource(writer);
		app.insert_resource(asset);

		app
	}

	#[test]
	fn call_writer() {
		let writer = _Writer::new().with_mock(|mock| {
			mock.expect_write()
				.times(1)
				.with(
					eq(_ResourceDto(_Resource::default())),
					eq(Path::from("my/path")),
				)
				.return_const(Ok(()));
		});
		let mut app = setup(writer, _Resource::default(), Path::from("my/path"));

		update_with_change(&mut app);
	}

	#[test]
	fn return_result() {
		let writer = _Writer::new().with_mock(|mock| {
			mock.expect_write::<_ResourceDto>()
				.return_const(Err(_Error));
		});
		let mut app = setup(writer, _Resource::default(), Path::from("my/path"));

		update_with_change(&mut app);

		assert_eq!(
			&_Result(Err(SaveError::Writer(_Error))),
			app.world().resource::<_Result>()
		);
	}

	#[test]
	fn do_not_call_writer_if_not_changed() {
		let writer = _Writer::new().with_mock(|mock| {
			mock.expect_write::<_ResourceDto>()
				.never()
				.return_const(Ok(()));
		});
		let mut app = setup(writer, _Resource::default(), Path::from("my/path"));

		update_without_change(&mut app);
	}

	#[test]
	fn return_invalid_inputs() {
		let writer = _Writer::new().with_mock(|mock| {
			mock.expect_write::<_ResourceDto>().return_const(Ok(()));
		});
		let mut app = setup(
			writer,
			_Resource {
				invalid_inputs: Vec::from([(
					ActionKey::Menu(MenuState::Inventory),
					HashSet::from([UserInput::from(MouseButton::Left)]),
				)]),
			},
			Path::from("my/path"),
		);

		update_with_change(&mut app);

		assert_eq!(
			&_Result(Err(SaveError::InvalidInput(InvalidInputWarning::from([(
				ActionKey::Menu(MenuState::Inventory),
				HashSet::from([UserInput::from(MouseButton::Left)]),
			)])))),
			app.world().resource::<_Result>()
		);
	}

	#[test]
	fn return_invalid_inputs_even_when_added() {
		let writer = _Writer::new().with_mock(|mock| {
			mock.expect_write::<_ResourceDto>().return_const(Ok(()));
		});
		let mut app = setup(
			writer,
			_Resource {
				invalid_inputs: Vec::from([(
					ActionKey::Menu(MenuState::Inventory),
					HashSet::from([UserInput::from(MouseButton::Left)]),
				)]),
			},
			Path::from("my/path"),
		);

		update_without_change(&mut app);

		assert_eq!(
			&_Result(Err(SaveError::InvalidInput(InvalidInputWarning::from([(
				ActionKey::Menu(MenuState::Inventory),
				HashSet::from([UserInput::from(MouseButton::Left)]),
			)])))),
			app.world().resource::<_Result>()
		);
	}
}
