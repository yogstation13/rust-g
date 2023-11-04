use crate::error::Result;
use rand::prelude::*;
use rand::rngs::StdRng;
use rand::Rng;
use serde::Serialize;
use std::{cmp, fmt};

struct BspLevel {
    level: Level,
    map_subsection_min_size: usize,
    map_subsection_min_room_width: usize,
    map_subsection_min_room_height: usize,
}

byond_fn!(fn bsp_generate(width, height, hash, map_subsection_min_size, map_subsection_min_room_width, map_subsection_min_room_height) {
    bsp_gen(width, height, hash, map_subsection_min_size, map_subsection_min_room_width, map_subsection_min_room_height).ok()
});

fn bsp_gen(
    width_as_str: &str,
    height_as_str: &str,
    hash_as_str: &str,
    map_subsection_min_size_as_str: &str,
    map_subsection_min_room_width_as_str: &str,
    map_subsection_min_room_height_as_str: &str,
) -> Result<String> {
    let default_hash: u64 = rand::thread_rng().gen();
    let width = width_as_str.parse::<usize>()?;
    let height = height_as_str.parse::<usize>()?;
    let map_subsection_min_room_width = map_subsection_min_room_width_as_str.parse::<usize>()?;
    let map_subsection_min_room_height = map_subsection_min_room_height_as_str.parse::<usize>()?;

    //map subsections that the BSP algorithm creates should never be smaller than the minimum desired room size they will contain. This will crash the server
    let map_subsection_min_size = cmp::max(
        map_subsection_min_size_as_str.parse::<usize>()?,
        cmp::max(
            map_subsection_min_room_width,
            map_subsection_min_room_height,
        ) + 1,
    );

    let mut rng: StdRng = SeedableRng::seed_from_u64(
        hash_as_str
            .parse::<usize>()?
            .try_into()
            .unwrap_or(default_hash),
    );

    let level = BspLevel::new(
        width,
        height,
        &mut rng,
        map_subsection_min_size,
        map_subsection_min_room_width,
        map_subsection_min_room_height,
    );

    Ok(serde_json::to_string(&level.rooms)?)
}

impl BspLevel {
    fn new(
        width: usize,
        height: usize,
        rng: &mut StdRng,
        map_subsection_min_size: usize,
        map_subsection_min_room_width: usize,
        map_subsection_min_room_height: usize,
    ) -> Level {
        let level = Level::new(width, height);

        let mut map = BspLevel {
            level,
            map_subsection_min_size,
            map_subsection_min_room_width,
            map_subsection_min_room_height,
        };

        map.place_rooms(rng);

        map.level
    }

    fn place_rooms(&mut self, rng: &mut StdRng) {
        let mut root = Leaf::new(
            0,
            0,
            self.level.width,
            self.level.height,
            self.map_subsection_min_size,
            self.map_subsection_min_room_width,
            self.map_subsection_min_room_height,
        );
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
    min_size: usize,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    min_room_width: usize,
    min_room_height: usize,
    left_child: Option<Box<Leaf>>,
    right_child: Option<Box<Leaf>>,
    room: Option<Room>,
    corridors: Vec<Room>,
}

impl Leaf {
    fn new(
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        min_size: usize,
        min_room_width: usize,
        min_room_height: usize,
    ) -> Self {
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
        self.left_child.is_none() && self.right_child.is_none() 
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

            self.room = Some(Room::new(
                format!("bsp room"),
                x + self.x,
                y + self.y,
                width,
                height,
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

#[derive(Debug, Clone, Copy)]
pub enum TileType {
    Space = 0,
    Floor = 1,
    Wall = 2,
}
pub struct Level {
    width: usize,
    height: usize,
    board: Vec<Vec<usize>>,
    rooms: Vec<Room>,
    increment: usize,
}

impl Level {
    fn new(width: usize, height: usize) -> Self {
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
        self.increment += 1;
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
            write!(f, "\n")?
        }

        Ok(())
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
        (((other.x - self.center.x).pow(2) + (other.y - self.center.y).pow(2)) as f64).sqrt()
            as usize
    }
}
