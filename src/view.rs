use failure::Error;
use piston_window::*;
use std::fs::File;
use std::path::Path;
use tiled;

#[derive(Debug)]
struct Tile {
  tilesheet_x: f64,
  tilesheet_y: f64,
  width: f64,
  height: f64,
}

impl Tile {
  fn new(tilesheet_x: f64, tilesheet_y: f64, width: f64, height: f64) -> Tile {
    Tile {
      tilesheet_x: tilesheet_x,
      tilesheet_y: tilesheet_y,
      width: width,
      height: height,
    }
  }

  fn src_rect(&self) -> [f64; 4] {
    [self.tilesheet_x, self.tilesheet_y, self.width, self.height]
  }
}

#[derive(Debug)]
pub struct TileMap {
  tiles: Vec<Vec<Option<Tile>>>,
  tilesheet: G2dTexture,
  pub tile_width: usize,
  pub tile_height: usize,
}

impl TileMap {
  pub fn new<P: AsRef<Path>>(path: P, window: &mut PistonWindow) -> Result<TileMap, Error> {
    let file = File::open(path)?;
    let map = tiled::parse(file)?;

    let tileset = map.get_tileset_by_gid(1).cloned().unwrap();

    let image_path = format!("assets/{}", &tileset.images[0].source);
    let tilesheet = Texture::from_path(&mut window.factory,
                                       image_path,
                                       Flip::None,
                                       &TextureSettings::new())
      .unwrap();

    let tile_dimensions = (tileset.tile_width as usize, tileset.tile_height as usize);
    let tilesheet_width = tileset.images[0].width as usize;
    let num_tiles_per_row = tilesheet_width / tile_dimensions.0;

    Ok(TileMap {
      tiles: map.layers[0]
        .tiles
        .iter()
        .map(|vec| {
          vec.iter()
            .map(|&index| {
              if index == 0 {
                None
              } else {
                Some(Tile::new((((index - 1) as usize % num_tiles_per_row) *
                                tile_dimensions.0) as f64,
                               (((index - 1) as usize / num_tiles_per_row) *
                                tile_dimensions.1) as f64,
                               tile_dimensions.0 as f64,
                               tile_dimensions.1 as f64))
              }
            })
            .collect()
        })
        .collect(),
      tilesheet: tilesheet,
      tile_width: tile_dimensions.0,
      tile_height: tile_dimensions.1,
    })
  }

  pub fn render(&self, c: &Context, g: &mut G2d) {
    let image = Image::new();
    for (y, row) in self.tiles.iter().enumerate() {
      for (x, tile_opt) in row.iter().enumerate() {
        if let &Some(ref tile) = tile_opt {
          let transform = c.transform
            .scale(0.5, 0.5)
            .trans(x as f64 * self.tile_width as f64,
                   y as f64 * self.tile_height as f64);

          image.src_rect(tile.src_rect())
            .draw(&self.tilesheet, &DrawState::default(), transform, g);
        }
      }
    }
  }
}
