use super::GetMarkerMeta;
use crate::markers::{
	functions::{insert_hand_marker_fn, remove_hand_marker_fn},
	meta::MarkerMeta,
	HandGun,
	Left,
	Right,
};

impl GetMarkerMeta for HandGun {
	fn marker() -> MarkerMeta {
		MarkerMeta {
			insert_fn: insert_hand_marker_fn::<(HandGun, Left), (HandGun, Right)>,
			remove_fn: remove_hand_marker_fn::<(HandGun, Left), (HandGun, Right)>,
		}
	}
}
