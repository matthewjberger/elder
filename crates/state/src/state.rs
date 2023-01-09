use thiserror::Error;

#[derive(Error, Debug)]
pub enum StateMachineError {
	#[error("No states are present in the state machine.")]
	NoStatesPresent,
}

type Result<T, E = StateMachineError> = std::result::Result<T, E>;

pub type StateResult<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

pub trait State<T> {
	fn label(&self) -> String {
		"Unlabeled State".to_string()
	}

	fn on_start(&mut self, _resources: &mut T) -> StateResult<()> {
		Ok(())
	}

	fn on_stop(&mut self, _resources: &mut T) -> StateResult<()> {
		Ok(())
	}

	fn on_pause(&mut self, _resources: &mut T) -> StateResult<()> {
		Ok(())
	}

	fn on_resume(&mut self, _resources: &mut T) -> StateResult<()> {
		Ok(())
	}

	fn update(&mut self, _resources: &mut T) -> StateResult<Transition<T>> {
		Ok(Transition::None)
	}
}

pub enum Transition<T> {
	None,
	Pop,
	Push(Box<dyn State<T>>),
	Switch(Box<dyn State<T>>),
	Quit,
}

pub struct StateMachine<T> {
	running: bool,
	states: Vec<Box<dyn State<T>>>,
}

impl<T> StateMachine<T> {
	pub fn new(initial_state: impl State<T> + 'static) -> Self {
		Self {
			running: false,
			states: vec![Box::new(initial_state)],
		}
	}

	pub fn active_state_label(&self) -> Option<String> {
		if !self.running {
			return None;
		}
		self.states.last().map(|state| state.label())
	}

	pub fn is_running(&self) -> bool {
		self.running
	}

	pub fn start(&mut self, resources: &mut T) -> StateResult<()> {
		if self.running {
			return Ok(());
		}
		self.running = true;
		self.active_state_mut()?.on_start(resources)
	}

	pub fn update(&mut self, resources: &mut T) -> StateResult<()> {
		if !self.running {
			return Ok(());
		}
		let transition = self.active_state_mut()?.update(resources)?;
		self.transition(transition, resources)
	}

	pub fn transition(&mut self, request: Transition<T>, resources: &mut T) -> StateResult<()> {
		if !self.running {
			return Ok(());
		}
		match request {
			Transition::None => Ok(()),
			Transition::Pop => self.pop(resources),
			Transition::Push(state) => self.push(state, resources),
			Transition::Switch(state) => self.switch(state, resources),
			Transition::Quit => self.stop(resources),
		}
	}

	pub fn active_state_mut(&mut self) -> Result<&mut Box<(dyn State<T> + 'static)>> {
		self.states.last_mut().ok_or(StateMachineError::NoStatesPresent)
	}

	pub fn switch(&mut self, state: Box<dyn State<T>>, resources: &mut T) -> StateResult<()> {
		if !self.running {
			return Ok(());
		}
		if let Some(mut state) = self.states.pop() {
			state.on_stop(resources)?;
		}
		self.states.push(state);
		self.active_state_mut()?.on_start(resources)
	}

	pub fn push(&mut self, state: Box<dyn State<T>>, resources: &mut T) -> StateResult<()> {
		if !self.running {
			return Ok(());
		}
		if let Ok(state) = self.active_state_mut() {
			state.on_pause(resources)?;
		}
		self.states.push(state);
		self.active_state_mut()?.on_start(resources)
	}

	pub fn pop(&mut self, resources: &mut T) -> StateResult<()> {
		if !self.running {
			return Ok(());
		}

		if let Some(mut state) = self.states.pop() {
			state.on_stop(resources)?;
		}

		if let Some(state) = self.states.last_mut() {
			state.on_resume(resources)
		} else {
			self.running = false;
			Ok(())
		}
	}

	pub fn stop(&mut self, resources: &mut T) -> StateResult<()> {
		if !self.running {
			return Ok(());
		}
		while let Some(mut state) = self.states.pop() {
			state.on_stop(resources)?;
		}
		self.running = false;
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Default)]
	pub struct Resources {
		value: u32,
	}

	#[derive(Default)]
	pub struct PrimaryState;
	impl State<Resources> for PrimaryState {
		fn label(&self) -> String {
			"Primary State".to_string()
		}

		fn update(&mut self, resources: &mut Resources) -> StateResult<Transition<Resources>> {
			resources.value = 10;
			Ok(Transition::Quit)
		}
	}

	#[derive(Default)]
	pub struct SecondaryState;
	impl State<Resources> for SecondaryState {
		fn label(&self) -> String {
			"Secondary State".to_string()
		}
	}

	#[test]
	pub fn switch() -> StateResult<()> {
		let mut resources = Resources::default();
		let mut state_machine = StateMachine::new(PrimaryState::default());
		assert!(!state_machine.is_running());

		state_machine.start(&mut resources)?;
		assert_eq!(state_machine.active_state_label(), Some("Primary State".to_string()));

		state_machine.switch(Box::new(SecondaryState::default()), &mut resources)?;
		assert_eq!(state_machine.states.len(), 1);
		assert_eq!(state_machine.active_state_label(), Some("Secondary State".to_string()));
		Ok(())
	}

	#[test]
	pub fn push_pop() -> StateResult<()> {
		let mut resources = Resources::default();
		let mut state_machine = StateMachine::new(PrimaryState::default());
		assert!(!state_machine.is_running());

		state_machine.start(&mut resources)?;
		assert_eq!(state_machine.active_state_label(), Some("Primary State".to_string()));

		state_machine.push(Box::new(SecondaryState::default()), &mut resources)?;
		assert_eq!(state_machine.states.len(), 2);
		assert_eq!(state_machine.active_state_label(), Some("Secondary State".to_string()));

		state_machine.pop(&mut resources)?;
		assert_eq!(state_machine.states.len(), 1);
		assert_eq!(state_machine.active_state_label(), Some("Primary State".to_string()));

		Ok(())
	}

	#[test]
	pub fn quit() -> StateResult<()> {
		let mut resources = Resources::default();
		let mut state_machine = StateMachine::new(PrimaryState::default());
		assert!(!state_machine.is_running());

		state_machine.start(&mut resources)?;
		assert!(state_machine.is_running());
		assert_eq!(state_machine.active_state_label(), Some("Primary State".to_string()));

		state_machine.stop(&mut resources)?;
		assert!(!state_machine.is_running());

		Ok(())
	}

	#[test]
	pub fn resources() -> StateResult<()> {
		let mut resources = Resources::default();
		let mut state_machine = StateMachine::new(PrimaryState::default());
		assert!(!state_machine.is_running());

		state_machine.start(&mut resources)?;
		assert!(state_machine.is_running());
		assert_eq!(state_machine.active_state_label(), Some("Primary State".to_string()));

		state_machine.update(&mut resources)?;
		assert_eq!(resources.value, 10);
		assert!(!state_machine.is_running());

		Ok(())
	}
}
