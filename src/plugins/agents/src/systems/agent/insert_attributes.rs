use crate::components::agent::Agent;
use bevy::prelude::*;
use common::{
	traits::{
		accessors::get::{GetProperty, TryApplyOn},
		handles_physics::PhysicalDefaultAttributes,
	},
	zyheeda_commands::ZyheedaCommands,
};

impl<TAsset> Agent<TAsset>
where
	TAsset: Asset + GetProperty<PhysicalDefaultAttributes>,
{
	pub(crate) fn insert_attributes<TComponent>(
		mut commands: ZyheedaCommands,
		agents: Query<(Entity, &Self), Without<TComponent>>,
		configs: Res<Assets<TAsset>>,
	) where
		TComponent: Component + From<PhysicalDefaultAttributes>,
	{
		for (entity, Agent { config_handle, .. }) in &agents {
			let Some(config) = configs.get(config_handle) else {
				continue;
			};

			commands.try_apply_on(&entity, |mut e| {
				let default_attributes = config.get_property();
				e.try_insert(TComponent::from(default_attributes));
			});
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use common::{
		attributes::{effect_target::EffectTarget, health::Health},
		traits::handles_map_generation::AgentType,
	};
	use testing::{SingleThreadedApp, new_handle};

	#[derive(Asset, TypePath)]
	struct _Config(PhysicalDefaultAttributes);

	impl GetProperty<PhysicalDefaultAttributes> for _Config {
		fn get_property(&self) -> PhysicalDefaultAttributes {
			self.0
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Component(PhysicalDefaultAttributes);

	impl From<PhysicalDefaultAttributes> for _Component {
		fn from(attributes: PhysicalDefaultAttributes) -> Self {
			Self(attributes)
		}
	}

	fn setup<const N: usize>(attributes: [(&Handle<_Config>, _Config); N]) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut assets = Assets::default();

		for (id, asset) in attributes {
			assets.insert(id, asset);
		}

		app.insert_resource(assets);
		app.add_systems(Update, Agent::<_Config>::insert_attributes::<_Component>);

		app
	}

	#[test]
	fn insert_default_attributes() {
		let config_handle = new_handle();
		let attributes = PhysicalDefaultAttributes {
			health: Health::new(100.),
			force_interaction: EffectTarget::Immune,
			gravity_interaction: EffectTarget::Affected,
		};
		let mut app = setup([(&config_handle, _Config(attributes))]);
		let entity = app
			.world_mut()
			.spawn(Agent {
				agent_type: AgentType::Player,
				config_handle,
			})
			.id();

		app.update();

		assert_eq!(
			Some(&_Component(attributes)),
			app.world().entity(entity).get::<_Component>(),
		);
	}

	#[test]
	fn insert_only_once() {
		let config_handle = new_handle();
		let attributes = PhysicalDefaultAttributes {
			health: Health::new(100.),
			force_interaction: EffectTarget::Immune,
			gravity_interaction: EffectTarget::Affected,
		};
		let mut app = setup([(&config_handle, _Config(attributes))]);
		let entity = app
			.world_mut()
			.spawn(Agent {
				agent_type: AgentType::Player,
				config_handle: config_handle.clone(),
			})
			.id();

		app.update();
		let mut configs = app.world_mut().resource_mut::<Assets<_Config>>();
		let config = configs.get_mut(&config_handle).unwrap();
		config.0.health = Health::new(42.);
		app.update();

		assert_eq!(
			Some(&_Component(attributes)),
			app.world().entity(entity).get::<_Component>(),
		);
	}

	#[test]
	fn insert_again_when_target_component_removed() {
		let config_handle = new_handle();
		let attributes = PhysicalDefaultAttributes {
			health: Health::new(100.),
			force_interaction: EffectTarget::Immune,
			gravity_interaction: EffectTarget::Affected,
		};
		let mut app = setup([(&config_handle, _Config(attributes))]);
		let entity = app
			.world_mut()
			.spawn(Agent {
				agent_type: AgentType::Player,
				config_handle: config_handle.clone(),
			})
			.id();

		app.update();
		app.world_mut().entity_mut(entity).remove::<_Component>();
		app.update();

		assert_eq!(
			Some(&_Component(attributes)),
			app.world().entity(entity).get::<_Component>(),
		);
	}
}
