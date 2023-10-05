use std::{fmt, cmp};
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use rand::rngs::StdRng;
use rand::Rng;
use serde::Serialize;
use crate::error::Result;

struct BspLevel {
    level: Level,
    map_subsection_min_size: i32,
    map_subsection_min_room_width: i32,
    map_subsection_min_room_height: i32,
}


byond_fn!(fn bsp_generate(width, height, hash, map_subsection_min_size, map_subsection_min_room_width, map_subsection_min_room_height) {
    bsp_gen(width, height, hash, map_subsection_min_size, map_subsection_min_room_width, map_subsection_min_room_height).ok()
});

fn bsp_gen(width_as_str: &str,
    height_as_str: &str,
    hash_as_str: &str,
    map_subsection_min_size_as_str: &str,
    map_subsection_min_room_width_as_str: &str,
    map_subsection_min_room_height_as_str: &str)
    -> Result<String>{
    let default_hash: u64 = rand::thread_rng().gen();
    let width = width_as_str.parse::<i32>()?;
    let height = height_as_str.parse::<i32>()?;
    let map_subsection_min_room_width = map_subsection_min_room_width_as_str.parse::<i32>()?;
    let map_subsection_min_room_height = map_subsection_min_room_height_as_str.parse::<i32>()?;

    let map_subsection_min_size = cmp::max(
        map_subsection_min_size_as_str.parse::<i32>()?,
        cmp::max(map_subsection_min_room_width, map_subsection_min_room_height) + 1
    );

    //let seed: &str = Alphanumeric.sample_string(&mut rand::thread_rng(), 32).as_str();

    let mut rng: StdRng = SeedableRng::seed_from_u64(hash_as_str.parse::<usize>()?.try_into().unwrap_or(default_hash));


    let level = BspLevel::new(width, height, &mut rng, map_subsection_min_size, map_subsection_min_room_width, map_subsection_min_room_height);

    Ok(serde_json::to_string(&level.all_rooms)?)
}

impl BspLevel {
    fn new(
        width: i32,
        height: i32,
        rng: &mut StdRng,
        map_subsection_min_size: i32,
        map_subsection_min_room_width: i32,
        map_subsection_min_room_height: i32,
    ) -> Level {
        let level = Level::new(width, height);

        let mut map = BspLevel { level, map_subsection_min_size, map_subsection_min_room_width, map_subsection_min_room_height };

        map.place_rooms(rng);

        map.level
    }

    fn place_rooms(&mut self, rng: &mut StdRng) {
        let mut root = Leaf::new(0, 0, self.level.width, self.level.height, self.map_subsection_min_size, self.map_subsection_min_room_width,self.map_subsection_min_room_height);
        root.generate(rng);

        root.create_rooms(rng);

        for leaf in root.iter() {
            if leaf.is_leaf() {
                if let Some(room) = leaf.get_room() {

                    self.level.add_room(&room);
                }
            }

            for corridor in &leaf.corridors {
                self.level.add_room(&corridor);
            }
        }

    }
}

struct Leaf {
    min_size: i32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    min_room_width: i32,
    min_room_height: i32,
    left_child: Option<Box<Leaf>>,
    right_child: Option<Box<Leaf>>,
    room: Option<Room>,
    corridors: Vec<Room>,
}

impl Leaf {
    fn new(x: i32, y: i32, width: i32, height: i32, min_size: i32, min_room_width: i32, min_room_height: i32,) -> Self {
        Leaf {
            min_size,
            x,
            y,
            width,
            height,
            min_room_width,
            min_room_height,
            left_child: None,
            right_child: None,
            room: None,
            corridors: vec![],
        }
    }

    fn is_leaf(&self) -> bool {
        match self.left_child {
            None => match self.right_child {
                None => true,
                Some(_) => false,
            },
            Some(_) => false,
        }
    }

    fn generate(&mut self, rng: &mut StdRng) {
        if self.is_leaf() {
            if self.split(rng) {
                self.left_child.as_mut().unwrap().generate(rng);
                self.right_child.as_mut().unwrap().generate(rng);
            }
        }
    }

    fn split(&mut self, rng: &mut StdRng) -> bool {
        // if width >25% height, split vertically
        // if height >25% width, split horz
        // otherwise random

        // this is the random choice
        let mut split_horz = match rng.gen_range(0..2) {
            0 => false,
            _ => true,
        };

        // then override with width/height check
        if self.width > self.height && (self.width as f32 / self.height as f32) >= 1.25 {
            split_horz = false;
        } else if self.height > self.width && (self.height as f32 / self.width as f32) >= 1.25 {
            split_horz = true;
        }

        let max = match split_horz {
            true => self.height - self.min_size,
            false => self.width - self.min_size,
        };

        // the current area is small enough, so stop splitting
        if max <= self.min_size {
            return false;
        }

        let split_pos = rng.gen_range(self.min_size..max);
        if split_horz {
            self.left_child = Some(Box::new(Leaf::new(
                self.x,
                self.y,
                self.width,
                split_pos,
                self.min_size,
                self.min_room_width,
                self.min_room_height,
            )));
            self.right_child = Some(Box::new(Leaf::new(
                self.x,
                self.y + split_pos,
                self.width,
                self.height - split_pos,
                self.min_size,
                self.min_room_width,
                self.min_room_height,
            )));
        } else {
            self.left_child = Some(Box::new(Leaf::new(
                self.x,
                self.y,
                split_pos,
                self.height,
                self.min_size,
                self.min_room_width,
                self.min_room_height,
            )));
            self.right_child = Some(Box::new(Leaf::new(
                self.x + split_pos,
                self.y,
                self.width - split_pos,
                self.height,
                self.min_size,
                self.min_room_width,
                self.min_room_height,
            )));
        }

        true
    }

    fn create_rooms(&mut self, rng: &mut StdRng) {
        if let Some(ref mut room) = self.left_child {
            room.as_mut().create_rooms(rng);
        };

        if let Some(ref mut room) = self.right_child {
            room.as_mut().create_rooms(rng);
        };

        // if last level, add a room
        if self.is_leaf() {
            let width = rng.gen_range(self.min_room_width..=self.width);
            let height = rng.gen_range(self.min_room_height..=self.height);
            let x = rng.gen_range(0..=self.width - width);
            let y = rng.gen_range(0..=self.height - height);
            let choices = [0, 4];
            let weights = [1, 4];
            let dist = WeightedIndex::new(&weights).unwrap();
            let mut rng = thread_rng();
            let room_layout = choices[dist.sample(&mut rng)];

            self.room = Some(Room::new(
                format!("extra room"),
                x + self.x,
                y + self.y,
                width,
                height,
                room_layout,
            ));
        }

    }

    fn get_room(&self) -> Option<Room> {
        if self.is_leaf() {
            return self.room.clone();
        }

        let mut left_room: Option<Room> = None;
        let mut right_room: Option<Room> = None;

        if let Some(ref room) = self.left_child {
            left_room = room.get_room();
        }

        if let Some(ref room) = self.right_child {
            right_room = room.get_room();
        }

        match (left_room, right_room) {
            (None, None) => None,
            (Some(room), _) => Some(room),
            (_, Some(room)) => Some(room),
        }
    }

    fn iter(&self) -> LeafIterator {
        LeafIterator::new(&self)
    }
}

struct LeafIterator<'a> {
    current_node: Option<&'a Leaf>,
    right_nodes: Vec<&'a Leaf>,
}

impl<'a> LeafIterator<'a> {
    fn new(root: &'a Leaf) -> LeafIterator<'a> {
        let mut iter = LeafIterator {
            right_nodes: vec![],
            current_node: None,
        };

        iter.add_subtrees(root);
        iter
    }

    // set the current node to the one provided
    // and add any child leaves to the node vec
    fn add_subtrees(&mut self, node: &'a Leaf) {
        if let Some(ref left) = node.left_child {
            self.right_nodes.push(&*left);
        }
        if let Some(ref right) = node.right_child {
            self.right_nodes.push(&*right);
        }

        self.current_node = Some(node);
    }
}

impl<'a> Iterator for LeafIterator<'a> {
    type Item = &'a Leaf;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.current_node.take();
        if let Some(rest) = self.right_nodes.pop() {
            self.add_subtrees(rest);
        }

        match result {
            Some(leaf) => Some(&*leaf),
            None => None,
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
