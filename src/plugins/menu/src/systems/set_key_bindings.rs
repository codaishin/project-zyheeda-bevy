use crate::traits::update_key_bindings::UpdateKeyBindings;
use bevy::{
	ecs::{
		component::Mutable,
		system::{StaticSystemParam, SystemParam},
	},
	prelude::*,
};
use common::traits::handles_input::{GetAllInputs, InputSetupChanged};
use std::ops::Deref;

impl<T> SetKeyBindings for T where T: UpdateKeyBindings + Component<Mutability = Mutable> {}

pub(crate) trait SetKeyBindings:
	UpdateKeyBindings + Component<Mutability = Mutable> + Sized
{
	fn set_key_bindings_from<TInput>(
		input: StaticSystemParam<TInput>,
		mut components: Query<&mut Self>,
	) where
		for<'w, 's> TInput: SystemParam<Item<'w, 's>: GetAllInputs + InputSetupChanged>,
	{
		for mut component in &mut components {
			if !input.input_setup_changed() && !component.is_added() {
				continue;
			}

			component.update_key_bindings(input.deref());
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::tools::action_key::{ActionKey, user_input::UserInput};
	use std::{any::type_name, fmt::Debug};
	use testing::SingleThreadedApp;

	#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
	struct _Key;

	#[derive(Debug, PartialEq, Clone, Copy)]
	struct _KeyCode;

	#[derive(Component, Debug, PartialEq, Default)]
	struct _Component {
		called_with: Vec<&'static str>,
	}

	impl UpdateKeyBindings for _Component {
		fn update_key_bindings<TInput>(&mut self, _: &TInput) {
			self.called_with.push(type_name::<TInput>());
		}
	}

	#[derive(SystemParam)]
	struct _Input<'w> {
		changed: Res<'w, _InputChanged>,
	}

	impl<'w> GetAllInputs for _Input<'w> {
		fn get_all_inputs(&self) -> impl Iterator<Item = (ActionKey, UserInput)> {
			std::iter::empty()
		}
	}

	impl<'w> InputSetupChanged for _Input<'w> {
		fn input_setup_changed(&self) -> bool {
			self.changed.0
		}
	}

	#[derive(Resource, Debug, PartialEq)]
	struct _InputChanged(bool);

	fn setup(input_changed: _InputChanged) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(input_changed);
		app.add_systems(Update, _Component::set_key_bindings_from::<_Input>);

		app
	}

	#[test]
	fn call_component_update() {
		let mut app = setup(_InputChanged(true));
		let entity = app.world_mut().spawn(_Component::default()).id();

		app.update();

		assert_eq!(
			Some(&_Component {
				called_with: vec![type_name::<_Input>()]
			}),
			app.world().entity(entity).get::<_Component>()
		);
	}

	#[test]
	fn do_not_call_component_update_when_component_not_added() {
		let mut app = setup(_InputChanged(true));
		let entity = app.world_mut().spawn(_Component::default()).id();

		app.update();
		app.insert_resource(_InputChanged(false));
		app.update();

		assert_eq!(
			Some(&_Component {
				called_with: vec![type_name::<_Input>()]
			}),
			app.world().entity(entity).get::<_Component>()
		);
	}

	#[test]
	fn call_component_update_again_when_input_changed() {
		let mut app = setup(_InputChanged(true));
		let entity = app.world_mut().spawn(_Component::default()).id();

		app.update();
		app.update();

		assert_eq!(
			Some(&_Component {
				called_with: vec![type_name::<_Input>(), type_name::<_Input>()]
			}),
			app.world().entity(entity).get::<_Component>()
		);
	}
}
