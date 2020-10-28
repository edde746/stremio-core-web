mod deep_links;
mod serializers;

mod serialize_installed_addons;
use serialize_installed_addons::*;

mod serialize_meta_details;
use serialize_meta_details::*;

mod serialize_player;
use serialize_player::*;

mod serialize_remote_addons;
pub use serialize_remote_addons::*;

mod model;
pub use model::*;
