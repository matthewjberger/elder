use elder::state::{State, StateResult, Transition};

#[derive(Default)]
pub struct Editor;

impl State<()> for Editor {
	fn label(&self) -> String {
		"Elder Game Engine - Editor".to_string()
	}

	fn start(&mut self, _resources: &mut ()) -> StateResult<()> {
		Ok(())
	}

	fn stop(&mut self, _resources: &mut ()) -> StateResult<()> {
		Ok(())
	}

	fn pause(&mut self, _resources: &mut ()) -> StateResult<()> {
		Ok(())
	}

	fn resume(&mut self, _resources: &mut ()) -> StateResult<()> {
		Ok(())
	}

	fn update(&mut self, _resources: &mut ()) -> StateResult<Transition<()>> {
		Ok(Transition::None)
	}
}
