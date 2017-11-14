error_chain!{
  foreign_links {
    Io(::std::io::Error);
    Tiled(::tiled::TiledError);
  }
}
