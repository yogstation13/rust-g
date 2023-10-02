use std::fmt;
use std::fmt::Write;
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
    match random_room_gen(width, height, desired_room_count, hash) {
        Ok(s) => Some(s),
        Err(e) => Some(format!("{e}"))
    }
});
fn random_room_gen(width_as_str: &str,
    height_as_str: &str,
    desired_room_count_as_str: &str,
    hash_as_str: &str,
    )
    -> Result<String>{
    let default_hash: u64 = rand::thread_rng().gen();
    let width = width_as_str.parse::<i32>()?;
    let height = height_as_str.parse::<i32>()?;
    let desired_room_count = desired_room_count_as_str.parse::<i32>()?;

    //let seed: &str = Alphanumeric.sample_string(&mut rand::thread_rng(), 32).as_str();

    let mut rng: StdRng = SeedableRng::seed_from_u64(hash_as_str.parse::<usize>()?.try_into().unwrap_or(default_hash));


    let level = RandomRoomLevel::new(width, height, desired_room_count, &mut rng);


    let mut string = String::new();
    for room in &level.all_rooms {
        let serialized = serde_json::to_string(&room).unwrap();
        let _ = write!(string, "{}", serialized);
    }

    Ok(format!("{}",serde_json::to_string(&level.all_rooms)?))
}

impl RandomRoomLevel {
    fn new(
        width: i32,
        height: i32,
        desired_room_count: i32,
        rng: &mut StdRng,

    ) -> Level {
        let level = Level::new(width, height);

        let mut map = RandomRoomLevel { level };

        map.place_rooms_random(desired_room_count, rng);
        map.level
    }

    fn place_rooms_random(&mut self, desired_room_count: i32, rng: &mut StdRng) {
        let max_rooms = desired_room_count as usize;
        let max_attempts = 15;
        let mut attempts = 0;
        while self.level.all_rooms.iter().filter(|&rm| rm.room_type == 3).count() <= max_rooms && attempts <= max_attempts {
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
            let room = Room::new(format!("ruin room: {}",self.level.all_rooms.iter().filter(|&rm| rm.room_type == 3).count()), x, y, width, height, 3);

            for other_room in &self.level.all_rooms {
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
    width: i32,
    height: i32,
    board: Vec<Vec<i32>>,
    all_rooms: Vec<Room>,
    increment: i32,
    //hash: String,
}

impl Level {
    fn new(
        width: i32,
        height: i32,
    ) -> Self {
        let mut new_level = Level {
            width,
            height,
            board: Vec::new(),
            all_rooms: Vec::new(),
            increment: 0,
        };
        new_level.update_board();
        new_level
    }

    fn update_board(&mut self) -> Vec<Vec<i32>> {
        let mut new_board = Vec::new();
        self.increment+=1;
        for _ in 0..self.height {
            let space_tile = 0;
            //let wall_tile = 1;
            let floor_tile = 5;
            let gen_floor_first = true;

            let mut row = vec![floor_tile; self.width as usize];
            if !gen_floor_first {
                row = vec![space_tile; self.width as usize];
            }
            // if gen_floor_first {
            //     if index == 0 || index == self.height - 1 {
            //         row = vec![wall_tile; self.width as usize];
            //     }

            //     row[0] = wall_tile;
            //     row[self.width as usize - 1] = wall_tile;
            // }

            new_board.push(row);
        }
        for room in &self.all_rooms {
            for row in 0..room.height {
                for col in 0..room.width {
                    let y = (room.y + row) as usize;
                    let x = (room.x + col) as usize;
                    if row == 0 || col == 0 || row == room.height - 1 || col == room.width - 1 {
                        // might just let byond handle the walls
                        new_board[y][x] = 1;
                    } else {
                        new_board[y][x] = room.room_type;
                    }
                }
            }
        }
        self.board = new_board.clone();
        //draw(self, "increments", &self.increment.to_string()).unwrap();
        new_board
    }

    fn add_room(&mut self, room: &Room) {
        // match room.room_type {
        //     2 => self.mandatory_rooms.push(room.clone()),
        //     _ => self.rooms.push(room.clone()),
        // }
        self.all_rooms.push(room.clone());
        self.update_board();

    }

}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in 0..self.height as usize {
            for col in 0..self.width as usize {
                write!(f, "{}", self.board[row][col])?
            }
            // write!(f, "\n")?
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
    fn get_height(&self) -> i32 {
        let height: i32;
        match *self {
            RoomDimensions::Maint3x3 => height = 3,
            RoomDimensions::Maint3x5 => height = 5,
            RoomDimensions::Maint5x3 => height = 3,
            RoomDimensions::Maint5x4 => height = 4,
            RoomDimensions::Maint10x5 => height = 5,
            RoomDimensions::Maint10x10 => height = 10,
        }
        return height + 2
    }

    fn get_width(&self) -> i32 {
        let width: i32;
        match *self {
            RoomDimensions::Maint3x3 => width = 3,
            RoomDimensions::Maint3x5 => width = 3,
            RoomDimensions::Maint5x3 => width = 5,
            RoomDimensions::Maint5x4 => width = 5,
            RoomDimensions::Maint10x5 => width = 10,
            RoomDimensions::Maint10x10 => width = 10,
        }
        return width + 2;
    }
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Point {
    x: i32,
    y: i32,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Room {
    id: String,
    x: i32,
    y: i32,
    x2: i32,
    y2: i32,
    width: i32,
    height: i32,
    center: Point,
    room_type: i32,
}

impl Room {
    pub fn new(id: String, x: i32, y: i32, width: i32, height: i32, room_type: i32) -> Self {
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
            room_type,
        }
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.x <= other.x2 && self.x2 >= other.x && self.y <= other.y2 && self.y2 >= other.y
    }
    pub fn get_distance_to(&self, other: &Point) -> i32 {
        (((other.x - self.center.x).pow(2) + (other.y - self.center.y).pow(2)) as f64).sqrt() as i32
    }
}
