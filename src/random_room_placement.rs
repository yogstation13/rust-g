use std::fmt;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use rand::rngs::StdRng;
use rand::Rng;
use serde::Serialize;
use crate::error::Result;

struct RandomRoomLevel {
    level: Level,
}

byond_fn!(fn random_room_generate(width, height, desired_room_count, hash) {
    random_room_gen(width, height, desired_room_count, hash).ok()
});

fn random_room_gen(width_as_str: &str,
    height_as_str: &str,
    desired_room_count_as_str: &str,
    hash_as_str: &str,
    )
    -> Result<String>{
    let default_hash: u64 = rand::thread_rng().gen();
    let width = width_as_str.parse::<usize>()?;
    let height = height_as_str.parse::<usize>()?;
    let desired_room_count = desired_room_count_as_str.parse::<usize>()?;

    let mut rng: StdRng = SeedableRng::seed_from_u64(hash_as_str.parse::<usize>()?.try_into().unwrap_or(default_hash));


    let level = RandomRoomLevel::new(width, height, desired_room_count, &mut rng);

    Ok(serde_json::to_string(&level.rooms)?)
}

impl RandomRoomLevel {
    fn new(
        width: usize,
        height: usize,
        desired_room_count: usize,
        rng: &mut StdRng,
    ) -> Level {
        let level = Level::new(width, height);

        let mut map = RandomRoomLevel { level };

        map.place_rooms_random(desired_room_count, rng);
        map.level
    }

    fn place_rooms_random(&mut self, desired_room_count: usize, rng: &mut StdRng) {
        let max_rooms = desired_room_count as usize;
        let max_attempts = 15;
        let mut attempts = 0;
        while self.level.rooms.len() <= max_rooms && attempts <= max_attempts {
            attempts += 1;
            let mut x = rng.gen_range(0..self.level.width);
            let mut y = rng.gen_range(0..self.level.height);

            let choices = [
                RoomDimensions::Maint3x3,
                RoomDimensions::Maint3x5,
                RoomDimensions::Maint5x3,
                RoomDimensions::Maint5x4,
                RoomDimensions::Maint10x5,
                RoomDimensions::Maint10x10,
            ];
            let weights = [4, 3, 4, 3, 2, 1];
            let dist = WeightedIndex::new(&weights).unwrap();
            //let mut rng = thread_rng();
            let room_layout = &choices[dist.sample(rng)];
            let width = room_layout.get_width();
            let height = room_layout.get_height();

            if x + width > self.level.width {
                x = self.level.width - width;
            }

            if y + height > self.level.height {
                y = self.level.height - height;
            }

            let mut collides = false;
            let room = Room::new(format!("ruin room: {}", self.level.rooms.len()), x, y, width, height);

            for other_room in &self.level.rooms {
                if room.intersects(&other_room){
                    collides = true;
                    break;
                }
            }

            if !collides {
                self.level.add_room(&room);
                attempts = 0;
            }
        }
    }
}

pub struct Level {
    width: usize,
    height: usize,
    board: Vec<Vec<usize>>,
    rooms: Vec<Room>,
    increment: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum TileType {
    Space = 0,
    Floor = 1,
    Wall = 2,
}

impl Level {
    fn new(
        width: usize,
        height: usize,
    ) -> Self {
        let mut new_level = Level {
            width,
            height,
            board: Vec::new(),
            rooms: Vec::new(),
            increment: 0,
        };
        new_level.update_board();
        new_level
    }

    fn update_board(&mut self) -> Vec<Vec<usize>> {
        let mut new_board = Vec::new();
        self.increment+=1;
        for _ in 0..self.height {
            let gen_floor_first = true;

            let mut row = vec![TileType::Floor as usize; self.width as usize];
            if !gen_floor_first {
                row = vec![TileType::Space as usize; self.width as usize];
            }


            new_board.push(row);
        }
        for room in &self.rooms {
            for row in 0..room.height {
                for col in 0..room.width {
                    let y = (room.y + row) as usize;
                    let x = (room.x + col) as usize;
                    if row == 0 || col == 0 || row == room.height - 1 || col == room.width - 1 {
                        // might just let byond handle the walls
                        new_board[y][x] = TileType::Wall as usize;
                    } else {
                        new_board[y][x] = TileType::Floor as usize;
                    }
                }
            }
        }
        self.board = new_board.clone();
        new_board
    }

    fn add_room(&mut self, room: &Room) {
        self.rooms.push(room.clone());
        self.update_board();

    }

}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in 0..self.height as usize {
            for col in 0..self.width as usize {
                write!(f, "{}", self.board[row][col])?
            }
        }

        Ok(())
    }
}


#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub enum RoomDimensions {
    Maint3x3,
    Maint3x5,
    Maint5x3,
    Maint5x4,
    Maint10x5,
    Maint10x10,
}
impl RoomDimensions {
    fn get_height(&self) -> usize {
        return match *self {
            RoomDimensions::Maint3x3 => 3,
            RoomDimensions::Maint3x5 => 5,
            RoomDimensions::Maint5x3 => 3,
            RoomDimensions::Maint5x4 => 4,
            RoomDimensions::Maint10x5 => 5,
            RoomDimensions::Maint10x10 => 10,
        } + 2 //add 2 because the dimensions are equal to the inside of the room, and we need the dimensions with the walls in mind
    }

    fn get_width(&self) -> usize {
        return match *self {
            RoomDimensions::Maint3x3 => 3,
            RoomDimensions::Maint3x5 => 3,
            RoomDimensions::Maint5x3 => 5,
            RoomDimensions::Maint5x4 => 5,
            RoomDimensions::Maint10x5 => 10,
            RoomDimensions::Maint10x10 => 10,
        } + 2; //add 2 because the dimensions are equal to the inside of the room, and we need the dimensions with the walls in mind
    }
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Point {
    x: usize,
    y: usize,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Room {
    id: String,
    x: usize,
    y: usize,
    x2: usize,
    y2: usize,
    width: usize,
    height: usize,
    center: Point,
}

impl Room {
    pub fn new(id: String, x: usize, y: usize, width: usize, height: usize) -> Self {
        Room {
            id,
            x,
            y,
            x2: x + width,
            y2: y + height,
            width,
            height,
            center: Point {
                x: x + (width / 2),
                y: y + (height / 2),
            },
        }
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.x <= other.x2 && self.x2 >= other.x && self.y <= other.y2 && self.y2 >= other.y
    }
    pub fn get_distance_to(&self, other: &Point) -> usize {
        (((other.x - self.center.x).pow(2) + (other.y - self.center.y).pow(2)) as f64).sqrt() as usize
    }
}
