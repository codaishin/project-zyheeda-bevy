use macros::serde_model;

#[serde_model]
#[derive(Debug, PartialEq, Default, Clone)]
pub enum ModelRender {
	#[default]
	None,
	Hand(String),
	Forearm(String),
}
