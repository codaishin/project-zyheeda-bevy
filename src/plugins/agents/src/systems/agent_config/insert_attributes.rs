use crate::{assets::agent_config::AgentConfigAsset, components::agent_config::AgentConfig};
use bevy::prelude::*;
use common::{
	traits::{accessors::get::TryApplyOn, handles_physics::PhysicalDefaultAttributes},
	zyheeda_commands::ZyheedaCommands,
};

impl AgentConfig {
	pub(crate) fn insert_attributes<TComponent>(
		mut commands: ZyheedaCommands,
		agents: Query<(Entity, &Self), Without<TComponent>>,
		configs: Res<Assets<AgentConfigAsset>>,
	) where
		TComponent: Component + From<PhysicalDefaultAttributes>,
	{
		for (entity, AgentConfig { config_handle }) in &agents {
			let Some(config) = configs.get(config_handle) else {
				continue;
			};

			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(TComponent::from(config.attributes));
			});
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use common::attributes::{effect_target::EffectTarget, health::Health};
	use testing::{SingleThreadedApp, new_handle};

	#[derive(Component, Debug, PartialEq)]
	struct _Component(PhysicalDefaultAttributes);

	impl From<PhysicalDefaultAttributes> for _Component {
		fn from(attributes: PhysicalDefaultAttributes) -> Self {
			Self(attributes)
		}
	}

	fn setup<const N: usize>(configs: [(&Handle<AgentConfigAsset>, AgentConfigAsset); N]) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut config_assets = Assets::default();

		for (id, config) in configs {
			_ = config_assets.insert(id, config);
		}

		app.insert_resource(config_assets);
		app.add_systems(Update, AgentConfig::insert_attributes::<_Component>);

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
		let mut app = setup([(
			&config_handle,
			AgentConfigAsset {
				attributes,
				..default()
			},
		)]);
		let entity = app.world_mut().spawn(AgentConfig { config_handle }).id();

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
		let mut app = setup([(
			&config_handle,
			AgentConfigAsset {
				attributes,
				..default()
			},
		)]);
		let entity = app
			.world_mut()
			.spawn(AgentConfig {
				config_handle: config_handle.clone(),
			})
			.id();

		app.update();
		let mut configs = app.world_mut().resource_mut::<Assets<AgentConfigAsset>>();
		let config = configs.get_mut(&config_handle).unwrap();
		config.attributes.health = Health::new(42.);
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
		let mut app = setup([(
			&config_handle,
			AgentConfigAsset {
				attributes,
				..default()
			},
		)]);
		let entity = app
			.world_mut()
			.spawn(AgentConfig {
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
