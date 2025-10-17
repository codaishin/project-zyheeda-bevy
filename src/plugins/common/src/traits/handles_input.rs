mod bevy_impls;

use crate::{
	tools::action_key::{ActionKey, user_input::UserInput},
	traits::{
		accessors::get::{GetProperty, Property},
		iteration::IterFinite,
	},
};
use bevy::{ecs::system::SystemParam, prelude::*};
use std::hash::Hash;

pub trait HandlesActionKeyButton {
	/// Controls triggering of actions through mouse left clicking the associated
	/// button.
	type TActionKeyButton: Component + From<ActionKey> + GetProperty<MouseOverride>;
}

/// Indicate whether left mouse behavior is overridden for an associated instance that
/// implements [`GetInputState`].
pub struct MouseOverride;

impl Property for MouseOverride {
	type TValue<'a> = bool;
}

pub trait HandlesInput {
	type TInput<'world, 'state>: SystemParam
		+ for<'w, 's> SystemParam<Item<'w, 's>: GetInput + GetAllInputs + InputSetupChanged>
		+ for<'w, 's> SystemParam<Item<'w, 's>: GetInputState + GetAllInputStates>
		+ for<'w, 's> SystemParam<Item<'w, 's>: GetRawUserInput>;
}

pub trait HandlesInputMut {
	type TInputMut<'world, 'state>: SystemParam
		+ for<'w, 's> SystemParam<Item<'w, 's>: GetInput + GetAllInputs + InputSetupChanged>
		+ for<'w, 's> SystemParam<Item<'w, 's>: GetInputState + GetAllInputStates>
		+ for<'w, 's> SystemParam<Item<'w, 's>: GetRawUserInput>
		+ for<'w, 's> SystemParam<Item<'w, 's>: UpdateKey>;
}

/// Helper type to designate [`HandlesInput::TInput`] as a [`SystemParam`] implementation for a
/// given generic system constraint
pub type InputSystemParam<'w, 's, T> = <T as HandlesInput>::TInput<'w, 's>;

/// Helper type to designate [`HandlesInputMut::TInputMut`] as a [`SystemParam`] implementation for a
/// given generic system constraint
pub type InputMutSystemParam<'w, 's, T> = <T as HandlesInputMut>::TInputMut<'w, 's>;

pub trait UpdateKey {
	fn update_key<TAction>(&mut self, action: TAction, input: UserInput)
	where
		TAction: Copy + Into<ActionKey> + 'static;
}

pub trait InvalidUserInput {
	fn invalid_input(&self) -> &[UserInput];
}

pub trait InputSetupChanged {
	fn input_setup_changed(&self) -> bool;
}

pub trait GetInput {
	fn get_input<TAction>(&self, action: TAction) -> UserInput
	where
		TAction: Into<ActionKey> + 'static;
}

pub trait GetAllInputs {
	fn get_all_inputs(&self) -> impl Iterator<Item = (ActionKey, UserInput)>;
}

/// Allows alternative access to bevy input
///
/// Use to avoid conflicts when accessing [`ButtonInput`] in systems is not possible,
/// due to it being already used by the implementing type.
pub trait GetRawUserInput {
	fn get_raw_user_input(&self, state: RawInputState) -> impl Iterator<Item = UserInput>;
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum RawInputState {
	JustPressed,
	Held,
	JustReleased,
}

pub trait GetInputState {
	fn get_input_state<TAction>(&self, action: TAction) -> InputState
	where
		TAction: Into<ActionKey> + 'static;
}

pub trait GetAllInputStates {
	fn get_all_input_states<TAction>(&self) -> impl Iterator<Item = (TAction, InputState)>
	where
		TAction: Into<ActionKey> + IterFinite + 'static;
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum InputState {
	Pressed { just_now: bool },
	Released { just_now: bool },
}

impl InputState {
	pub const fn pressed() -> Self {
		Self::Pressed { just_now: false }
	}

	pub const fn just_pressed() -> Self {
		Self::Pressed { just_now: true }
	}

	pub const fn released() -> Self {
		Self::Released { just_now: false }
	}

	pub const fn just_released() -> Self {
		Self::Released { just_now: true }
	}
}
