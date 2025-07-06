use std::{net::TcpStream, sync::mpsc};

use crate::{packet, player::Player, world::World};

pub enum WorldCommand {
  AddPlayer { player_id: u32, stream: TcpStream, name: String },
  SetBlock { x: usize, y: usize, z: usize, value: u8, mode: u8 },
  MovePlayer {player_id: u32, x: u16, y: u16, z: u16, yaw: u8, pitch: u8},
  RemovePlayer {player_id: u32},
  PlayerMessage {player_id: u32, message: String},
  SystemMessage {message: String},
  Save,
}

pub fn world_command_thread(rx: mpsc::Receiver<WorldCommand>) {
  let mut world = World::new();
  if let Err(e) = world.load() {
      eprintln!("Error loading world: {}", e);
  } else {
      println!("World loaded successfully.");
  }

  while let Ok(command) = rx.recv() {
      match command {
          WorldCommand::AddPlayer { player_id, stream, name } => {
              world.handle_add_player(player_id, stream, name);
          }
          WorldCommand::SetBlock { x, y, z, value, mode } => {
              world.handle_set_block(x, y, z, value, mode);
          }
          WorldCommand::MovePlayer { player_id, x, y, z, yaw, pitch } => {
              world.handle_move_player(player_id, x, y, z, yaw, pitch);
          }
          WorldCommand::RemovePlayer { player_id } => {
              world.handle_remove_player(player_id);
          }
          WorldCommand::PlayerMessage { player_id, message } => {
              world.handle_player_message(player_id, message);
          }
          WorldCommand::SystemMessage { message } => {
              world.handle_system_message(message);
          }
          WorldCommand::Save => {
              world.handle_save();
          }
      }
  }
}

impl World {
  pub fn handle_add_player(&mut self, player_id: u32, stream: TcpStream, name: String) {
      let mut player = Player {
          stream,
          name: name.trim().to_string(),
          x: 0,
          y: 0,
          z: 0,
          yaw: 0,
          pitch: 0,
      };

      player.send(&packet::server_identification_packet());
      player.send_world(self);

      for (other_id, other) in &self.players {
          player.send(&packet::spawn_player_packet(*other_id as u8, &other.name, 0, 0, 0, 0, 0));
      }

      self.broadcast(&packet::spawn_player_packet(player_id as u8, &player.name, 0, 0, 0, 0, 0));
      player.send(&packet::teleport_player_packet(255, 32, 32, 32, 0, 0));

      self.players.insert(player_id, player);
      self.broadcast_message(&format!("{} joined the game.", self.players[&player_id].name));
  }

  pub fn handle_set_block(&mut self, x: usize, y: usize, z: usize, value: u8, mode: u8) {
      let block_value = if mode != 0 { value } else { 0 };
      self.set_block(x, y, z, block_value, Some(true));
  }

  pub fn handle_move_player(&mut self, player_id: u32, x: u16, y: u16, z: u16, yaw: u8, pitch: u8) {
      if let Some(player) = self.players.get_mut(&player_id) {
          player.x = x;
          player.y = y;
          player.z = z;
          player.yaw = yaw;
          player.pitch = pitch;
          self.broadcast(&packet::teleport_player_packet(player_id as u8, x, y, z, yaw, pitch));
      }
  }

  pub fn handle_remove_player(&mut self, player_id: u32) {
      if let Some(player) = self.players.get(&player_id) {
          self.broadcast_message(&format!("{} left the game.", player.name));
      }

      self.players.remove(&player_id);
      self.broadcast(&packet::despawn_player_packet(player_id as u8));
  }

  pub fn handle_player_message(&mut self, player_id: u32, message: String) {
      if let Some(player) = self.players.get(&player_id) {
          self.broadcast_message(&format!("{}: {}", player.name, message));
      }
  }

  pub fn handle_system_message(&mut self, message: String) {
      self.broadcast_message(&message);
  }

  pub fn handle_save(&self) {
      if let Err(e) = self.save() {
          eprintln!("Error saving world: {}", e);
      } else {
          println!("World saved successfully.");
      }
  }
}