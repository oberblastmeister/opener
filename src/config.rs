mod edit_config;
mod open_config;
mod utils;

use utils::{load_to_string, store_string};

pub use open_config::OpenConfig;
pub use open_config::PossibleMimes;
pub use edit_config::EditConfig;
