pub(crate) mod act_on;
pub(crate) mod cast_ray;
pub(crate) mod query_filter_definition;
pub(crate) mod rapier_context;
pub(crate) mod ray_cast;
pub(crate) mod send_collision_interaction;
pub(crate) mod update_blockers;

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum TrackState {
	Changed,
	Unchanged,
}

pub(crate) trait Track<TEvent> {
	fn track(&mut self, event: &TEvent) -> TrackState;
}

pub(crate) trait Flush {
	type TResult;
	fn flush(&mut self) -> Self::TResult;
}
