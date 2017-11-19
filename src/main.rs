extern crate failure;
extern crate glutin_window;
extern crate graphics;
extern crate nalgebra as na;
extern crate ncollide;
extern crate nphysics2d;
extern crate opengl_graphics;
extern crate piston;
extern crate specs;
#[macro_use]
extern crate specs_derive;
extern crate tiled;

use failure::Error;
use glutin_window::GlutinWindow;
use opengl_graphics::OpenGL;
use piston::window::WindowSettings;
use piston::event_loop::{Events, EventSettings};
use piston::input::{Button, PressEvent, RenderEvent};

mod ecs;

fn run() -> Result<(), Error> {
  let mut window: GlutinWindow = WindowSettings::new("prototype", [960, 640])
    .opengl(OpenGL::V3_2)
    .exit_on_esc(true)
    .build()
    .unwrap();

  let mut world = ecs::create_world();
  let mut dispatcher = ecs::create_dispatcher();

  ecs::load_tiles_into_world("main.tmx", &mut world)?;
  ecs::create_player(&mut world);

  let mut events = Events::new(EventSettings::new());
  while let Some(e) = events.next(&mut window) {
    if let Some(Button::Keyboard(key)) = e.press_args() {
      world.write_resource::<ecs::KeyPressEvents>().push(key);
    }
    if let Some(args) = e.render_args() {
      world.write_resource::<ecs::RenderEvents>().push(args);
    }

    dispatcher.dispatch(&world.res);
    world.maintain();
  }

  Ok(())
}

fn main() {
  if let Err(e) = run() {
    println!("{}", e);
  }
}
