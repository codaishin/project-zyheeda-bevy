use crate::{assets::agent_config::AgentConfigAsset, components::agent::Agent};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::{GetContextMut, TryApplyOn},
		animation::{Animations, RegisterAnimations2},
	},
	zyheeda_commands::ZyheedaCommands,
};

impl Agent {
	pub(crate) fn register_animations<TAnimations>(
		mut commands: ZyheedaCommands,
		mut animations: StaticSystemParam<TAnimations>,
		agents: Query<(Entity, &Self), Without<marker::AnimationsRegistered>>,
		configs: Res<Assets<AgentConfigAsset>>,
	) where
		TAnimations: for<'c> GetContextMut<Animations, TContext<'c>: RegisterAnimations2>,
	{
		for (entity, Self { config_handle, .. }) in agents {
			let key = Animations { entity };

			let Some(config) = configs.get(config_handle) else {
				continue;
			};
			let Some(mut ctx) = TAnimations::get_context_mut(&mut animations, key) else {
				continue;
			};

			commands.try_apply_on(&entity, |mut e| {
				ctx.register_animations(&config.animations);
				e.try_insert(marker::AnimationsRegistered);
			});
		}
	}
}

mod marker {
	use super::*;

	#[derive(Component)]
	pub struct AnimationsRegistered;
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::assets::agent_config::AgentConfigAsset;
	use common::{
		tools::path::Path,
		traits::{
			animation::{
				AffectedAnimationBones2,
				Animation2,
				AnimationKey,
				AnimationPath,
				PlayMode,
			},
			handles_map_generation::AgentType,
		},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::collections::HashMap;
	use testing::{NestedMocks, SingleThreadedApp, new_handle};

	#[derive(Component, NestedMocks)]
	struct _Component {
		mock: Mock_Component,
	}

	#[automock]
	impl RegisterAnimations2 for _Component {
		fn register_animations(&mut self, animations: &HashMap<AnimationKey, Animation2>) {
			self.mock.register_animations(animations);
		}
	}

	fn setup<const N: usize>(configs: [(&Handle<AgentConfigAsset>, AgentConfigAsset); N]) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut config_assets = Assets::default();

		for (id, asset) in configs {
			config_assets.insert(id, asset);
		}

		app.insert_resource(config_assets);
		app.add_systems(Update, Agent::register_animations::<Query<Mut<_Component>>>);

		app
	}

	#[test]
	fn set_animations_from_config() {
		let animations = HashMap::from([(
			AnimationKey::Run,
			Animation2 {
				bones: AffectedAnimationBones2::default(),
				path: AnimationPath::Single(Path::from("my/path")),
				play_mode: PlayMode::Replay,
				mask: 1 << 42,
			},
		)]);
		let handle = new_handle();
		let asset = AgentConfigAsset {
			animations: animations.clone(),
			..default()
		};
		let mut app = setup([(&handle, asset)]);
		app.world_mut().spawn((
			Agent {
				agent_type: AgentType::Player,
				config_handle: handle,
			},
			_Component::new().with_mock(move |mock| {
				mock.expect_register_animations()
					.times(1)
					.with(eq(animations.clone()))
					.return_const(());
			}),
		));

		app.update();
	}

	#[test]
	fn act_only_once() {
		let animations = HashMap::from([(
			AnimationKey::Run,
			Animation2 {
				bones: AffectedAnimationBones2::default(),
				path: AnimationPath::Single(Path::from("my/path")),
				play_mode: PlayMode::Replay,
				mask: 1 << 42,
			},
		)]);
		let handle = new_handle();
		let asset = AgentConfigAsset {
			animations: animations.clone(),
			..default()
		};
		let mut app = setup([(&handle, asset)]);
		app.world_mut().spawn((
			Agent {
				agent_type: AgentType::Player,
				config_handle: handle,
			},
			_Component::new().with_mock(move |mock| {
				mock.expect_register_animations().times(1).return_const(());
			}),
		));

		app.update();
		app.update();
	}

	#[test]
	fn set_animations_from_config_when_config_is_late() {
		let animations = HashMap::from([(
			AnimationKey::Run,
			Animation2 {
				bones: AffectedAnimationBones2::default(),
				path: AnimationPath::Single(Path::from("my/path")),
				play_mode: PlayMode::Replay,
				mask: 1 << 42,
			},
		)]);
		let handle = new_handle();
		let asset = AgentConfigAsset {
			animations: animations.clone(),
			..default()
		};
		let mut app = setup([]);
		app.world_mut().spawn((
			Agent {
				agent_type: AgentType::Player,
				config_handle: handle.clone(),
			},
			_Component::new().with_mock(move |mock| {
				mock.expect_register_animations()
					.times(1)
					.with(eq(animations.clone()))
					.return_const(());
			}),
		));

		app.update();
		app.world_mut()
			.resource_mut::<Assets<AgentConfigAsset>>()
			.insert(&handle, asset);
		app.update();
	}
}
