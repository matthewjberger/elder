mod editor;

use editor::Editor;
use elder::app::{run, AppConfig};

fn main() -> Result<(), elder::app::Error> {
	std::env::set_var("RUST_LOG", "info");
	env_logger::init();
	run(AppConfig::default(), Editor::default())
}
