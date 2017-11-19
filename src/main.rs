extern crate failure;
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate specs;
#[macro_use]
extern crate specs_derive;
extern crate tiled;

use failure::Error;
use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use specs::{DispatcherBuilder, World};

mod ecs;
// mod view;

fn run() -> Result<(), Error> {
  let opengl = OpenGL::V3_2;

  let mut window: GlutinWindow = WindowSettings::new("prototype", [960, 640])
    .opengl(opengl)
    .exit_on_esc(true)
    .build()
    .unwrap();

  let mut world = World::new();
  world.register::<ecs::Position>();
  world.register::<ecs::SourceRect>();
  world.register::<ecs::Rect>();
  world.register::<ecs::Player>();
  world.add_resource(ecs::RenderEvents::new());
  world.add_resource(ecs::KeyPressEvents::new());

  ecs::load_tiles_into_world("main.tmx", &mut world)?;
  world.create_entity()
    .with(ecs::Position { x: 0.0, y: 0.0 })
    .with(ecs::Rect {
      width: 64.0,
      height: 64.0,
      colour: [1.0, 0.0, 0.0, 1.0],
    })
    .with(ecs::Player)
    .build();

  let mut dispatcher = DispatcherBuilder::new()
    .add(ecs::InputSys, "input", &[])
    .add_thread_local(ecs::RenderSys::new(GlGraphics::new(opengl)))
    .build();

  let mut events = Events::new(EventSettings::new().lazy(true));
  while let Some(e) = events.next(&mut window) {
    if let Some(args) = e.render_args() {
      world.write_resource::<ecs::RenderEvents>().push(args);
      dispatcher.dispatch(&mut world.res);
    }
    if let Some(Button::Keyboard(key)) = e.press_args() {
      world.write_resource::<ecs::KeyPressEvents>().push(key);
    }
  }

  Ok(())
}

fn main() {
  if let Err(e) = run() {
    println!("{}", e);
  }
}
