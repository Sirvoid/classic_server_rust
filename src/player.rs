use std::{net::TcpStream};
use crate::{packet, send_data, world::World};

pub struct Player {
  pub stream: TcpStream,
  pub name: String,
  pub x: u16,
  pub y: u16,
  pub z: u16,
  pub yaw: u8,
  pub pitch: u8
}

impl Player {
  pub fn send(&mut self, data: &[u8]) {
    send_data(&mut self.stream, data);
  }

  pub fn send_world(&mut self, world: &mut World) {
    let size_x: usize;
    let size_y: usize;
    let size_z: usize;

    size_x = world.size_x;
    size_y = world.size_y;
    size_z = world.size_z;

    let _ = match world.to_gzip() {
      Ok(data) => {
        self.send(&packet::level_initialize());

        for chunk in data.chunks(1024) {
          self.send(&packet::level_data_chunk(chunk.to_vec()));
        }
        
        self.send(&packet::level_finalize(size_x, size_y, size_z));
      } Err(e) => {
        eprintln!("Error sending world to player: {}", e);
      }
    };
  }

}