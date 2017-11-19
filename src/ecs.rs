use failure::Error;
use graphics::{clear, DrawState, Image, rectangle, Transformed};
use na::{Point2, Translation2, Vector2};
use ncollide::shape::{Cuboid, ShapeHandle};
use nphysics2d::object::RigidBody;
use nphysics2d::volumetric::Volumetric;
use nphysics2d::world::World as NWorld;
use opengl_graphics::{GlGraphics, OpenGL, Texture, TextureSettings};
use piston::input::{Key, RenderArgs};
use specs::{Dispatcher, DispatcherBuilder, Entity, Entities, Fetch, FetchMut, Join, LazyUpdate,
            ReadStorage, System, VecStorage, World, WriteStorage};
use std::collections::VecDeque;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tiled;

// components
#[derive(Component, Debug)]
#[component(VecStorage)]
struct Position {
  x: f64,
  y: f64,
}

#[derive(Component, Debug)]
#[component(VecStorage)]
struct RenderRect {
  width: f64,
  height: f64,
  colour: [f32; 4],
}

#[derive(Component)]
#[component(VecStorage)]
struct Tile {
  x: f64,
  y: f64,
  width: f64,
  height: f64,
  atlas: Arc<Texture>,
}

#[derive(Component, Debug)]
#[component(VecStorage)]
struct Player;

#[derive(Component, Debug)]
#[component(VecStorage)]
struct NRigidBody {
  shape: Cuboid<Vector2<f64>>,
  density: Option<f64>,
  restituion: f64,
  friction: f64,
}

pub fn create_world() -> World {
  let mut world = World::new();
  world.register::<Position>();
  world.register::<Tile>();
  world.register::<RenderRect>();
  world.register::<Player>();
  world.register::<NRigidBody>();
  world.add_resource(RenderEvents::new());
  world.add_resource(KeyPressEvents::new());
  world
}

pub fn create_dispatcher<'a, 'b>() -> Dispatcher<'a, 'b> {
  DispatcherBuilder::new()
    .add(InputSys, "input", &[])
    .add_thread_local(PhysicsSys::new())
    .add_thread_local(RenderSys::new(GlGraphics::new(OpenGL::V3_2)))
    .build()
}

pub fn load_tiles_into_world<P: AsRef<Path>>(path: P, world: &mut World) -> Result<(), Error> {
  let mut path_buf = PathBuf::new();
  path_buf.push("assets");
  path_buf.push(path);
  let file = File::open(path_buf)?;
  let map = tiled::parse(file)?;
  let tileset = map.get_tileset_by_gid(1).cloned().unwrap();

  let tilesheet_path = format!("assets/{}", &tileset.images[0].source);
  let tilesheet = Arc::new(
    Texture::from_path(tilesheet_path, &TextureSettings::new()).unwrap(),
  );

  let tileset = map.get_tileset_by_gid(1).cloned().unwrap();

  let tile_dimensions = (tileset.tile_width as usize, tileset.tile_height as usize);
  let tilesheet_width = tileset.images[0].width as usize;
  let num_tiles_per_row = tilesheet_width / tile_dimensions.0;

  for (y, row) in map.layers[0].tiles.iter().enumerate() {
    for (x, &index) in row.iter().enumerate() {
      if index != 0 {
        let (px, py) = (
          x as f64,
          y as f64,
        );
        let (sx, sy) = (
          (((index - 1) as usize % num_tiles_per_row) * tile_dimensions.0) as f64,
          (((index - 1) as usize / num_tiles_per_row) * tile_dimensions.1) as f64,
        );
        world
          .create_entity()
          .with(Position { x: px, y: py })
          .with(Tile {
            x: sx,
            y: sy,
            width: tile_dimensions.0 as f64,
            height: tile_dimensions.1 as f64,
            atlas: Arc::clone(&tilesheet),
          })
          .with(NRigidBody {
            shape: Cuboid::new(Vector2::new(px, py)),
            density: None,
            restituion: 0.3,
            friction: 0.6,
          })
          .build();
      }
    }
  }

  Ok(())
}

pub fn create_player(world: &mut World) {
  world
    .create_entity()
    .with(Position { x: 15.0, y: 8.0 })
    .with(RenderRect {
      width: 64.0,
      height: 64.0,
      colour: [1.0, 0.0, 0.0, 1.0],
    })
    .with(Player)
    .with(NRigidBody {
      shape: Cuboid::new(Vector2::new(64.0f64, 64.0f64)),
      density: Some(1.0),
      restituion: 0.3,
      friction: 0.6,
    })
    .build();
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
struct RenderSys {
  gl: GlGraphics,
}

impl RenderSys {
  fn new(gl: GlGraphics) -> RenderSys {
    RenderSys { gl: gl }
  }
}

impl<'a> System<'a> for RenderSys {
  type SystemData = (FetchMut<'a, RenderEvents>,
   ReadStorage<'a, Position>,
   ReadStorage<'a, Tile>,
   ReadStorage<'a, RenderRect>);

  fn run(&mut self, data: Self::SystemData) {
    let (mut render_events, pos, tile, rect) = data;

    if let Some(args) = render_events.0.pop_front() {
      self.gl.draw(args.viewport(), |c, g| {
        clear([0.0; 4], g);

        // draw tile
        let image = Image::new();
        for (pos, tile) in (&pos, &tile).join() {
          let transform = c.transform.scale(0.5, 0.5).trans(pos.x * 64.0, pos.y * 64.0);

          image
            .src_rect([tile.x, tile.y, tile.width, tile.height])
            .draw(&*tile.atlas, &DrawState::default(), transform, g);
        }

        // draw rectangles
        for (pos, rect) in (&pos, &rect).join() {
          rectangle(
            rect.colour,
            [pos.x * 64.0, pos.y * 64.0, rect.width, rect.height],
            c.transform.scale(0.5, 0.5),
            g,
          );
        }
      })
    }
  }
}

struct InputSys;

impl<'a> System<'a> for InputSys {
  type SystemData = (FetchMut<'a, KeyPressEvents>,
   ReadStorage<'a, Player>,
   WriteStorage<'a, Position>);

  fn run(&mut self, data: Self::SystemData) {
    let (mut key_press_events, player, mut pos) = data;

    if let Some(key) = key_press_events.0.pop_front() {
      for (_, mut pos) in (&player, &mut pos).join() {
        match key {
          Key::Left => pos.x -= 1.0,
          Key::Right => pos.x += 1.0,
          Key::Up => pos.y -= 1.0,
          Key::Down => pos.y += 1.0,
          _ => {}
        };
      }
    }
  }
}

struct PhysicsSys {
  world: NWorld<f64>,
}

impl PhysicsSys {
  fn new() -> PhysicsSys {
    let mut world = NWorld::new();
    // world.set_gravity(Vector2::new(0.0, 9.81));
    PhysicsSys { world: world }
  }
}

impl<'a> System<'a> for PhysicsSys {
  type SystemData = (Fetch<'a, RenderEvents>,
   Entities<'a>,
   ReadStorage<'a, NRigidBody>,
   WriteStorage<'a, Position>,
   Fetch<'a, LazyUpdate>);

  fn run(&mut self, data: Self::SystemData) {
    let (render_events, entities, bodies, mut positions, lazy) = data;
    if let Some(args) = render_events.0.front() {
      let dt = args.ext_dt;

      // add things to the world
      for (entity, body, pos) in (&*entities, &bodies, &positions).join() {
        let mut rb = RigidBody::new(
          ShapeHandle::new(body.shape.clone()),
          body.density.map(
            |density| body.shape.mass_properties(density),
          ),
          body.restituion,
          body.friction,
        );
        rb.append_translation(&Translation2::new(pos.x, pos.y));
        rb.set_user_data(Some(Box::new(entity)));
        lazy.remove::<NRigidBody>(entity);
        self.world.add_rigid_body(rb);
      }

      // step
      self.world.step(dt);

      // update
      let mut bodies_to_remove = Vec::new();
      for body in self.world.rigid_bodies() {
        let b = body.borrow();
        if let Some(any) = b.user_data() {
          if let Some(entity) = any.downcast_ref::<Entity>() {
            match positions.get_mut(*entity) {
              Some(position) => {
                let p = b.position() * Point2::origin();
                position.x = p.x;
                position.y = p.y;
              }
              None => {
                bodies_to_remove.push(body);
              }
            }
          }
        }
      }
    }
  }
}
