//READER
pub struct PacketReader<'a> {
  pub index: i16,
  pub buffer: &'a [u8]
}

impl PacketReader<'_> {
  pub fn read_u8(&mut self) -> u8 {
      let read: u8 = self.buffer[self.index as usize];
      self.index += 1;
      return read;
  }

  pub fn read_u16(&mut self) -> u16 {
    let high = self.buffer[self.index as usize] as u16;
    let low = self.buffer[(self.index + 1) as usize] as u16;
    self.index += 2;
    return (high << 8) | low
  }

  pub fn read_string(&mut self) -> String {
      let start_index: usize = self.index as usize;
      let read: String = String::from_utf8(self.buffer[start_index..(start_index + 64)].to_vec()).unwrap();
      self.index += 64;
      return read;
  }
}

pub fn client_player_identification(reader: &mut PacketReader) -> (u8, String, String) {
  let protocol_version = reader.read_u8();
  let username = reader.read_string();
  let key: String = reader.read_string();

  return (protocol_version, username, key)
}

pub fn client_set_block(reader: &mut PacketReader) -> (u16, u16, u16, u8, u8) {
  let x = reader.read_u16();
  let y = reader.read_u16();
  let z = reader.read_u16();
  let mode = reader.read_u8();
  let id = reader.read_u8();

  return (x, y, z, id, mode)
}

pub fn client_position_orientation(reader: &mut PacketReader) -> (u8, u16, u16, u16, u8, u8) {
  let player_id = reader.read_u8(); // Always 255 (self)
  let x = reader.read_u16();
  let y = reader.read_u16();
  let z = reader.read_u16();
  let yaw = reader.read_u8();
  let pitch = reader.read_u8();

  return (player_id, x, y, z, yaw, pitch)
}

pub fn client_chat_message(reader: &mut PacketReader) -> (u8, String) {
  let unused = reader.read_u8();
  let message = reader.read_string();
  return (unused, message)
}

//WRITER
pub struct PacketWriter {
  buffer: Vec<u8>
}

impl PacketWriter {
  pub fn write_u8(&mut self, value: u8) {
      self.buffer.push(value);
  }

  pub fn write_u16(&mut self, value: u16) {
    self.buffer.push((value >> 8) as u8);
    self.buffer.push(value as u8);
  }

  pub fn write_byte_arr(&mut self, mut array: Vec<u8>) {
    array.resize(1024, 0); 
    self.buffer.extend(array);
  }

  pub fn write_string(&mut self, value: &String) {
      let mut bytes = value.clone().into_bytes();
      bytes.resize(64, 0);
      self.buffer.extend(bytes);
  }
}

pub fn server_identification_packet() -> Vec<u8> {
  let mut packet: PacketWriter = PacketWriter { buffer: Vec::new() };
  packet.write_u8(0x00);
  packet.write_u8(7);
  packet.write_string(&String::from("server name"));
  packet.write_string(&String::from("motd"));
  packet.write_u8(0);
  return packet.buffer;
}

pub fn level_initialize() -> Vec<u8> {
  let mut packet: PacketWriter = PacketWriter { buffer: Vec::new() };
  packet.write_u8(0x02);
  return packet.buffer;
}

pub fn level_data_chunk(chunk: Vec<u8>) -> Vec<u8> {
  let mut packet: PacketWriter = PacketWriter { buffer: Vec::new() };
  packet.write_u8(0x03);
  packet.write_u16(chunk.len() as u16);
  packet.write_byte_arr(chunk);
  packet.write_u8(0);
  return packet.buffer;
}

pub fn level_finalize(size_x: usize, size_y: usize, size_z: usize) -> Vec<u8> {
  let mut packet: PacketWriter = PacketWriter { buffer: Vec::new() };
  packet.write_u8(0x04);
  packet.write_u16(size_x as u16);
  packet.write_u16(size_y as u16);
  packet.write_u16(size_z as u16);
  return packet.buffer;
}

pub fn set_block(x: u16, y: u16, z:u16, id: u8) -> Vec<u8> {
  let mut packet: PacketWriter = PacketWriter { buffer: Vec::new() };
  packet.write_u8(0x06);
  packet.write_u16(x);
  packet.write_u16(y);
  packet.write_u16(z);
  packet.write_u8(id);
  return packet.buffer;
}

pub fn message(message: &String) -> Vec<u8> {
  let mut packet: PacketWriter = PacketWriter { buffer: Vec::new() };
  packet.write_u8(0x0d);
  packet.write_u8(0);
  packet.write_string(message);
  return packet.buffer;
}

pub fn spawn_player_packet(
  player_id: u8,
  player_name: &String,
  x: u16,
  y: u16,
  z: u16,
  yaw: u8,
  pitch: u8,
) -> Vec<u8> {
  let mut packet = PacketWriter { buffer: Vec::new() };
  packet.write_u8(0x07);
  packet.write_u8(player_id);
  packet.write_string(player_name);
  packet.write_u16(x);
  packet.write_u16(y);
  packet.write_u16(z);
  packet.write_u8(yaw);
  packet.write_u8(pitch);
  return packet.buffer
}

pub fn teleport_player_packet(
  player_id: u8,
  x: u16,
  y: u16,
  z: u16,
  yaw: u8,
  pitch: u8,
) -> Vec<u8> {
  let mut packet = PacketWriter { buffer: Vec::new() };
  packet.write_u8(0x08);
  packet.write_u8(player_id);
  packet.write_u16(x);
  packet.write_u16(y);
  packet.write_u16(z);
  packet.write_u8(yaw);
  packet.write_u8(pitch);
  return packet.buffer
}

pub fn despawn_player_packet(player_id: u8) -> Vec<u8> {
  let mut packet = PacketWriter { buffer: Vec::new() };
  packet.write_u8(0x0C);
  packet.write_u8(player_id);
  return packet.buffer
}