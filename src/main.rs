mod packet;
mod world;
mod player;
mod world_command;
mod scheduler;

use std::{collections::HashMap, io::{Read, Write}, net::{TcpListener, TcpStream}, sync::mpsc::{self}, thread};
use crate::{scheduler::Scheduler, world_command::{world_command_thread, WorldCommand}};

pub fn send_data(stream: &mut TcpStream, data: &[u8]) {
  if let Err(e) = stream.write_all(data) {
    eprintln!("Error occured when writing data: {}", e);
  }

  if let Err(e) = stream.flush() {
    eprintln!("Error occured when flushing data: {}", e);
  }
}

fn handle_client(player_id: u32, mut stream: TcpStream, tx: mpsc::Sender<WorldCommand>) {
  let mut buf = [0; 512];

  loop {
    match stream.read(&mut buf) {
      Ok(0) => {
        tx.send(WorldCommand::RemovePlayer { player_id }).unwrap();
        break;
      },
      Ok(_) => {
        let mut reader = packet::PacketReader { index: 0, buffer: &buf };
        let packet_id = reader.read_u8();
        match packet_id {
          0x00 => {
            let (_, name, _) = packet::client_player_identification(&mut reader);
            tx.send(WorldCommand::AddPlayer {
              player_id,
              stream: stream.try_clone().unwrap(),
              name,
            }).unwrap();
          }
          0x05 => {
            let (x, y, z, value, mode) = packet::client_set_block(&mut reader);
            tx.send(WorldCommand::SetBlock { x: x as usize, y: y as usize, z: z as usize, value, mode }).unwrap();
          }
          0x08 => {
            let (_, x, y, z, yaw , pitch) = packet::client_position_orientation(&mut reader);
            tx.send(WorldCommand::MovePlayer { player_id, x, y, z, yaw, pitch }).unwrap();
          }
          0x0d => {
            let (_, message) = packet::client_chat_message(&mut reader);
            tx.send(WorldCommand::PlayerMessage { player_id, message }).unwrap();
          }
          _ => {}
        }
      }
      Err(e) => {
        eprintln!("Client error: {}", e);
        tx.send(WorldCommand::RemovePlayer { player_id }).unwrap();
        break;
      }
    }
  }
}

fn main() -> std::io::Result<()> {
  let (tx, rx) = mpsc::channel::<WorldCommand>();
  thread::spawn(move || world_command_thread(rx));
  
  let mut scheduler: Scheduler = Scheduler { tasks: HashMap::new() };
  scheduler.schedule_all_default(tx.clone());
  scheduler.start_scheduler();

  let listener = TcpListener::bind("127.0.0.1:25565")?;
  let mut next_id: u32 = 0;

  for stream in listener.incoming() {
    match stream {
      Ok(stream) => {
        let tx = tx.clone();

        let player_id = next_id;
        next_id += 1;

        thread::spawn(move || {
          handle_client(player_id, stream, tx);
        });
      }
      Err(e) => {
        eprintln!("Connection failed: {}", e);
      }
    }
  }
  Ok(())
}