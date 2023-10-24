use std::{error, thread};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};

use onlinetictactoe_common::game::Player;
use onlinetictactoe_common::packets::{GameId, Packet};
use onlinetictactoe_common::serialization::Serializable;

use crate::Action::{CreateGame, JoinGame};
use crate::utils::{generate_new_game_id, PacketStream};

mod game;
mod utils;



fn main() {
    let mut joinable_games = HashMap::new();
    let listener = TcpListener::bind("[::]:3125").expect("couldn't bind to address");
    for stream in listener.incoming() {
        match handle_stream(stream, &mut joinable_games) {
            Ok(_) => {}
            Err(e) => {
                println!("{}", e);
            }
        }
    }
}

fn handle_stream(stream: std::io::Result<TcpStream>, joinable_games: &mut HashMap<GameId, TcpStream>) -> Result<(), Box<dyn error::Error>> {
    let mut stream = stream?;
    match handle_connection(&mut stream, joinable_games) {
        Ok(action) => {
            match action {
                CreateGame { game_id } => {
                    joinable_games.insert(game_id, stream);
                }
                JoinGame { other } => {
                    thread::spawn(|| {
                        game::start_game(other, stream);
                    });
                }
            }
            Ok(())
        }
        Err(e) => {
            stream.shutdown(Shutdown::Both)?;
            Err(e)
        }
    }
}

enum Action {
    CreateGame { game_id: GameId },
    JoinGame { other: TcpStream },
}

fn handle_connection(stream: &mut TcpStream, joinable_games: &mut HashMap<GameId, TcpStream>) -> Result<Action, Box<dyn error::Error>> {
    let packet = stream.read_packet()?;
    println!("{:?}", packet);
    match packet {
        Packet::CreateGameRequest => {
            let game_id = generate_new_game_id(joinable_games);
            stream.write_packet(&Packet::GameCreatedResponse { game_id, player_icon: Player::X })?;
            Ok(CreateGame { game_id })
        }
        Packet::JoinGameRequest { game_id } => {
            match joinable_games.remove(&game_id) {
                None => {
                    stream.write_packet(&Packet::GameNotFoundResponse {})?;
                    Err("game not found".into())
                }
                Some(other) => {
                    stream.write_packet(&Packet::GameJoinedResponse { player_icon: Player::O })?;
                    Ok(JoinGame { other })
                }
            }
        }
        _ => {
            Err("invalid packet".into())
        }
    }
}
