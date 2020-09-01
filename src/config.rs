mod edit_config;
mod open_config;
mod utils;

use utils::{load_to_string, store_string};

pub use open_config::OpenConfig;
pub use open_config::MimeCommands;
pub use open_config::PossibleCommands;
pub use open_config::RegexCommands;
pub use edit_config::EditConfig;
pub use edit_config::StreamingIteratorMut;
