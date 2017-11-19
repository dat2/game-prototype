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
  world.register::<ecs::Tile>();
  world.add_resource(ecs::RenderEvents::new());

  ecs::load_tiles_into_world("main.tmx", &mut world)?;

  let tile_renderer = ecs::RenderSys::new(GlGraphics::new(opengl));
  let mut dispatcher = DispatcherBuilder::new()
    .add_thread_local(tile_renderer)
    .build();

  let mut events = Events::new(EventSettings::new().lazy(true));
  while let Some(e) = events.next(&mut window) {
    if let Some(args) = e.render_args() {
      world.write_resource::<ecs::RenderEvents>().push(args);
      dispatcher.dispatch(&mut world.res);
    }
    if let Some(_) = e.update_args() {
    }
  }

  Ok(())
}

fn main() {
  if let Err(e) = run() {
    println!("{}", e);
  }
}
