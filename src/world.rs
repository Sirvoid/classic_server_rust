use std::{io::{self, Read}, collections::HashMap, fs::File, io::Write};
use flate2::{read::GzDecoder, Compression, write::GzEncoder};

use crate::packet;
use crate::player::Player;

pub struct World {
  pub size_x: usize,
  pub size_y: usize,
  pub size_z: usize,
  pub data: Box<[u8]>,
  pub players: HashMap<u32, Player>,
}

impl World {
  pub fn new() -> World {
    let size_x = 128;
    let size_y = 64;
    let size_z = 128;

    let mut world: World = World { 
      size_x,
      size_y,
      size_z,
      data: vec![0; size_x * size_y * size_z].into_boxed_slice(),
      players: HashMap::new()
    };

    for x in 0..size_x {
      for z in 0..size_z {
        world.set_block(x, 0, z, 2, Some(false));
      }
    }

    return world;
  }

  pub fn set_block(&mut self, x: usize, y: usize, z: usize, value: u8, send_update: Option<bool>) {
    if x >= self.size_x || y >= self.size_y || z >= self.size_z {
      eprintln!("set_block out of bounds: ({}, {}, {})", x, y, z);
      return;
    }

    let index = x + self.size_x * (z + self.size_z * y);
    self.data[index] = value;

    if send_update.unwrap_or(false) {
      for (_, player) in &mut self.players {
        player.send(&packet::set_block(x as u16, y as u16, z as u16, value));
      }
    }
    
  }

  pub fn to_gzip(&self) -> Result<Vec<u8>, io::Error> {
    let mut gz = GzEncoder::new(Vec::new(), Compression::default());

    let total_size = self.size_x * self.size_y * self.size_z;
    let size_bytes = (total_size as u32).to_be_bytes();

    let _ = gz.write_all(&size_bytes);
    let _ = gz.write_all(&self.data);

    let compressed_data = gz.finish()?;
    return Ok(compressed_data);
  }

  pub fn from_gzip(&self, gzip_data: Vec<u8>) -> Result<Vec<u8>, io::Error> {
    let mut gz = GzDecoder::new(&gzip_data[..]);
    let mut data: Vec<u8> = vec![];
    gz.read_to_end(&mut data)?;
    return Ok(data);
  }

  pub fn load(&mut self) -> Result<(), io::Error> {
    let file_name = "world.dat";
    let mut file = File::open(file_name)?;
    
    let mut compressed_data: Vec<u8> = vec![];
    file.read_to_end(&mut compressed_data)?;

    let mut uncompressed_data = self.from_gzip(compressed_data)?;
    uncompressed_data.drain(..4); //remove 4 bytes world size header
    self.data = uncompressed_data.into();
    return Ok(());
  }
  
  pub fn save(&self) -> Result<(), io::Error> {
    let compressed_data = self.to_gzip()?;
    let file_name = "world.dat";
    let mut file = File::create(file_name)?;
    file.write_all(&compressed_data)?;
    return Ok(());
  }

  pub fn broadcast(&mut self, data: &[u8]) {
    for (_, player) in &mut self.players {
      player.send(data);
    }
  }

  pub fn broadcast_message(&mut self, message: &String) {
    println!("{message}");
    for (_, player) in &mut self.players {
      player.send(&packet::message(message));
    }
  }
}