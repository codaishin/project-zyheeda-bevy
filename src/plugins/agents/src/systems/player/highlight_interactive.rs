use crate::components::player::Player;
use bevy::{ecs::system::StaticSystemParam, platform::collections::HashSet, prelude::*};
use common::prelude::*;

impl Player {
	pub(crate) fn highlight_interactive<TPhysics, TGraphics>(
		players: Query<Entity, With<Player>>,
		physics: StaticSystemParam<TPhysics>,
		mut graphics: StaticSystemParam<TGraphics>,
		mut interacting: Local<HashSet<Entity>>,
	) where
		TPhysics: for<'c> GetContext<InteractionsOngoing, TContext<'c>: IterInteractions>,
		TGraphics: for<'c> TryGetContextMut<Visual, TContext<'c>: SetHighlight>,
	{
		let Ok(player) = players.single() else {
			return;
		};

		let ongoing = InteractionsOngoing { entity: player };
		let ongoing = TPhysics::get_context(&physics, ongoing);

		if !ongoing.context_changed() {
			return;
		}

		let ongoing = ongoing.iter_interactions().collect::<HashSet<_>>();

		interacting.retain(|entity| {
			if ongoing.contains(entity) {
				return true;
			}

			let key = Visual { entity: *entity };
			if let Some(mut ctx) = TGraphics::try_get_context_mut(&mut graphics, key) {
				ctx.set_highlight(Highlight::None);
			};

			false
		});

		for entity in ongoing {
			let key = Visual { entity };
			if let Some(mut ctx) = TGraphics::try_get_context_mut(&mut graphics, key) {
				ctx.set_highlight(Highlight::Interacting);
				interacting.insert(entity);
			};
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{SystemParam, SystemParamItem};
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

	#[derive(Clone, Default)]
	struct _InteractiveEntry {
		ongoing: Vec<Entity>,
		ongoing_changed: bool,
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
				},
			)])));

			app.update();
			app.insert_resource(_Interactions(HashMap::from([(
				player,
				_InteractiveEntry {
					ongoing: vec![interactive],
					ongoing_changed: false,
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
				},
			)])));

			app.update();
			app.insert_resource(_Interactions(HashMap::from([(
				player,
				_InteractiveEntry {
					ongoing: vec![interactive],
					ongoing_changed: true,
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
					ongoing: vec![interactive],
					ongoing_changed: true,
				},
			)])));

			app.update();
			app.insert_resource(_Interactions(HashMap::from([(
				player,
				_InteractiveEntry {
					ongoing: vec![],
					ongoing_changed: true,
				},
			)])));
			app.update();

			assert_eq!(
				Some(&_Highlight(Highlight::None)),
				app.world().entity(interactive).get::<_Highlight>(),
			);
		}

		#[test]
		fn do_nothing_if_not_changed() {
			let mut app = setup();
			let interactive = app
				.world_mut()
				.spawn(_Highlight(Highlight::Interacting))
				.id();
			let player = app.world_mut().spawn(Player).id();
			app.insert_resource(_Interactions(HashMap::from([(
				player,
				_InteractiveEntry {
					ongoing: vec![interactive],
					ongoing_changed: true,
				},
			)])));

			app.update();
			app.insert_resource(_Interactions(HashMap::from([(
				player,
				_InteractiveEntry {
					ongoing: vec![],
					ongoing_changed: false,
				},
			)])));
			app.update();

			assert_eq!(
				Some(&_Highlight(Highlight::Interacting)),
				app.world().entity(interactive).get::<_Highlight>(),
			);
		}
	}
}
