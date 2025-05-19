use crate::traits::ExtraComponentsDefinition;
use bevy::prelude::*;
use common::traits::{handles_lights::HandlesLights, thread_safe::ThreadSafe};

impl<T> ApplyExtraComponents for T {}

pub(crate) trait ApplyExtraComponents {
	fn apply_extra_components<TLights>(
		mut commands: Commands,
		new: Query<(Entity, &Name), Added<Name>>,
	) where
		Self: ExtraComponentsDefinition,
		TLights: HandlesLights + ThreadSafe,
	{
		if new.is_empty() {
			return;
		}

		let target_names = Self::target_names();

		for (id, ..) in new.iter().filter(contained_in(target_names)) {
			let Ok(entity) = &mut commands.get_entity(id) else {
				continue;
			};
			Self::insert_bundle::<TLights>(entity);
		}
	}
}

fn contained_in(target_names: Vec<String>) -> impl Fn(&(Entity, &Name)) -> bool {
	move |(.., name)| target_names.contains(&name.as_str().to_owned())
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{test_tools::utils::SingleThreadedApp, traits::thread_safe::ThreadSafe};
	use mockall::automock;
	use std::marker::PhantomData;

	struct _Definition;

	#[derive(Debug, PartialEq)]
	struct _Lights;

	impl HandlesLights for _Lights {
		type TResponsiveLightBundle = ();
		type TResponsiveLightTrigger = ();

		const DEFAULT_LIGHT: Srgba = Srgba::BLACK;

		fn responsive_light_trigger() -> Self::TResponsiveLightTrigger {
			panic!("SHOULD NOT BE CALLED")
		}

		fn responsive_light_bundle<TDriver>(
			_: common::traits::handles_lights::Responsive,
		) -> Self::TResponsiveLightBundle
		where
			TDriver: 'static,
		{
			panic!("SHOULD NOT BE CALLED")
		}
	}

	#[derive(Component, Debug, PartialEq, Clone)]
	struct _Component<TLights>(PhantomData<TLights>);

	impl ExtraComponentsDefinition for _Definition {
		fn target_names() -> Vec<String> {
			vec!["AAA".to_owned()]
		}

		fn insert_bundle<TLights>(entity: &mut EntityCommands)
		where
			TLights: ThreadSafe,
		{
			entity.insert(_Component::<TLights>(PhantomData));
		}
	}

	fn setup<TDefinition: ExtraComponentsDefinition + 'static>() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, TDefinition::apply_extra_components::<_Lights>);

		app
	}

	#[test]
	fn add_component_when_name_matches() {
		let mut app = setup::<_Definition>();
		let agent = app.world_mut().spawn(Name::new("AAA")).id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some(&_Component::<_Lights>(PhantomData)),
			agent.get::<_Component<_Lights>>()
		);
	}

	#[test]
	fn ignore_when_name_not_matching() {
		let mut app = setup::<_Definition>();
		let agent = app.world_mut().spawn(Name::new("CCC")).id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(None, agent.get::<_Component<_Lights>>());
	}

	#[test]
	fn do_only_operate_once() {
		let mut app = setup::<_Definition>();
		let agent = app.world_mut().spawn(Name::new("AAA")).id();

		app.update();

		app.world_mut()
			.entity_mut(agent)
			.remove::<_Component<_Lights>>();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(None, agent.get::<_Component<_Lights>>());
	}

	struct _Definition2;

	impl ExtraComponentsDefinition for _Definition2 {
		fn target_names() -> Vec<String> {
			Mock_TargetNames::target_names()
		}

		fn insert_bundle<TLights>(_entity: &mut EntityCommands)
		where
			TLights: 'static,
		{
		}
	}

	#[automock]
	trait _TargetNames {
		fn target_names() -> Vec<String>;
	}

	#[test]
	fn do_not_call_target_names_multiple_times() {
		let mut app = setup::<_Definition2>();
		app.world_mut().spawn(Name::new("AAA"));
		app.world_mut().spawn(Name::new("AAA"));

		let target_names = Mock_TargetNames::target_names_context();
		target_names.expect().times(1).return_const(vec![]);

		app.update();
	}
}
