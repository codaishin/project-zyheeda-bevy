use crate::components::player::Player;
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::traits::{
	accessors::get::{ContextChanged, GetContext, TryGetContextMut},
	handles_graphics::{Highlight, SetHighlight, Visual},
	handles_physics::{InteractionsJustStopped, InteractionsOngoing, IterInteractions},
};

impl Player {
	pub(crate) fn highlight_interactive<TPhysics, TGraphics>(
		players: Query<Entity, With<Player>>,
		physics: StaticSystemParam<TPhysics>,
		mut graphics: StaticSystemParam<TGraphics>,
	) where
		TPhysics: SystemParam
			+ for<'c> GetContext<InteractionsOngoing, TContext<'c>: IterInteractions>
			+ for<'c> GetContext<InteractionsJustStopped, TContext<'c>: IterInteractions>,
		TGraphics: SystemParam + for<'c> TryGetContextMut<Visual, TContext<'c>: SetHighlight>,
	{
		let Ok(player) = players.single() else {
			return;
		};

		let ongoing = InteractionsOngoing { entity: player };
		let ongoing = TPhysics::get_context(&physics, ongoing);

		if ongoing.context_changed() {
			for entity in ongoing.iter_interactions() {
				let key = Visual { entity };
				let Some(mut ctx) = TGraphics::try_get_context_mut(&mut graphics, key) else {
					continue;
				};

				ctx.set_highlight(Highlight::Interacting);
			}
		}

		let stopped = InteractionsJustStopped { entity: player };
		let stopped = TPhysics::get_context(&physics, stopped);

		if stopped.context_changed() {
			for entity in stopped.iter_interactions() {
				let key = Visual { entity };
				let Some(mut ctx) = TGraphics::try_get_context_mut(&mut graphics, key) else {
					continue;
				};

				ctx.set_highlight(Highlight::None);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{SystemParam, SystemParamItem};
	use common::traits::{
		accessors::get::ContextChanged,
		handles_graphics::{GetHighlight, Highlight},
	};
	use std::{collections::HashMap, iter::Copied, slice::Iter};
	use testing::SingleThreadedApp;

	#[derive(Resource)]
	struct _Interactions(HashMap<Entity, _InteractiveEntry>);

	#[derive(SystemParam)]
	struct _InteractionsParam<'w> {
		interactions: Res<'w, _Interactions>,
	}

	impl GetContext<InteractionsOngoing> for _InteractionsParam<'static> {
		type TContext<'ctx> = _InteractiveCtx;

		fn get_context<'ctx>(
			param: &'ctx SystemParamItem<Self>,
			InteractionsOngoing { entity }: InteractionsOngoing,
		) -> Self::TContext<'ctx> {
			match param.interactions.0.get(&entity).cloned() {
				Some(entry) => _InteractiveCtx {
					interactions: entry.ongoing,
					changed: entry.ongoing_changed,
				},
				None => panic!("NOT CONTEXT SET UP FOR {entity}"),
			}
		}
	}

	impl GetContext<InteractionsJustStopped> for _InteractionsParam<'static> {
		type TContext<'ctx> = _InteractiveCtx;

		fn get_context<'ctx>(
			param: &'ctx SystemParamItem<Self>,
			InteractionsJustStopped { entity }: InteractionsJustStopped,
		) -> Self::TContext<'ctx> {
			match param.interactions.0.get(&entity).cloned() {
				Some(entry) => _InteractiveCtx {
					interactions: entry.stopped,
					changed: entry.stopped_changed,
				},
				None => panic!("NOT CONTEXT SET UP FOR {entity}"),
			}
		}
	}

	#[derive(Clone, Default)]
	struct _InteractiveEntry {
		ongoing: Vec<Entity>,
		ongoing_changed: bool,
		stopped: Vec<Entity>,
		stopped_changed: bool,
	}

	#[derive(Clone)]
	struct _InteractiveCtx {
		interactions: Vec<Entity>,
		changed: bool,
	}

	impl ContextChanged for _InteractiveCtx {
		fn context_changed(&self) -> bool {
			self.changed
		}
	}

	impl IterInteractions for _InteractiveCtx {
		type TIter<'a>
			= Copied<Iter<'a, Entity>>
		where
			Self: 'a;

		fn iter_interactions(&self) -> Self::TIter<'_> {
			self.interactions.iter().copied()
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Highlight(Highlight);

	impl GetHighlight for _Highlight {
		fn get_highlight(&self) -> Highlight {
			self.0
		}
	}

	impl SetHighlight for _Highlight {
		fn set_highlight(&mut self, highlight: Highlight) {
			self.0 = highlight;
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(_Interactions(HashMap::from([])));
		app.add_systems(
			Update,
			Player::highlight_interactive::<_InteractionsParam, Query<Mut<_Highlight>>>,
		);

		app
	}

	mod ongoing {
		use super::*;

		#[test]
		fn set_highlight_interactive() {
			let mut app = setup();
			let interactive = app.world_mut().spawn(_Highlight(Highlight::None)).id();
			let player = app.world_mut().spawn(Player).id();
			app.insert_resource(_Interactions(HashMap::from([(
				player,
				_InteractiveEntry {
					ongoing: vec![interactive],
					ongoing_changed: true,
					..default()
				},
			)])));

			app.update();

			assert_eq!(
				Some(&_Highlight(Highlight::Interacting)),
				app.world().entity(interactive).get::<_Highlight>(),
			);
		}

		#[test]
		fn act_only_once() {
			let mut app = setup();
			let interactive = app.world_mut().spawn(_Highlight(Highlight::None)).id();
			let player = app.world_mut().spawn(Player).id();
			app.insert_resource(_Interactions(HashMap::from([(
				player,
				_InteractiveEntry {
					ongoing: vec![interactive],
					ongoing_changed: true,
					..default()
				},
			)])));

			app.update();
			app.insert_resource(_Interactions(HashMap::from([(
				player,
				_InteractiveEntry {
					ongoing: vec![interactive],
					ongoing_changed: false,
					..default()
				},
			)])));
			app.world_mut()
				.entity_mut(interactive)
				.insert(_Highlight(Highlight::None));
			app.update();

			assert_eq!(
				Some(&_Highlight(Highlight::None)),
				app.world().entity(interactive).get::<_Highlight>(),
			);
		}

		#[test]
		fn act_again_if_interactions_changed() {
			let mut app = setup();
			let interactive = app.world_mut().spawn(_Highlight(Highlight::None)).id();
			let player = app.world_mut().spawn(Player).id();
			app.insert_resource(_Interactions(HashMap::from([(
				player,
				_InteractiveEntry {
					ongoing: vec![interactive],
					ongoing_changed: true,
					..default()
				},
			)])));

			app.update();
			app.insert_resource(_Interactions(HashMap::from([(
				player,
				_InteractiveEntry {
					ongoing: vec![interactive],
					ongoing_changed: true,
					..default()
				},
			)])));
			app.world_mut()
				.entity_mut(interactive)
				.insert(_Highlight(Highlight::None));
			app.update();

			assert_eq!(
				Some(&_Highlight(Highlight::Interacting)),
				app.world().entity(interactive).get::<_Highlight>(),
			);
		}
	}

	mod stopped {
		use super::*;

		#[test]
		fn unset_highlight_interactive() {
			let mut app = setup();
			let interactive = app
				.world_mut()
				.spawn(_Highlight(Highlight::Interacting))
				.id();
			let player = app.world_mut().spawn(Player).id();
			app.insert_resource(_Interactions(HashMap::from([(
				player,
				_InteractiveEntry {
					stopped: vec![interactive],
					stopped_changed: true,
					..default()
				},
			)])));

			app.update();

			assert_eq!(
				Some(&_Highlight(Highlight::None)),
				app.world().entity(interactive).get::<_Highlight>(),
			);
		}

		#[test]
		fn act_only_once() {
			let mut app = setup();
			let interactive = app
				.world_mut()
				.spawn(_Highlight(Highlight::Interacting))
				.id();
			let player = app.world_mut().spawn(Player).id();
			app.insert_resource(_Interactions(HashMap::from([(
				player,
				_InteractiveEntry {
					stopped: vec![interactive],
					stopped_changed: true,
					..default()
				},
			)])));

			app.update();
			app.insert_resource(_Interactions(HashMap::from([(
				player,
				_InteractiveEntry {
					stopped: vec![interactive],
					stopped_changed: false,
					..default()
				},
			)])));
			app.world_mut()
				.entity_mut(interactive)
				.insert(_Highlight(Highlight::Interacting));
			app.update();

			assert_eq!(
				Some(&_Highlight(Highlight::Interacting)),
				app.world().entity(interactive).get::<_Highlight>(),
			);
		}

		#[test]
		fn act_again_if_interactions_changed() {
			let mut app = setup();
			let interactive = app
				.world_mut()
				.spawn(_Highlight(Highlight::Interacting))
				.id();
			let player = app.world_mut().spawn(Player).id();
			app.insert_resource(_Interactions(HashMap::from([(
				player,
				_InteractiveEntry {
					stopped: vec![interactive],
					stopped_changed: true,
					..default()
				},
			)])));

			app.update();
			app.insert_resource(_Interactions(HashMap::from([(
				player,
				_InteractiveEntry {
					stopped: vec![interactive],
					stopped_changed: true,
					..default()
				},
			)])));
			app.world_mut()
				.entity_mut(interactive)
				.insert(_Highlight(Highlight::Interacting));
			app.update();

			assert_eq!(
				Some(&_Highlight(Highlight::None)),
				app.world().entity(interactive).get::<_Highlight>(),
			);
		}
	}
}
