use failure::Error;
use graphics::*;
use opengl_graphics::{GlGraphics, Texture, TextureSettings};
use piston::input::*;
use specs::{FetchMut, Join, ReadStorage, System, VecStorage, World, WriteStorage};
use std::collections::VecDeque;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tiled;

// components
#[derive(Component)]
#[component(VecStorage)]
pub struct Position {
  pub x: f64,
  pub y: f64,
}

#[derive(Component)]
#[component(VecStorage)]
pub struct Rect {
  pub width: f64,
  pub height: f64,
  pub colour: [f32; 4],
}

#[derive(Component)]
#[component(VecStorage)]
pub struct SourceRect {
  x: f64,
  y: f64,
  width: f64,
  height: f64,
  texture: Arc<Texture>,
}

#[derive(Component)]
#[component(VecStorage)]
pub struct Player;

pub fn load_tiles_into_world<P: AsRef<Path>>(path: P, world: &mut World) -> Result<(), Error> {
  let mut path_buf = PathBuf::new();
  path_buf.push("assets");
  path_buf.push(path);
  let file = File::open(path_buf)?;
  let map = tiled::parse(file)?;
  let tileset = map.get_tileset_by_gid(1).cloned().unwrap();

  let tilesheet_path = format!("assets/{}", &tileset.images[0].source);
  let tilesheet = Arc::new(Texture::from_path(tilesheet_path, &TextureSettings::new()).unwrap());

  let tileset = map.get_tileset_by_gid(1).cloned().unwrap();

  let tile_dimensions = (tileset.tile_width as usize, tileset.tile_height as usize);
  let tilesheet_width = tileset.images[0].width as usize;
  let num_tiles_per_row = tilesheet_width / tile_dimensions.0;

  for (y, row) in map.layers[0].tiles.iter().enumerate() {
    for (x, &index) in row.iter().enumerate() {
      if index != 0 {
        world.create_entity()
          .with(Position {
            x: x as f64 * tile_dimensions.0 as f64,
            y: y as f64 * tile_dimensions.1 as f64,
          })
          .with(SourceRect {
            x: (((index - 1) as usize % num_tiles_per_row) * tile_dimensions.0) as f64,
            y: (((index - 1) as usize / num_tiles_per_row) * tile_dimensions.1) as f64,
            width: tile_dimensions.0 as f64,
            height: tile_dimensions.1 as f64,
            texture: tilesheet.clone(),
          })
          .build();
      }
    }
  }

  Ok(())
}

// piston resources
#[derive(Debug)]
pub struct KeyPressEvents(VecDeque<Key>);

impl KeyPressEvents {
  pub fn new() -> KeyPressEvents {
    KeyPressEvents(VecDeque::new())
  }

  pub fn push(&mut self, key: Key) {
    self.0.push_back(key);
  }
}

#[derive(Debug)]
pub struct RenderEvents(VecDeque<RenderArgs>);

impl RenderEvents {
  pub fn new() -> RenderEvents {
    RenderEvents(VecDeque::new())
  }

  pub fn push(&mut self, args: RenderArgs) {
    self.0.push_back(args);
  }
}

// systems
pub struct RenderSys {
  gl: GlGraphics,
}

impl RenderSys {
  pub fn new(gl: GlGraphics) -> RenderSys {
    RenderSys { gl: gl }
  }
}

impl<'a> System<'a> for RenderSys {
  type SystemData = (FetchMut<'a, RenderEvents>,
   ReadStorage<'a, Position>,
   ReadStorage<'a, SourceRect>,
   ReadStorage<'a, Rect>);

  fn run(&mut self, data: Self::SystemData) {
    let (mut render_events, pos, src_rect, rect) = data;

    if let Some(args) = render_events.0.pop_front() {
      self.gl.draw(args.viewport(), |c, g| {
        clear([0.0; 4], g);

        // draw src_rect
        let image = Image::new();
        for (pos, src_rect) in (&pos, &src_rect).join() {
          let transform = c.transform
            .scale(0.5, 0.5)
            .trans(pos.x, pos.y);

          image.src_rect([src_rect.x, src_rect.y, src_rect.width, src_rect.height])
            .draw(&*src_rect.texture, &DrawState::default(), transform, g);
        }

        // draw rectangles
        for (pos, rect) in (&pos, &rect).join() {
          rectangle(rect.colour,
                    [pos.x, pos.y, rect.width, rect.height],
                    c.transform.scale(0.5, 0.5),
                    g);
        }
      })
    }
  }
}

pub struct InputSys;

impl<'a> System<'a> for InputSys {
  type SystemData = (FetchMut<'a, KeyPressEvents>,
   ReadStorage<'a, Player>,
   WriteStorage<'a, Position>);

  fn run(&mut self, data: Self::SystemData) {
    let (mut key_press_events, player, mut pos) = data;

    if let Some(key) = key_press_events.0.pop_front() {
      for (_, mut pos) in (&player, &mut pos).join() {
        match key {
          Key::Left => pos.x -= 64.0,
          Key::Right => pos.x += 64.0,
          Key::Up => pos.y -= 64.0,
          Key::Down => pos.y += 64.0,
          _ => {}
        }
      }
    }
  }
}
