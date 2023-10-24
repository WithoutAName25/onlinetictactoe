use rand::random;
use crate::game::{Grid, Player, Pos};
use crate::serialization::{check_buffer_size, deserialize_flags, Serializable, SerializationError, serialize_flags};

const BASE32_ALPHABET: [char; 32] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H',
    'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P',
    'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X',
    'Y', 'Z', '2', '3', '4', '5', '6', '7',
];

fn get_base32_value(&c: &char) -> Result<u8, ()> {
    match BASE32_ALPHABET.iter().position(|&v| v == c) {
        Some(index) => Ok(index as u8),
        None => Err(()),
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct GameId([u8; 3]);

impl GameId {
    pub fn to_string(&self) -> String {
        let mut result = String::with_capacity(5);
        result.push(BASE32_ALPHABET[(self.0[0] >> 3) as usize]);
        result.push(BASE32_ALPHABET[((self.0[0] & 0b111) << 2 | (self.0[1] >> 6)) as usize]);
        result.push(BASE32_ALPHABET[((self.0[1] & 0b111110) >> 1) as usize]);
        result.push(BASE32_ALPHABET[((self.0[1] & 0b1) << 4 | (self.0[2] >> 4)) as usize]);
        result.push(BASE32_ALPHABET[((self.0[2] & 0b1111) << 1) as usize]);
        result
    }

    pub fn from_string(string: &str) -> Result<Self, ()> {
        if string.len() != 5 {
            return Err(());
        }
        let mut char_values = [0u8; 5];
        for (i, c) in string.chars().enumerate() {
            char_values[i] = get_base32_value(&c)?;
        }
        let mut result = [0u8; 3];
        result[0] = (char_values[0] << 3) | (char_values[1] >> 2);
        result[1] = (char_values[1] << 6) | (char_values[2] << 1) | (char_values[3] >> 4);
        result[2] = (char_values[3] << 4) | (char_values[4] >> 1);
        Ok(GameId(result))
    }

    pub fn random() -> Self {
        GameId(random::<[u8; 3]>())
    }
}

impl Serializable for GameId {
    fn serialize(&self, buffer: &mut [u8]) -> Result<usize, SerializationError> {
        check_buffer_size(buffer, 3)?;
        buffer[0..3].copy_from_slice(&self.0);
        Ok(3)
    }

    fn deserialize(buffer: &[u8]) -> Result<(usize, Self), SerializationError> {
        check_buffer_size(buffer, 3)?;
        let mut result = [0u8; 3];
        result.copy_from_slice(&buffer[0..3]);
        Ok((3, GameId(result)))
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum Packet {
    CreateGameRequest = 1,
    GameCreatedResponse { game_id: GameId, player_icon: Player } = 2,
    JoinGameRequest { game_id: GameId } = 3,
    GameNotFoundResponse = 11,
    GameJoinedResponse { player_icon: Player } = 4,
    GameStartNotification = 5,
    GameEndNotification { winner: Player } = 6,
    GridUpdate { grid: Grid } = 7,
    YourTurnNotification = 8,
    PlaceRequest { p: Pos } = 9,
    PlaceResponse { success: bool } = 10,
}

impl Packet {
    fn type_id(&self) -> u8 {
        unsafe { *(self as *const Self as *const u8) }
    }
}

impl Serializable for Packet {
    fn serialize(&self, buffer: &mut [u8]) -> Result<usize, SerializationError> {
        check_buffer_size(buffer, 1)?;
        let mut offset = 0;
        buffer[offset] = self.type_id();
        offset += 1;
        match self {
            Packet::CreateGameRequest => {}
            Packet::GameCreatedResponse { game_id, player_icon } => {
                offset += game_id.serialize(&mut buffer[offset..])?;
                offset += player_icon.serialize(&mut buffer[offset..])?;
            }
            Packet::JoinGameRequest { game_id } => {
                offset += game_id.serialize(&mut buffer[offset..])?;
            }
            Packet::GameNotFoundResponse => {}
            Packet::GameJoinedResponse { player_icon } => {
                offset += player_icon.serialize(&mut buffer[offset..])?;
            }
            Packet::GameStartNotification => {}
            Packet::GameEndNotification { winner } => {
                offset += winner.serialize(&mut buffer[offset..])?;
            }
            Packet::GridUpdate { grid } => {
                offset += grid.serialize(&mut buffer[offset..])?;
            }
            Packet::YourTurnNotification => {}
            Packet::PlaceRequest { p } => {
                offset += p.serialize(&mut buffer[offset..])?;
            }
            Packet::PlaceResponse { success } => {
                offset += serialize_flags(&mut buffer[offset..], &[success])?;
            }
        }
        Ok(offset)
    }

    fn deserialize(buffer: &[u8]) -> Result<(usize, Packet), SerializationError> {
        check_buffer_size(buffer, 1)?;
        let mut offset = 0;
        let type_id = buffer[offset];
        offset += 1;
        match type_id {
            1 => {
                Ok(Packet::CreateGameRequest)
            }
            2 => {
                let (size, game_id) = GameId::deserialize(&buffer[offset..])?;
                offset += size;
                let (size, player_icon) = Player::deserialize(&buffer[offset..])?;
                offset += size;
                Ok(Packet::GameCreatedResponse {
                    game_id,
                    player_icon,
                })
            }
            3 => {
                let (size, game_id) = GameId::deserialize(&buffer[offset..])?;
                offset += size;
                Ok(Packet::JoinGameRequest {
                    game_id,
                })
            }
            11 => {
                Ok(Packet::GameNotFoundResponse)
            }
            4 => {
                let (size, player_icon) = Player::deserialize(&buffer[offset..])?;
                offset += size;
                Ok(Packet::GameJoinedResponse {
                    player_icon,
                })
            }
            5 => {
                Ok(Packet::GameStartNotification)
            }
            6 => {
                let (size, winner) = Player::deserialize(&buffer[offset..])?;
                offset += size;
                Ok(Packet::GameEndNotification {
                    winner,
                })
            }
            7 => {
                let (size, grid) = Grid::deserialize(&buffer[offset..])?;
                offset += size;
                Ok(Packet::GridUpdate {
                    grid,
                })
            }
            8 => {
                Ok(Packet::YourTurnNotification)
            }
            9 => {
                let (size, p) = Pos::deserialize(&buffer[offset..])?;
                offset += size;
                Ok(Packet::PlaceRequest {
                    p,
                })
            }
            10 => {
                let (size, [success]) = deserialize_flags(&buffer[offset..])?;
                offset += size;
                Ok(Packet::PlaceResponse {
                    success,
                })
            }
            _ => {
                Err(
                    SerializationError {
                        message: "Wrong packet id".to_string()
                    }
                )
            }
        }.map(|packet| (offset, packet))
    }
}
