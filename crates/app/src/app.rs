use std::io;

use image::io::Reader;
use state::state::{State, StateMachine};
use thiserror::Error;
use winit::{
	self,
	dpi::PhysicalSize,
	error::OsError,
	event::{Event, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	window::{Fullscreen, Icon, Window, WindowBuilder},
};

#[derive(Error, Debug)]
pub enum Error {
	#[error("Failed to create icon file!")]
	CreateIcon(#[source] winit::window::BadIcon),

	#[error("Failed to create a window!")]
	CreateWindow(#[source] OsError),

	// #[error("Failed to create world!")]
	// CreateWorld(#[source] WorldError),

	// #[error("Failed to create the renderer!")]
	// CreateRenderer(#[source] Box<dyn std::error::Error>),
	#[error("Failed to decode icon file at path: {1}")]
	DecodeIconFile(#[source] image::ImageError, String),

	#[error("Failed to handle an event in the state machine!")]
	HandleEvent(#[source] Box<dyn std::error::Error>),

	// #[error("Failed to initialize the gamepad input library!")]
	// InitializeGamepadLibrary(#[source] gilrs::Error),
	#[error("Failed to open icon file at path: {1}")]
	OpenIconFile(#[source] io::Error, String),
	// #[error("Failed to render a frame!")]
	// RenderFrame(#[source] Box<dyn std::error::Error>),
	#[error("Failed to start the state machine!")]
	StartStateMachine(#[source] Box<dyn std::error::Error>),

	#[error("Failed to stop the state machine!")]
	StopStateMachine(#[source] Box<dyn std::error::Error>),

	// #[error("Failed to update the renderer!")]
	// UpdateRenderer(#[source] Box<dyn std::error::Error>),
	#[error("Failed to update the state machine!")]
	UpdateStateMachine(#[source] Box<dyn std::error::Error>),
	// #[error("Failed to to update the gui!")]
	// UpdateGui(#[source] Box<dyn std::error::Error>),

	// #[error("Failed to to resize the renderer!")]
	// ResizeRenderer(#[source] Box<dyn std::error::Error>),
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub struct AppConfig {
	pub width: u32,
	pub height: u32,
	pub is_fullscreen: bool,
	pub title: String,
	pub icon: Option<String>,
}

impl Default for AppConfig {
	fn default() -> Self {
		Self {
			width: 1024,
			height: 768,
			is_fullscreen: false,
			title: "Elder App".to_string(),
			icon: None,
		}
	}
}

pub fn run(config: AppConfig, initial_state: impl State<()> + 'static) -> Result<()> {
	log::info!("Application started");

	let event_loop = EventLoop::new();
	let mut window_builder = WindowBuilder::new()
		.with_title(config.title.to_string())
		.with_inner_size(PhysicalSize::new(config.width, config.height));

	if let Some(icon_path) = config.icon.as_ref() {
		let image = Reader::open(icon_path)
			.map_err(|error| Error::OpenIconFile(error, icon_path.to_string()))?
			.decode()
			.map_err(|error| Error::DecodeIconFile(error, icon_path.to_string()))?
			.into_rgba8();
		let (width, height) = image.dimensions();
		let icon = Icon::from_rgba(image.into_raw(), width, height).map_err(Error::CreateIcon)?;
		window_builder = window_builder.with_window_icon(Some(icon));
	}

	let mut window = window_builder.build(&event_loop).map_err(Error::CreateWindow)?;

	if config.is_fullscreen {
		window.set_fullscreen(Some(Fullscreen::Borderless(window.primary_monitor())));
	}

	let mut state_machine = StateMachine::new(initial_state);

	event_loop.run(move |event, _, control_flow| {
		if let Err(error) = run_loop(&mut window, &mut state_machine, &event, control_flow) {
			log::error!("Application error: {}", error);
		}
	});
}

fn run_loop(window: &mut Window, state_machine: &mut StateMachine<()>, event: &Event<()>, control_flow: &mut ControlFlow) -> Result<()> {
	control_flow.set_poll();

	if !state_machine.is_running() {
		state_machine.start(&mut ()).map_err(Error::StartStateMachine)?;
	}

	match event {
		Event::MainEventsCleared => {
			state_machine.update(&mut ()).map_err(Error::UpdateStateMachine)?;
		},

		Event::WindowEvent { ref event, window_id } if *window_id == window.id() => match event {
			WindowEvent::CloseRequested => control_flow.set_exit(),
			_ => {},
		},

		Event::LoopDestroyed => {
			state_machine.stop(&mut ()).map_err(Error::StopStateMachine)?;
		},

		_ => {},
	}
	Ok(())
}
