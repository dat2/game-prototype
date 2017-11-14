#[macro_use]
extern crate error_chain;
extern crate piston_window;
extern crate tiled;

use piston_window::*;
use std::fs::File;

mod errors;

fn run() -> errors::Result<()> {
  let mut window: PistonWindow = WindowSettings::new("prototype", [960, 600]).exit_on_esc(true)
    .build()?;

  let reader = File::open("assets/main.tmx")?;
  let map = tiled::parse(reader)?;

  let tileset = map.get_tileset_by_gid(1).cloned().unwrap();
  let tile_width = tileset.tile_width;
  let tile_height = tileset.tile_height;

  let tilesheet = Texture::from_path(&mut window.factory,
                                     format!("assets/{}", &tileset.images[0].source),
                                     Flip::None,
                                     &TextureSettings::new())?;

  let (width, _) = tilesheet.get_size();
  let layer = &map.layers[0];
  let image = Image::new();

  while let Some(e) = window.next() {
    if let Some(_) = e.render_args() {
      window.draw_2d(&e, |c, g| {
        clear([0.5; 4], g);

        for (y, row) in layer.tiles.iter().enumerate().clone() {
          for (x, &tile) in row.iter().enumerate() {
            if tile == 0 {
              continue;
            }

            let tile = tile - 1;

            let src_rect = [(tile % (width / tile_width) * tile_width) as f64,
                            (tile / (width / tile_width) * tile_height) as f64,
                            tile_width as f64,
                            tile_height as f64];

            let trans = c.transform
              .scale(0.5, 0.5)
              .trans(x as f64 * tile_width as f64, y as f64 * tile_height as f64);

            image.src_rect(src_rect).draw(&tilesheet, &DrawState::default(), trans, g);
          }
        }
      });
    }
  }

  Ok(())
}

quick_main!(run);
