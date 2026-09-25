mod apply_meta_to_context;

use crate::{
	assets::agent_meta::AgentMeta,
	components::{agent::Agent, agent_config::AgentConfig},
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::prelude::*;

impl AgentConfig {
	pub(crate) fn apply<TParam, TKey>(
		mut param: StaticSystemParam<TParam>,
		incomplete: Query<(Entity, &Agent, &Self)>,
		metas: Res<Assets<AgentMeta>>,
	) where
		TParam: TryGetContextMut<TKey>,
		TKey: ThreadSafe + From<Entity> + for<'c> ApplyMetaToContext<TParam::TContext<'c>>,
	{
		for (entity, agent, Self { config_handle }) in incomplete {
			let Some(meta) = metas.get(config_handle) else {
				continue;
			};

			let key = TKey::from(entity);
			let Some(mut ctx) = TParam::try_get_context_mut(&mut param, key) else {
				continue;
			};

			TKey::apply_meta_to_context(&mut ctx, meta, agent);
		}
	}
}

pub(crate) trait ApplyMetaToContext<TContext> {
	fn apply_meta_to_context(ctx: &mut TContext, meta: &AgentMeta, agent: &Agent);
}

#[cfg(test)]
mod tests {
	use super::*;
	use macros::EntityKey;
	use testing::{SingleThreadedApp, new_handle};

	#[derive(EntityKey)]
	struct _EntityContext {
		entity: Entity,
	}

	type _Param = Query<'static, 'static, &'static mut _Context>;

	#[derive(Component, Debug, PartialEq)]
	struct _Context(Option<(AgentMeta, Agent)>);

	impl<'c> ApplyMetaToContext<Mut<'c, _Context>> for _EntityContext {
		fn apply_meta_to_context(ctx: &mut Mut<'c, _Context>, meta: &AgentMeta, agent: &Agent) {
			**ctx = _Context(Some((meta.clone(), *agent)));
		}
	}

	fn setup<const N: usize>(metas: [(&Handle<AgentMeta>, AgentMeta); N]) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut assets = Assets::default();

		for (id, asset) in metas {
			_ = assets.insert(id, asset);
		}

		app.insert_resource(assets);
		app.add_systems(Update, AgentConfig::apply::<_Param, _EntityContext>);

		app
	}

	#[test]
	fn apply_meta() {
		let config_handle = new_handle();
		let meta = AgentMeta {
			required_clearance: RequiredClearance {
				vertical: Units::from_u8(1),
				horizontal: Units::from_u8(2),
			},
			..default()
		};
		let mut app = setup([(&config_handle, meta.clone())]);
		let entity = app
			.world_mut()
			.spawn((
				_Context(None),
				Agent {
					agent_type: AgentType::Enemy(EnemyType::VoidSphere),
				},
				AgentConfig { config_handle },
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Context(Some((
				meta,
				Agent {
					agent_type: AgentType::Enemy(EnemyType::VoidSphere)
				}
			)))),
			app.world().entity(entity).get::<_Context>(),
		);
	}
}
