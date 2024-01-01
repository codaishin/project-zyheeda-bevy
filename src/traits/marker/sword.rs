use super::GetMarkerMeta;
use crate::markers::{
	functions::{insert_hand_marker_fn, remove_hand_marker_fn},
	meta::MarkerMeta,
	Left,
	Right,
	Sword,
};

impl GetMarkerMeta for Sword {
	fn marker() -> MarkerMeta {
		MarkerMeta {
			insert_fn: insert_hand_marker_fn::<(Sword, Left), (Sword, Right)>,
			remove_fn: remove_hand_marker_fn::<(Sword, Left), (Sword, Right)>,
		}
	}
}
