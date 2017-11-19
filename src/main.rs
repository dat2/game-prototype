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
use piston::event_loop::{Events, EventSettings};
use piston::input::{Button, PressEvent, RenderEvent};
use specs::{DispatcherBuilder, World};

mod ecs;

fn run() -> Result<(), Error> {
  let opengl = OpenGL::V3_2;

  let mut window: GlutinWindow = WindowSettings::new("prototype", [960, 640])
    .opengl(opengl)
    .exit_on_esc(true)
    .build()
    .unwrap();

  let mut world = World::new();
  world.register::<ecs::Position>();
  world.register::<ecs::Tile>();
  world.register::<ecs::RenderRect>();
  world.register::<ecs::Player>();
  world.add_resource(ecs::RenderEvents::new());
  world.add_resource(ecs::KeyPressEvents::new());

  ecs::load_tiles_into_world("main.tmx", &mut world)?;
  ecs::create_player(&mut world);

  let mut dispatcher = DispatcherBuilder::new()
    .add(ecs::InputSys, "input", &[])
    .add_thread_local(ecs::RenderSys::new(GlGraphics::new(opengl)))
    .build();

  let mut events = Events::new(EventSettings::new());
  while let Some(e) = events.next(&mut window) {
    if let Some(Button::Keyboard(key)) = e.press_args() {
      world.write_resource::<ecs::KeyPressEvents>().push(key);
    }
    if let Some(args) = e.render_args() {
      world.write_resource::<ecs::RenderEvents>().push(args);
      dispatcher.dispatch(&world.res);
    }
  }

  Ok(())
}

fn main() {
  if let Err(e) = run() {
    println!("{}", e);
  }
}
