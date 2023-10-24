use crate::serialization::{check_buffer_size, Serializable, SerializationError};

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum Player {
    NONE = 0,
    X = 1,
    O = 2,
}

impl Player {
    pub fn id(&self) -> u8 {
        unsafe { *(self as *const Self as *const u8) }
    }

    pub fn from_id(id: u8) -> Result<Self, SerializationError> {
        match id {
            0 => Ok(Player::NONE),
            1 => Ok(Player::X),
            2 => Ok(Player::O),
            _ => Err(SerializationError {
                message: "Invalid player".to_string(),
            }),
        }
    }
}

impl Serializable for Player {
    fn serialize(&self, buffer: &mut [u8]) -> Result<usize, SerializationError> {
        check_buffer_size(buffer, 1)?;
        buffer[0] = self.id();
        Ok(1)
    }

    fn deserialize(buffer: &[u8]) -> Result<(usize, Self), SerializationError> where Self: Sized {
        check_buffer_size(buffer, 1)?;
        Ok((1, Player::from_id(buffer[0])?))
    }
}

#[derive(Debug)]
pub struct Grid {
    pub cells: [[Player; 3]; 3],
}

impl Serializable for Grid {
    fn serialize(&self, buffer: &mut [u8]) -> Result<usize, SerializationError> {
        check_buffer_size(buffer, 3)?;
        for (i, row) in self.cells.iter().enumerate() {
            let mut value = 0;
            for (j, cell) in row.iter().enumerate() {
                value |= cell.id() << (j * 2);
            }
            buffer[i] = value;
        }
        Ok(3)
    }

    fn deserialize(buffer: &[u8]) -> Result<(usize, Self), SerializationError> where Self: Sized {
        check_buffer_size(buffer, 3)?;
        let mut cells = [[Player::NONE; 3]; 3];
        for (i, row) in cells.iter_mut().enumerate() {
            let value = buffer[i];
            for (j, cell) in row.iter_mut().enumerate() {
                *cell = Player::from_id((value >> (j * 2)) & 0b11)?;
            }
        }
        Ok((3, Grid { cells }))
    }
}

#[derive(Debug)]
pub struct Pos {
    pub x: u8,
    pub y: u8,
}

impl Serializable for Pos {
    fn serialize(&self, buffer: &mut [u8]) -> Result<usize, SerializationError> {
        check_buffer_size(buffer, 1)?;
        buffer[0] = self.x << 4 | (self.y & 0x0F);
        Ok(1)
    }

    fn deserialize(buffer: &[u8]) -> Result<(usize, Self), SerializationError> where Self: Sized {
        check_buffer_size(buffer, 1)?;
        Ok((1, Pos {
            x: buffer[0] >> 4,
            y: buffer[0] & 0x0F,
        }))
    }
}