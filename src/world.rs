use std::{collections::HashMap, io::Write};

use flate2::Compression;
use flate2::write::GzEncoder;

use crate::packet;
use crate::player::Player;

use std::{
  net::TcpStream,
  sync::mpsc
};

pub enum WorldCommand {
  AddPlayer { player_id: u32, stream: TcpStream, name: String },
  SetBlock { x: usize, y: usize, z: usize, value: u8, mode: u8 },
  MovePlayer {player_id: u32, x: u16, y: u16, z: u16, yaw: u8, pitch: u8},
  RemovePlayer {player_id: u32},
  PlayerMessage {player_id: u32, message: String}
}

pub struct World {
  pub size_x: usize,
  pub size_y: usize,
  pub size_z: usize,
  pub data: Box<[u8]>,
  pub players: HashMap<u32, Player>,
}

pub fn world_thread(rx: mpsc::Receiver<WorldCommand>) {
  let mut world = World::new();

  loop {
      match rx.recv() {
          Ok(WorldCommand::AddPlayer { player_id, stream, name }) => {
              let mut player = Player {stream: stream, name: name.trim().to_string(), x: 0, y: 0, z: 0, yaw: 0, pitch: 0 };
              let player_name = player.name.clone();

              player.send(&packet::server_identification_packet());
              player.send_world(&mut world);

              for (other_id, other) in &world.players {
                player.send(&packet::spawn_player_packet(*other_id as u8, &other.name, 0, 0, 0, 0, 0));
              }
              world.broadcast(&packet::spawn_player_packet(player_id as u8, &player_name, 0, 0, 0, 0, 0));

              player.send(&packet::teleport_player_packet(255, 32, 32, 32, 0, 0));

              world.players.insert(player_id, player);
              world.broadcast_message(&format!("{player_name} joined the game."));
              
          }
          Ok(WorldCommand::SetBlock { x, y, z, value, mode }) => {
              if mode != 0 { 
                world.set_block(x, y, z, value, Some(true));
              } else {
                world.set_block(x, y, z, 0, Some(true));
              }
          }
          Ok(WorldCommand::MovePlayer { player_id, x, y, z, yaw, pitch }) => {
            let player_exist = world.players.get_mut(&player_id);

            if let Some(player) = player_exist {
              player.x = x;
              player.y = y;
              player.z = z;
              player.yaw = yaw;
              player.pitch = pitch;
              world.broadcast(&packet::teleport_player_packet(player_id as u8, x, y, z, yaw, pitch));
            }
          }
          Ok(WorldCommand::RemovePlayer { player_id }) => {
            let player_exist = world.players.get(&player_id);
            if let Some(player) = player_exist {
              world.broadcast_message(&format!("{} left the game.", player.name));
            }

            world.players.remove(&player_id);
            world.broadcast(&packet::despawn_player_packet(player_id as u8));
          }
          Ok(WorldCommand::PlayerMessage { player_id, message }) => {
            let player_exist = world.players.get(&player_id);

            if let Some(player) = player_exist {
              world.broadcast_message(&format!("{}: {}", player.name, message));
            }
            
          }
          Err(_) => break,
      }
  }
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

    if let Some(update) = send_update {
      if update {
        for (_, player) in &mut self.players {
          player.send(&packet::set_block(x as u16, y as u16, z as u16, value));
        }
      }
    }
  }

  pub fn to_gzip(&self) -> Vec<u8> {
    let mut gz = GzEncoder::new(Vec::new(), Compression::default());

    let total_size = self.size_x * self.size_y * self.size_z;
    let size_bytes = (total_size as u32).to_be_bytes();

    let _ = gz.write_all(&size_bytes);
    let _ = gz.write_all(&self.data);

    let compressed_data = gz.finish();
    match compressed_data {
      Ok(_) => { 
        return compressed_data.unwrap(); 
      } Err(e) => { 
        eprintln!("Error when compressing the map: {}", e); 
        return Vec::new(); 
      }
    }
  }

  pub fn broadcast(&mut self, data: &Vec<u8>) {
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