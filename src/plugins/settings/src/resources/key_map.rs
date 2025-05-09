pub(crate) mod dto;

use bevy::prelude::*;
use common::{
	tools::action_key::{ActionKey, user_input::UserInput},
	traits::{
		handles_custom_assets::TryLoadFrom,
		handles_settings::{InvalidInput, UpdateKey},
		iterate::Iterate,
		iteration::IterFinite,
		key_mappings::{GetInput, TryGetAction},
		load_asset::LoadAsset,
		thread_safe::ThreadSafe,
	},
};
use dto::KeyMapDto;
use std::{
	collections::{
		HashMap,
		HashSet,
		hash_map::{Entry, Iter},
	},
	error::Error,
	fmt::{Debug, Display},
	hash::Hash,
	marker::PhantomData,
};

#[derive(Resource, Asset, TypePath, Debug, PartialEq, Clone)]
pub struct KeyMap(KeyMapInternal);

impl<TAction> GetInput<TAction, UserInput> for KeyMap
where
	TAction: Copy,
	ActionKey: From<TAction>,
	UserInput: From<TAction>,
{
	fn get_input(&self, action: TAction) -> UserInput {
		self.0.get_input(action)
	}
}

impl<TAction> TryGetAction<UserInput, TAction> for KeyMap
where
	TAction: TryFrom<ActionKey> + Copy,
	UserInput: From<TAction>,
{
	fn try_get_action(&self, input: UserInput) -> Option<TAction> {
		self.0.try_get_action(input)
	}
}

impl<TAction> UpdateKey<TAction, UserInput> for KeyMap
where
	TAction: InvalidInput<UserInput> + Copy,
	ActionKey: From<TAction>,
	UserInput: From<TAction>,
{
	fn update_key(&mut self, action: TAction, input: UserInput) {
		self.0.update_key(action, input);
	}
}

impl TryLoadFrom<KeyMapDto<ActionKey, UserInput>> for KeyMap {
	type TInstantiationError = RepeatedAssignments<ActionKey, UserInput>;

	fn try_load_from<TLoadAsset>(
		dto: KeyMapDto<ActionKey, UserInput>,
		asset_server: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError>
	where
		TLoadAsset: LoadAsset,
	{
		KeyMapInternal::try_load_from(dto, asset_server).map(KeyMap)
	}
}

#[derive(Debug, PartialEq, Clone)]
struct KeyMapInternal<TAllActions = ActionKey, TInput = UserInput>
where
	TAllActions: Hash + Eq,
	TInput: Hash + Eq,
{
	phantom_data: PhantomData<(TAllActions, TInput)>,
	action_to_input: HashMap<TAllActions, TInput>,
	input_to_action: HashMap<TInput, TAllActions>,
}

impl<TAction, TInput> Default for KeyMapInternal<TAction, TInput>
where
	TAction: InvalidInput<TInput> + IterFinite + Copy + Hash + Eq,
	TInput: From<TAction> + Copy + Hash + Eq,
{
	fn default() -> Self {
		let mut map = Self {
			phantom_data: PhantomData,
			action_to_input: HashMap::default(),
			input_to_action: HashMap::default(),
		};

		for key in TAction::iterator() {
			map.update_key(key, TInput::from(key));
		}

		map
	}
}

impl<TAllActions, TAction, TInput> GetInput<TAction, TInput> for KeyMapInternal<TAllActions, TInput>
where
	TAllActions: From<TAction> + Hash + Eq,
	TAction: Copy,
	TInput: From<TAction> + Copy + Hash + Eq,
{
	fn get_input(&self, action: TAction) -> TInput {
		let Some(input) = self.action_to_input.get(&TAllActions::from(action)) else {
			return TInput::from(action);
		};

		*input
	}
}

impl<TAllActions, TAction, TInput> TryGetAction<TInput, TAction>
	for KeyMapInternal<TAllActions, TInput>
where
	TAllActions: Copy + Hash + Eq,
	TAction: TryFrom<TAllActions>,
	TInput: PartialEq + Hash + Eq,
{
	fn try_get_action(&self, input: TInput) -> Option<TAction> {
		let action = self.input_to_action.get(&input)?;
		TAction::try_from(*action).ok()
	}
}

impl<TAllActions, TAction, TInput> UpdateKey<TAction, TInput>
	for KeyMapInternal<TAllActions, TInput>
where
	TAllActions: From<TAction> + Hash + Eq + Copy,
	TAction: Copy + InvalidInput<TInput>,
	TInput: From<TAction> + Hash + Eq + Copy,
{
	fn update_key(&mut self, action: TAction, input: TInput) {
		if action.invalid_input().contains(&input) {
			return;
		}

		let old_input = self.get_input(action);
		let action = TAllActions::from(action);

		match self.input_to_action.get(&input).copied() {
			Some(old_action) => {
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
}

impl<TAction, TInput> TryLoadFrom<KeyMapDto<TAction, TInput>> for KeyMapInternal<TAction, TInput>
where
	TAction: InvalidInput<TInput> + IterFinite + Debug + Copy + Hash + Eq + TypePath + ThreadSafe,
	TInput: From<TAction> + Debug + Copy + Hash + Eq + TypePath + ThreadSafe,
{
	type TInstantiationError = RepeatedAssignments<TAction, TInput>;

	fn try_load_from<TLoadAsset>(
		KeyMapDto { actions }: KeyMapDto<TAction, TInput>,
		_: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError>
	where
		TLoadAsset: LoadAsset,
	{
		let mut mapper = Self::default();
		for (action, input) in actions {
			mapper.update_key(action, input);
		}

		if let Some(repeated) = RepeatedAssignments::from_mapper(&mapper) {
			return Err(repeated);
		}

		Ok(mapper)
	}
}

#[derive(TypePath, Debug, PartialEq)]
pub struct RepeatedAssignments<TAllKeys, TKeyCode>(HashMap<TKeyCode, HashSet<TAllKeys>>)
where
	TAllKeys: Debug + Eq + Hash + TypePath,
	TKeyCode: Debug + Eq + Hash + TypePath;

impl<TAllActions, TInput> RepeatedAssignments<TAllActions, TInput>
where
	TAllActions: Debug + Eq + Hash + TypePath + Copy,
	TInput: Debug + Eq + Hash + TypePath + Copy,
{
	fn from_mapper(mapper: &KeyMapInternal<TAllActions, TInput>) -> Option<Self> {
		let mut repeated = Self(HashMap::default());

		for (action, input) in &mapper.action_to_input {
			match repeated.0.entry(*input) {
				Entry::Occupied(mut entry) => {
					entry.get_mut().insert(*action);
				}
				Entry::Vacant(entry) => {
					entry.insert(HashSet::from([*action]));
				}
			}
		}

		repeated.0.retain(|_, keys| keys.len() > 1);

		if repeated.0.is_empty() {
			return None;
		}

		Some(repeated)
	}
}

impl<TAllKeys, TKeyCode> Display for RepeatedAssignments<TAllKeys, TKeyCode>
where
	TAllKeys: Debug + Eq + Hash + TypePath,
	TKeyCode: Debug + Eq + Hash + TypePath,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let actions = self
			.0
			.iter()
			.map(|(input, actions)| format!("  - {:?} is assigned to: {:?}", input, actions))
			.collect::<Vec<_>>()
			.join("\n");
		writeln!(f, "Multiple keys assigned to the same input(s):\n{actions}")
	}
}

impl<TAllKeys, TKeyCode> Error for RepeatedAssignments<TAllKeys, TKeyCode>
where
	TAllKeys: Debug + Eq + Hash + TypePath,
	TKeyCode: Debug + Eq + Hash + TypePath,
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
		A(_ActionA),
		B(_ActionB),
	}

	impl From<_AllActions> for _Input {
		fn from(action: _AllActions) -> Self {
			match action {
				_AllActions::A(key) => _Input::from(key),
				_AllActions::B(key) => _Input::from(key),
			}
		}
	}

	impl IterFinite for _AllActions {
		fn iterator() -> Iter<Self> {
			Iter(Some(_AllActions::A(_ActionA)))
		}

		fn next(current: &Iter<Self>) -> Option<Self> {
			match current.0? {
				_AllActions::A(_ActionA) => Some(_AllActions::B(_ActionB)),
				_AllActions::B(_ActionB) => None,
			}
		}
	}

	impl InvalidInput<_Input> for _AllActions {
		fn invalid_input(&self) -> &[_Input] {
			&[]
		}
	}

	#[derive(TypePath, Debug, PartialEq, Eq, Hash, Clone, Copy)]
	struct _ActionA;

	impl From<_ActionA> for _AllActions {
		fn from(action: _ActionA) -> Self {
			Self::A(action)
		}
	}

	impl TryFrom<_AllActions> for _ActionA {
		type Error = ();

		fn try_from(action: _AllActions) -> Result<Self, Self::Error> {
			match action {
				_AllActions::A(action) => Ok(action),
				_ => Err(()),
			}
		}
	}

	impl From<_ActionA> for _Input {
		fn from(_: _ActionA) -> Self {
			_Input::A
		}
	}

	impl InvalidInput<_Input> for _ActionA {
		fn invalid_input(&self) -> &[_Input] {
			&[]
		}
	}

	#[derive(TypePath, Debug, PartialEq, Eq, Hash, Clone, Copy)]
	struct _ActionB;

	impl From<_ActionB> for _AllActions {
		fn from(action: _ActionB) -> Self {
			Self::B(action)
		}
	}

	impl TryFrom<_AllActions> for _ActionB {
		type Error = ();

		fn try_from(action: _AllActions) -> Result<Self, Self::Error> {
			match action {
				_AllActions::B(action) => Ok(action),
				_ => Err(()),
			}
		}
	}

	impl From<_ActionB> for _Input {
		fn from(_: _ActionB) -> Self {
			_Input::B
		}
	}

	impl InvalidInput<_Input> for _ActionB {
		fn invalid_input(&self) -> &[_Input] {
			&[_Input::C]
		}
	}

	#[derive(TypePath, Debug, PartialEq, Eq, Hash, Clone, Copy)]
	enum _Input {
		A,
		B,
		C,
	}

	mod map {
		use super::*;

		#[test]
		fn to_input() {
			let mapper = KeyMapInternal::<_AllActions, _Input>::default();
			let mapped = mapper.get_input(_ActionB);

			assert_eq!(_Input::B, mapped);
		}

		#[test]
		fn to_key_a() {
			let mapper = KeyMapInternal::<_AllActions, _Input>::default();
			let mapped = mapper.try_get_action(_Input::A);

			assert_eq!(Some(_ActionA), mapped);
		}

		#[test]
		fn to_key_b() {
			let mapper = KeyMapInternal::<_AllActions, _Input>::default();
			let mapped = mapper.try_get_action(_Input::B);

			assert_eq!(Some(_ActionB), mapped);
		}
	}

	mod update {
		use super::*;

		#[test]
		fn key() {
			let action = _ActionA;
			let input = _Input::B;
			let mut mapper = KeyMapInternal::<_AllActions, _Input>::default();
			mapper.update_key(action, input);

			assert_eq!(
				(input, Some(action)),
				(mapper.get_input(action), mapper.try_get_action(input))
			);
		}

		#[test]
		fn key_removing_old_input_pairing() {
			let action = _ActionA;
			let input_b = _Input::B;
			let input_c = _Input::C;
			let mut mapper = KeyMapInternal::<_AllActions, _Input>::default();
			mapper.update_key(action, input_b);
			mapper.update_key(action, input_c);

			assert_eq!(
				(input_c, Some(action), None as Option<_Input>),
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
			let input_a = _Input::A;
			let input_b = _Input::B;
			let mut mapper = KeyMapInternal::<_AllActions, _Input>::default();
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
			let action = _ActionB;
			let input = _Input::C;
			let mut mapper = KeyMapInternal::<_AllActions, _Input>::default();
			mapper.update_key(action, input);

			assert_eq!(
				(_Input::from(action), None as Option<_ActionA>),
				(mapper.get_input(action), mapper.try_get_action(input))
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
		fn from_dto() -> Result<(), RepeatedAssignments<_AllActions, _Input>> {
			let dto = KeyMapDto::from([(_AllActions::A(_ActionA), _Input::C)]);

			let mapper = KeyMapInternal::try_load_from(dto, &mut _Server)?;

			assert_eq!(
				(_Input::C, Some(_ActionA), _Input::B, Some(_ActionB)),
				(
					mapper.get_input(_ActionA),
					mapper.try_get_action(_Input::C),
					mapper.get_input(_ActionB),
					mapper.try_get_action(_Input::B),
				)
			);
			Ok(())
		}

		#[derive(TypePath, Debug, PartialEq, Eq, Hash, Clone, Copy)]
		enum _FaultyAction {
			A,
			B,
			C,
		}

		impl From<_FaultyAction> for _Input {
			fn from(value: _FaultyAction) -> Self {
				match value {
					_FaultyAction::A => _Input::A,
					_FaultyAction::B => _Input::C, // this is the faulty mapping
					_FaultyAction::C => _Input::C,
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

		impl InvalidInput<_Input> for _FaultyAction {
			fn invalid_input(&self) -> &[_Input] {
				&[]
			}
		}

		#[test]
		fn from_dto_error_when_input_assigned_twice() {
			let mapper = KeyMapInternal::try_load_from(KeyMapDto::from([]), &mut _Server);

			assert_eq!(
				Err(RepeatedAssignments(HashMap::from([(
					_Input::C,
					HashSet::from([_FaultyAction::B, _FaultyAction::C])
				)]))),
				mapper
			);
		}
	}

	mod repeated_assignments {
		use super::*;
		use common::assert_count;

		macro_rules! either_or {
			($a:expr, $b:expr) => {
				$a || $b
			};
			($a:expr, $b:expr,) => {
				either_or!($a, $b)
			};
		}

		#[test]
		fn display() {
			let repeated = RepeatedAssignments(HashMap::from([
				(
					_Input::C,
					HashSet::from([_AllActions::A(_ActionA), _AllActions::B(_ActionB)]),
				),
				(
					_Input::A,
					HashSet::from([_AllActions::A(_ActionA), _AllActions::B(_ActionB)]),
				),
			]));

			let output = repeated.to_string();

			let [header, items @ ..] = assert_count!(3, output.lines());
			assert_eq!("Multiple keys assigned to the same input(s):", header);
			assert!(either_or!(
				items.contains(&"  - A is assigned to: {A(_ActionA), B(_ActionB)}"),
				items.contains(&"  - A is assigned to: {B(_ActionB), A(_ActionA)}"),
			));
			assert!(either_or!(
				items.contains(&"  - C is assigned to: {A(_ActionA), B(_ActionB)}"),
				items.contains(&"  - C is assigned to: {B(_ActionB), A(_ActionA)}"),
			));
		}
	}
}
