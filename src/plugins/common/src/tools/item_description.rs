use crate::traits::inspect_able::InspectMarker;

#[derive(Debug, PartialEq)]
pub struct ItemDescription;

impl InspectMarker for ItemDescription {
	type TFieldRef<'a> = String;
}
