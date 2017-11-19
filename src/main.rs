extern crate failure;
extern crate ncollide;
extern crate piston_window;
extern crate tiled;

use failure::Error;
use piston_window::*;

mod view;

fn run() -> Result<(), Error> {
  let mut window: PistonWindow = WindowSettings::new("prototype", [960, 640])
    .exit_on_esc(true)
    .build()
    .unwrap();

  let tile_map = view::TileMap::new("assets/main.tmx", &mut window)?;

  let mut player_position = (0.0, 0.0);

  while let Some(e) = window.next() {
    if let Some(Button::Keyboard(key)) = e.press_args() {
      match key {
        Key::Left => player_position.0 -= tile_map.tile_width as f64 / 2.0,
        Key::Right => player_position.0 += tile_map.tile_width as f64 / 2.0,
        _ => {}
      }
    }

    if let Some(_) = e.render_args() {
      window.draw_2d(&e, |c, mut g| {
        clear([0.0; 4], g);

        tile_map.render(&c, &mut g);

        // render player
        rectangle([1.0, 0.0, 0.0, 1.0],
                  [player_position.0, player_position.1, 32.0, 32.0],
                  c.transform,
                  g);

      });
    }

  }

  Ok(())
}

fn main() {
  if let Err(e) = run() {
    println!("{}", e);
  }
}
