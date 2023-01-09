use anyhow::Result;
use ecs::{izip, system, world::World};
use kiss3d::{
	event::{Action, Key, WindowEvent},
	light::Light,
	scene::SceneNode,
	text::Font,
	window::Window,
};
use na::{Point2, Point3, Translation3};
use nalgebra as na;
use physics::{Particle, Real, Vector3};
use std::{rc::Rc, time::Instant};

#[derive(Default, Debug, Eq, PartialEq, Copy, Clone)]
enum Shot {
	#[default]
	Pistol,
	Artillery,
	Fireball,
	Laser,
	Grenade,
}

#[derive(Default, Copy, Clone)]
struct Round {
	pub start_time: Option<Instant>,
	pub alive: bool,
}

const PARTICLE_TIMEOUT_SECS: usize = 5;
const AMMO_COUNT: usize = 10;

struct NextShot(pub Shot);

struct ShouldFire(pub bool);

fn main() -> Result<()> {
	let mut window = Window::new("Physics Engine - Ballistics Demo");
	window.set_light(Light::StickToCamera);
	let font = Font::default();

	let mut world = World::new();

	world.resources().borrow_mut().insert(NextShot(Shot::Pistol));
	world.resources().borrow_mut().insert(ShouldFire(false));

	let entities = world.create_entities(AMMO_COUNT);
	for entity in entities {
		let mut node = window.add_sphere(0.5);
		node.set_visible(false);
		node.set_color(0.0, 1.0, 1.0);
		world.add_component(entity, node).unwrap();

		let shot = world.resources().borrow().get::<NextShot>().unwrap().0;
		world.add_component(entity, Round::default()).unwrap();

		let position = Vector3::new(0.0, 1.5, 0.0);
		world.add_component(entity, shot_as_particle(shot, position)).unwrap();
	}

	while window.render() {
		map_keyboard_input(&window, &world);
		render_background(&world, &mut window, &font);
		physics_system(0.01, &mut world)?;
		projectile_system(&mut world)?;
		timeout_system(&mut world)?;
		sync_node_system(&mut world)?;
	}

	Ok(())
}

fn map_keyboard_input(window: &Window, world: &World) {
	for event in window.events().iter() {
		if let WindowEvent::Key(key, Action::Press, _) = event.value {
			match key {
				Key::Key1 => assign_next_shot(world, Shot::Pistol),
				Key::Key2 => assign_next_shot(world, Shot::Artillery),
				Key::Key3 => assign_next_shot(world, Shot::Fireball),
				Key::Key4 => assign_next_shot(world, Shot::Laser),
				Key::Key5 => assign_next_shot(world, Shot::Grenade),
				Key::Space => {
					if let Some(should_fire) = world.resources().borrow_mut().get_mut::<ShouldFire>() {
						should_fire.0 = true;
					}
				},
				_ => {},
			}
		}
	}
}

fn assign_next_shot(world: &World, shot: Shot) {
	world.resources().borrow_mut().insert(NextShot(shot))
}

fn render_background(world: &World, window: &mut Window, font: &Rc<Font>) {
	if let Some(NextShot(shot)) = world.resources().borrow().get::<NextShot>() {
		window.draw_text(&format!("Current Ammo Type: {:?}", shot), &Point2::origin(), 36.0, font, &Point3::new(0.0, 1.0, 1.0));
	}
	for offset in (0..200).step_by(10) {
		window.draw_line(
			&Point3::new(-5.0, 0.0, offset as _),
			&Point3::new(5.0, 0.0, offset as _),
			&Point3::new(0.75, 0.75, 0.75),
		);
	}
}

system!(physics_system, [_resources, _entity], (duration: f32), (particle: Particle, round: Round) -> Result<()> {
	if round.alive {
		particle.integrate(duration);
	}
	Ok(())
});

system!(projectile_system, [resources, _entity], (), (particle: Particle, round: Round) -> Result<()> {
	if round.alive {
		return Ok(())
	}
	if matches!(resources.borrow().get::<ShouldFire>(), Some(ShouldFire(true))) {
		round.start_time = Some(Instant::now());
		round.alive = true;
		let position = Vector3::new(0.0, 1.5, 0.0);
		*particle = shot_as_particle(resources.borrow().get::<NextShot>().unwrap().0, position);
		resources.borrow_mut().get_mut::<ShouldFire>().as_deref_mut().unwrap().0 = false;
	}
	Ok(())
});

system!(timeout_system, [_resources, _entity], (), (round: Round, particle: Particle) -> Result<()> {
	if !round.alive {
		return Ok(());
	}
	let out_of_bounds = particle.position.y() < 0.0 || particle.position.z() > 200.0;
	let expired = match round.start_time {
		Some(instant) => (Instant::now() - instant).as_secs() > PARTICLE_TIMEOUT_SECS as _,
		None => true,
	};
	if out_of_bounds || expired {
		round.start_time = None;
		round.alive = false;
	}
	Ok(())
});

system!(sync_node_system, [_resources, _entity], (), (node: SceneNode, particle: Particle, round: Round) -> Result<()> {
	node.set_visible(round.alive);
	node.set_local_translation(Translation3::new(
		particle.position.x() as _,
		particle.position.y() as _,
		particle.position.z() as _,
	));
	Ok(())
});

fn shot_as_particle(shot: Shot, position: Vector3) -> Particle {
	match shot {
		Shot::Pistol => {
			Particle {
				inverse_mass: (2.0 as Real).recip(),    // 2.0 kg
				velocity: Vector3::new(0.0, 0.0, 35.0), // 35 m/s
				acceleration: Vector3::new(0.0, -1.0, 0.0),
				damping: 0.99,
				position,
				force_accumulator: Vector3::zero(),
			}
		},
		Shot::Artillery => {
			Particle {
				inverse_mass: (200.0 as Real).recip(),   // 200.0 kg
				velocity: Vector3::new(0.0, 30.0, 40.0), // 50 m/s
				acceleration: Vector3::new(0.0, -20.0, 0.0),
				damping: 0.99,
				position,
				force_accumulator: Vector3::zero(),
			}
		},
		Shot::Fireball => {
			Particle {
				inverse_mass: (1.0 as Real).recip(),       // 1.0 kg
				velocity: Vector3::new(0.0, 00.0, 10.0),   // 5 m/s
				acceleration: Vector3::new(0.0, 0.6, 0.0), // Floats up
				damping: 0.9,
				position,
				force_accumulator: Vector3::zero(),
			}
		},
		Shot::Laser => {
			// Note that this is the kind of laser bolt seen in films, not a realistic laser beam!
			Particle {
				inverse_mass: (0.1 as Real).recip(),       // 1.0 kg
				velocity: Vector3::new(0.0, 0.0, 100.0),   // 100 m/s
				acceleration: Vector3::new(0.0, 0.0, 0.0), // No gravity
				damping: 0.99,
				position,
				force_accumulator: Vector3::zero(),
			}
		},
		Shot::Grenade => {
			Particle {
				inverse_mass: (0.9 as Real).recip(), // 200.0 kg
				velocity: Vector3::new(0.0, 15.0, 10.0),
				acceleration: Vector3::new(0.0, -10.0, 0.0),
				damping: 0.99,
				position,
				force_accumulator: Vector3::zero(),
			}
		},
	}
}
