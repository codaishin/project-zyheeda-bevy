pub(crate) mod dto;

use crate::traits::drain_invalid_inputs::DrainInvalidInputs;
use bevy::prelude::*;
use common::{
	errors::{ErrorData, Level},
	tools::action_key::{ActionKey, user_input::UserInput},
	traits::{
		handles_custom_assets::TryLoadFrom,
		handles_input::{GetInput, InvalidUserInput, UpdateKey},
		iterate::Iterate,
		iteration::IterFinite,
		key_mappings::{HashCopySafe, TryGetAction},
		load_asset::LoadAsset,
	},
};
use dto::KeyMapDto;
use std::{
	collections::{
		HashMap,
		HashSet,
		hash_map::{Entry, Iter},
	},
	error::Error as StdError,
	fmt::{Debug, Display},
	hash::Hash,
};
use zyheeda_core::prelude::*;

#[derive(Resource, Asset, TypePath, Debug, PartialEq, Clone)]
pub struct KeyMap(KeyMapInternal);

impl GetInput for KeyMap {
	fn get_input<TAction>(&self, action: TAction) -> UserInput
	where
		TAction: Into<ActionKey>,
	{
		self.0.get_input(action)
	}
}

impl TryGetAction for KeyMap {
	fn try_get_action<TAction>(&self, input: UserInput) -> Option<TAction>
	where
		TAction: Copy + TryFrom<ActionKey>,
	{
		self.0.try_get_action(input)
	}
}

impl UpdateKey for KeyMap {
	fn update_key<TAction>(&mut self, action: TAction, input: UserInput)
	where
		TAction: Copy + Into<ActionKey>,
	{
		self.0.update_key(action, input);
	}
}

impl TryLoadFrom<KeyMapDto<ActionKey>> for KeyMap {
	type TInstantiationError = LoadError<ActionKey>;

	fn try_load_from<TLoadAsset>(
		dto: KeyMapDto<ActionKey>,
		asset_server: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError>
	where
		TLoadAsset: LoadAsset,
	{
		KeyMapInternal::try_load_from(dto, asset_server).map(KeyMap)
	}
}

impl DrainInvalidInputs for KeyMap {
	type TInvalidInput = (ActionKey, HashSet<UserInput>);

	fn drain_invalid_inputs(&mut self) -> impl Iterator<Item = Self::TInvalidInput> {
		self.0.invalid_inputs.0.drain()
	}
}

#[derive(Debug, PartialEq, Clone)]
struct InvalidInputs<TAction, TInput>(HashMap<TAction, HashSet<TInput>>)
where
	TAction: Eq + Hash,
	TInput: Eq + Hash;

impl<TAction, TInput> InvalidInputs<TAction, TInput>
where
	TAction: Eq + Hash,
	TInput: Eq + Hash,
{
	fn push(&mut self, action: TAction, input: TInput) {
		match self.0.entry(action) {
			Entry::Occupied(mut entry) => {
				entry.get_mut().insert(input);
			}
			Entry::Vacant(entry) => {
				entry.insert(HashSet::from([input]));
			}
		};
	}
}

#[derive(Debug, PartialEq, Clone)]
struct KeyMapInternal<TAllActions = ActionKey>
where
	TAllActions: Hash + Eq,
{
	action_to_input: HashMap<TAllActions, UserInput>,
	input_to_action: HashMap<UserInput, TAllActions>,
	invalid_inputs: InvalidInputs<TAllActions, UserInput>,
}

impl<TAllActions> KeyMapInternal<TAllActions>
where
	TAllActions: Hash + Eq + Copy + InvalidUserInput + Into<UserInput>,
{
	fn update_key<TAction>(&mut self, action: TAction, input: UserInput)
	where
		TAction: Copy + Into<TAllActions>,
	{
		let action: TAllActions = action.into();
		let old_input = self.get_input(action);

		if self.input_to_action.get(&input) == Some(&action) {
			return;
		}

		if action.invalid_input().contains(&input) {
			self.invalid_inputs.push(action, input);
			return;
		}

		match self.input_to_action.get(&input).copied() {
			Some(old_action) => {
				if old_action.invalid_input().contains(&old_input) {
					self.invalid_inputs.push(old_action, old_input);
					return;
				}
				self.action_to_input.insert(old_action, old_input);
				self.input_to_action.insert(old_input, old_action);
			}
			None => {
				self.input_to_action.remove(&old_input);
			}
		}

		self.action_to_input.insert(action, input);
		self.input_to_action.insert(input, action);
	}

	fn try_get_action<TAction>(&self, input: UserInput) -> Option<TAction>
	where
		TAction: TryFrom<TAllActions>,
	{
		let action = self.input_to_action.get(&input)?;
		TAction::try_from(*action).ok()
	}

	fn get_input<TAction>(&self, action: TAction) -> UserInput
	where
		TAction: Into<TAllActions>,
	{
		let action: TAllActions = action.into();
		let Some(input) = self.action_to_input.get(&action) else {
			let as_input: UserInput = action.into();
			return as_input;
		};

		*input
	}
}

impl<TAction> Default for KeyMapInternal<TAction>
where
	TAction: Copy + Hash + Eq + IterFinite + InvalidUserInput + Into<UserInput>,
{
	fn default() -> Self {
		let mut map = Self {
			action_to_input: HashMap::default(),
			input_to_action: HashMap::default(),
			invalid_inputs: InvalidInputs(HashMap::default()),
		};

		for key in TAction::iterator() {
			map.update_key(key, key.into());
		}

		map
	}
}

impl<TAction> TryLoadFrom<KeyMapDto<TAction>> for KeyMapInternal<TAction>
where
	TAction: Debug + HashCopySafe + InvalidUserInput + IterFinite + TypePath + Into<UserInput>,
{
	type TInstantiationError = LoadError<TAction>;

	fn try_load_from<TLoadAsset>(
		KeyMapDto { actions }: KeyMapDto<TAction>,
		_: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError>
	where
		TLoadAsset: LoadAsset,
	{
		let mut mapper = Self::default();
		for (action, input) in actions {
			mapper.update_key(action, input);
		}

		if let Some(error) = LoadError::from_mapper(&mapper) {
			return Err(error);
		}

		Ok(mapper)
	}
}

#[derive(TypePath, Debug, PartialEq, Clone)]
pub struct InvalidInputWarning<TAction>(pub(crate) HashMap<TAction, HashSet<UserInput>>)
where
	TAction: Eq + Hash;

impl<TAction> InvalidInputWarning<TAction>
where
	TAction: InvalidUserInput + Debug + Eq + Hash,
{
	fn iter(&self) -> impl Iterator<Item = InvalidInputWarningItem<'_, TAction>> {
		self.0
			.iter()
			.map(|(action, inputs)| InvalidInputWarningItem { action, inputs })
	}
}

impl<T, TAction> From<T> for InvalidInputWarning<TAction>
where
	T: IntoIterator<Item = (TAction, HashSet<UserInput>)>,
	TAction: Debug + Eq + Hash,
{
	fn from(value: T) -> Self {
		Self(HashMap::from_iter(value))
	}
}

impl<TAction> Display for InvalidInputWarning<TAction>
where
	TAction: InvalidUserInput + Debug + Eq + Hash,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write_iter!(f, "Attempted to set invalid inputs: ", self)
	}
}

impl<TAction> ErrorData for InvalidInputWarning<TAction>
where
	TAction: InvalidUserInput + Debug + Eq + Hash,
{
	fn level(&self) -> Level {
		Level::Warning
	}

	fn label() -> impl Display {
		"Tried to set invalid input"
	}

	fn into_details(self) -> impl Display {
		self
	}
}

struct InvalidInputWarningItem<'a, TAction> {
	action: &'a TAction,
	inputs: &'a HashSet<UserInput>,
}

impl<TAction> Display for InvalidInputWarningItem<'_, TAction>
where
	TAction: InvalidUserInput + Debug + Eq + Hash,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"Tried to set {:?} to: {:?} (invalid inputs: {:?})",
			self.action,
			self.inputs,
			self.action.invalid_input()
		)
	}
}

#[derive(TypePath, Debug, PartialEq)]
pub enum LoadError<TAllActions>
where
	TAllActions: Debug + Eq + Hash + TypePath,
{
	RepeatedInputs(HashMap<UserInput, HashSet<TAllActions>>),
	MissingInputs(HashSet<TAllActions>),
}

impl<TAllActions> LoadError<TAllActions>
where
	TAllActions: IterFinite + InvalidUserInput + Debug + Eq + Hash + TypePath + Copy,
{
	fn from_mapper(mapper: &KeyMapInternal<TAllActions>) -> Option<Self> {
		let mut repeated = HashMap::<UserInput, HashSet<TAllActions>>::default();

		for (action, input) in &mapper.action_to_input {
			match repeated.entry(*input) {
				Entry::Occupied(mut entry) => {
					entry.get_mut().insert(*action);
				}
				Entry::Vacant(entry) => {
					entry.insert(HashSet::from([*action]));
				}
			}
		}

		repeated.retain(|_, keys| keys.len() > 1);

		if !repeated.is_empty() {
			return Some(Self::RepeatedInputs(repeated));
		}

		let mut incomplete = HashSet::<TAllActions>::default();

		for action in TAllActions::iterator() {
			if mapper.action_to_input.contains_key(&action) {
				continue;
			}

			incomplete.insert(action);
		}

		if !incomplete.is_empty() {
			return Some(Self::MissingInputs(incomplete));
		}

		None
	}
}

impl<TAction> Display for LoadError<TAction>
where
	TAction: Debug + Eq + Hash + TypePath + InvalidUserInput,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			LoadError::RepeatedInputs(repeated) => {
				let actions = repeated
					.iter()
					.map(|(input, actions)| format!("  - {input:?} is assigned to: {actions:?}"))
					.collect::<Vec<_>>()
					.join("\n");
				writeln!(
					f,
					"Multiple actions assigned to the same input(s):\n{actions}"
				)
			}
			LoadError::MissingInputs(missing) => {
				let actions = missing
					.iter()
					.map(|action| {
						let invalid_actions = action.invalid_input();
						if invalid_actions.is_empty() {
							format!("  - {action:?} has no input")
						} else {
							format!(
								"  - {action:?} has no input. Either missing or part of invalid inputs for this action: {invalid_actions:?}"
							)
						}
					})
					.collect::<Vec<_>>()
					.join("\n");
				writeln!(f, "Some actions have no input:\n{actions}")
			}
		}
	}
}

impl<TAction> StdError for LoadError<TAction> where
	TAction: Debug + Eq + Hash + TypePath + InvalidUserInput
{
}

impl<'a> Iterate<'a> for KeyMap {
	type TItem = (&'a ActionKey, &'a UserInput);
	type TIter = Iter<'a, ActionKey, UserInput>;

	fn iterate(&'a self) -> Self::TIter {
		self.0.action_to_input.iter()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::iteration::{Iter, IterFinite};

	#[derive(TypePath, Debug, PartialEq, Eq, Hash, Clone, Copy)]
	enum _AllActions {
		A,
		B,
	}

	impl From<_AllActions> for UserInput {
		fn from(action: _AllActions) -> Self {
			match action {
				_AllActions::A => UserInput::KeyCode(KeyCode::KeyA),
				_AllActions::B => UserInput::KeyCode(KeyCode::KeyB),
			}
		}
	}

	impl IterFinite for _AllActions {
		fn iterator() -> Iter<Self> {
			Iter(Some(_AllActions::A))
		}

		fn next(current: &Iter<Self>) -> Option<Self> {
			match current.0? {
				_AllActions::A => Some(_AllActions::B),
				_AllActions::B => None,
			}
		}
	}

	impl InvalidUserInput for _AllActions {
		fn invalid_input(&self) -> &[UserInput] {
			match self {
				_AllActions::A => _ActionA.invalid_input(),
				_AllActions::B => _ActionB.invalid_input(),
			}
		}
	}

	#[derive(TypePath, Debug, PartialEq, Eq, Hash, Clone, Copy)]
	struct _ActionA;

	impl From<_ActionA> for _AllActions {
		fn from(_: _ActionA) -> Self {
			Self::A
		}
	}

	impl TryFrom<_AllActions> for _ActionA {
		type Error = ();

		fn try_from(action: _AllActions) -> Result<Self, Self::Error> {
			match action {
				_AllActions::A => Ok(_ActionA),
				_ => Err(()),
			}
		}
	}

	impl InvalidUserInput for _ActionA {
		fn invalid_input(&self) -> &[UserInput] {
			&[]
		}
	}

	#[derive(TypePath, Debug, PartialEq, Eq, Hash, Clone, Copy)]
	struct _ActionB;

	impl From<_ActionB> for _AllActions {
		fn from(_: _ActionB) -> Self {
			Self::B
		}
	}

	impl TryFrom<_AllActions> for _ActionB {
		type Error = ();

		fn try_from(action: _AllActions) -> Result<Self, Self::Error> {
			match action {
				_AllActions::B => Ok(_ActionB),
				_ => Err(()),
			}
		}
	}

	impl InvalidUserInput for _ActionB {
		fn invalid_input(&self) -> &[UserInput] {
			&[UserInput::KeyCode(KeyCode::KeyC)]
		}
	}

	mod map {
		use super::*;

		#[test]
		fn to_input() {
			let mapper = KeyMapInternal::<_AllActions>::default();
			let mapped = mapper.get_input(_ActionB);

			assert_eq!(UserInput::KeyCode(KeyCode::KeyB), mapped);
		}

		#[test]
		fn to_key_a() {
			let mapper = KeyMapInternal::<_AllActions>::default();
			let mapped = mapper.try_get_action(UserInput::KeyCode(KeyCode::KeyA));

			assert_eq!(Some(_ActionA), mapped);
		}

		#[test]
		fn to_key_b() {
			let mapper = KeyMapInternal::<_AllActions>::default();
			let mapped = mapper.try_get_action(UserInput::KeyCode(KeyCode::KeyB));

			assert_eq!(Some(_ActionB), mapped);
		}
	}

	mod update {
		use super::*;

		#[test]
		fn key() {
			let action = _ActionA;
			let input = UserInput::KeyCode(KeyCode::KeyB);
			let mut mapper = KeyMapInternal::<_AllActions>::default();
			mapper.update_key(action, input);

			assert_eq!(
				(input, Some(action)),
				(mapper.get_input(action), mapper.try_get_action(input))
			);
		}

		#[test]
		fn key_removing_old_input_pairing() {
			let action = _ActionA;
			let input_b = UserInput::KeyCode(KeyCode::KeyB);
			let input_c = UserInput::KeyCode(KeyCode::KeyC);
			let mut mapper = KeyMapInternal::<_AllActions>::default();
			mapper.update_key(action, input_b);
			mapper.update_key(action, input_c);

			assert_eq!(
				(input_c, Some(action), None as Option<UserInput>),
				(
					mapper.get_input(action),
					mapper.try_get_action(input_c),
					mapper.try_get_action(input_b)
				)
			);
		}

		#[test]
		fn key_swapping_old_key() {
			let action_a = _ActionA;
			let action_b = _ActionB;
			let input_a = UserInput::KeyCode(KeyCode::KeyA);
			let input_b = UserInput::KeyCode(KeyCode::KeyB);
			let mut mapper = KeyMapInternal::<_AllActions>::default();
			mapper.update_key(action_a, input_a);
			mapper.update_key(action_b, input_a);

			assert_eq!(
				(input_b, Some(action_b), input_a, Some(input_a),),
				(
					mapper.get_input(action_a),
					mapper.try_get_action(input_a),
					mapper.get_input(action_b),
					mapper.try_get_action(input_b),
				)
			);
		}

		#[test]
		fn ignore_update_when_attempting_to_use_invalid_key() {
			let mut mapper = KeyMapInternal::<_AllActions>::default();
			mapper.update_key(_ActionB, UserInput::KeyCode(KeyCode::KeyC));

			assert_eq!(
				(
					UserInput::from(_AllActions::B),
					None as Option<_ActionA>,
					HashMap::from([(
						_AllActions::B,
						HashSet::from([UserInput::KeyCode(KeyCode::KeyC)])
					)])
				),
				(
					mapper.get_input(_ActionB),
					mapper.try_get_action(UserInput::KeyCode(KeyCode::KeyC)),
					mapper.invalid_inputs.0
				)
			);
		}

		#[test]
		fn ignore_update_when_swap_would_assign_other_action_with_invalid_key() {
			let mut mapper = KeyMapInternal::<_AllActions>::default();
			mapper.update_key(_ActionA, UserInput::KeyCode(KeyCode::KeyC));
			mapper.update_key(_ActionB, UserInput::KeyCode(KeyCode::KeyB));
			mapper.update_key(_ActionA, UserInput::KeyCode(KeyCode::KeyB));

			assert_eq!(
				(
					UserInput::KeyCode(KeyCode::KeyC),
					Some(_ActionA),
					UserInput::KeyCode(KeyCode::KeyB),
					Some(_ActionB),
					HashMap::from([(
						_AllActions::B,
						HashSet::from([UserInput::KeyCode(KeyCode::KeyC)])
					)])
				),
				(
					mapper.get_input(_ActionA),
					mapper.try_get_action(UserInput::KeyCode(KeyCode::KeyC)),
					mapper.get_input(_ActionB),
					mapper.try_get_action(UserInput::KeyCode(KeyCode::KeyB)),
					mapper.invalid_inputs.0
				)
			);
		}
	}

	mod try_load {
		use super::*;

		struct _Server;

		impl LoadAsset for _Server {
			fn load_asset<TAsset, TPath>(&mut self, _: TPath) -> Handle<TAsset>
			where
				TAsset: Asset,
			{
				panic!("NUT USED")
			}
		}

		#[test]
		fn from_dto() -> Result<(), LoadError<_AllActions>> {
			let dto = KeyMapDto::from([(_AllActions::A, UserInput::KeyCode(KeyCode::KeyC))]);

			let mapper = KeyMapInternal::try_load_from(dto, &mut _Server)?;

			assert_eq!(
				(
					UserInput::KeyCode(KeyCode::KeyC),
					Some(_ActionA),
					UserInput::KeyCode(KeyCode::KeyB),
					Some(_ActionB)
				),
				(
					mapper.get_input(_ActionA),
					mapper.try_get_action(UserInput::KeyCode(KeyCode::KeyC)),
					mapper.get_input(_ActionB),
					mapper.try_get_action(UserInput::KeyCode(KeyCode::KeyB)),
				)
			);
			Ok(())
		}

		mod double_inputs {
			use super::*;

			#[derive(TypePath, Debug, PartialEq, Eq, Hash, Clone, Copy)]
			enum _FaultyAction {
				A,
				B,
				C,
			}

			impl From<_FaultyAction> for UserInput {
				fn from(value: _FaultyAction) -> Self {
					match value {
						_FaultyAction::A => UserInput::KeyCode(KeyCode::KeyA),
						_FaultyAction::B => UserInput::KeyCode(KeyCode::KeyC), // this is the faulty mapping
						_FaultyAction::C => UserInput::KeyCode(KeyCode::KeyC),
					}
				}
			}

			impl IterFinite for _FaultyAction {
				fn iterator() -> Iter<Self> {
					Iter(Some(_FaultyAction::A))
				}

				fn next(current: &Iter<Self>) -> Option<Self> {
					match current.0? {
						_FaultyAction::A => Some(_FaultyAction::B),
						_FaultyAction::B => Some(_FaultyAction::C),
						_FaultyAction::C => None,
					}
				}
			}

			impl InvalidUserInput for _FaultyAction {
				fn invalid_input(&self) -> &[UserInput] {
					&[]
				}
			}

			#[test]
			fn from_dto_error() {
				let mapper = KeyMapInternal::try_load_from(KeyMapDto::from([]), &mut _Server);

				assert_eq!(
					Err(LoadError::RepeatedInputs(HashMap::from([(
						UserInput::KeyCode(KeyCode::KeyC),
						HashSet::from([_FaultyAction::B, _FaultyAction::C])
					)]))),
					mapper
				);
			}
		}

		mod missing_inputs {
			use super::*;

			#[derive(TypePath, Debug, PartialEq, Eq, Hash, Clone, Copy)]
			enum _FaultyAction {
				A,
				B,
				C,
			}

			impl From<_FaultyAction> for UserInput {
				fn from(value: _FaultyAction) -> Self {
					match value {
						_FaultyAction::A => UserInput::KeyCode(KeyCode::KeyA),
						_FaultyAction::B => UserInput::KeyCode(KeyCode::KeyB),
						_FaultyAction::C => UserInput::KeyCode(KeyCode::KeyC),
					}
				}
			}

			impl IterFinite for _FaultyAction {
				fn iterator() -> Iter<Self> {
					Iter(Some(_FaultyAction::A))
				}

				fn next(current: &Iter<Self>) -> Option<Self> {
					match current.0? {
						_FaultyAction::A => Some(_FaultyAction::B),
						_FaultyAction::B => Some(_FaultyAction::C),
						_FaultyAction::C => None,
					}
				}
			}

			impl InvalidUserInput for _FaultyAction {
				fn invalid_input(&self) -> &[UserInput] {
					&[UserInput::KeyCode(KeyCode::KeyB)] // input invalid, thus no input present for action
				}
			}

			#[test]
			fn from_dto_error() {
				let mapper = KeyMapInternal::try_load_from(KeyMapDto::from([]), &mut _Server);

				assert_eq!(
					Err(LoadError::MissingInputs(HashSet::from([_FaultyAction::B]))),
					mapper
				);
			}
		}
	}

	mod load_error {
		use super::*;
		use testing::assert_count;

		macro_rules! either_or {
			($a:expr, $b:expr $(,)?) => {
				$a || $b
			};
		}

		#[test]
		fn display_repeated_inputs() {
			let repeated = LoadError::RepeatedInputs(HashMap::from([
				(
					UserInput::KeyCode(KeyCode::KeyC),
					HashSet::from([_AllActions::A, _AllActions::B]),
				),
				(
					UserInput::KeyCode(KeyCode::KeyA),
					HashSet::from([_AllActions::A, _AllActions::B]),
				),
			]));

			let output = repeated.to_string();

			let [header, items @ ..] = assert_count!(3, output.lines());
			assert_eq!("Multiple actions assigned to the same input(s):", header);
			assert!(either_or!(
				items.contains(&"  - KeyCode(KeyA) is assigned to: {A, B}"),
				items.contains(&"  - KeyCode(KeyA) is assigned to: {B, A}"),
			));
			assert!(either_or!(
				items.contains(&"  - KeyCode(KeyC) is assigned to: {A, B}"),
				items.contains(&"  - KeyCode(KeyC) is assigned to: {B, A}"),
			));
		}

		#[test]
		fn display_missing_inputs() {
			let repeated =
				LoadError::MissingInputs(HashSet::from([_AllActions::A, _AllActions::B]));

			let output = repeated.to_string();

			let [header, items @ ..] = assert_count!(3, output.lines());
			assert_eq!("Some actions have no input:", header);
			assert!(items.contains(&"  - A has no input"),);
			assert!(items.contains(
				&"  - B has no input. Either missing or part of invalid inputs for this action: [KeyCode(KeyC)]"
			),);
		}
	}
}
