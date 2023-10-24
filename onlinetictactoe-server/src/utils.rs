use std::collections::HashMap;
use std::error;
use std::io::{Read, Write};
use std::net::TcpStream;
use onlinetictactoe_common::packets::{GameId, Packet};
use onlinetictactoe_common::serialization::Serializable;

pub trait PacketStream {
    fn write_packet(&mut self, packet: &Packet) -> Result<(), Box<dyn error::Error>>;
    fn read_packet(&mut self) -> Result<Packet, Box<dyn error::Error>>;
}

impl PacketStream for TcpStream {
    fn write_packet(&mut self, packet: &Packet) -> Result<(), Box<dyn error::Error>> {
        let mut buf = [0; 1024];
        let bytes_written = packet.serialize(&mut buf)?;
        self.write(&buf[0..bytes_written])?;
        Ok(())
    }

    fn read_packet(&mut self) -> Result<Packet, Box<dyn error::Error>> {
        let mut buf = [0; 1024];
        let bytes_read = self.read(&mut buf)?;
        let (_, packet) = Packet::deserialize(&buf[0..bytes_read])?;
        Ok(packet)
    }
}

pub fn generate_new_game_id(joinable_games: &HashMap<GameId, TcpStream>) -> GameId {
    let mut game_id;
    while {
        game_id = GameId::random();
        joinable_games.contains_key(&game_id)
    } {}
    game_id
}