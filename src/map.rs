use crate::constants::*;
use crate::object::*;
use rand::Rng;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;
use serde::{Deserialize, Serialize};
use std::cmp;
use tcod::colors::*;

pub type Map = Vec<Vec<Tile>>;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Tile {
    pub blocked: bool,
    pub explored: bool,
    pub block_sight: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile {
            blocked: false,
            explored: false,
            block_sight: false,
        }
    }

    pub fn wall() -> Self {
        Tile {
            blocked: true,
            explored: false,
            block_sight: true,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h,
        }
    }

    pub fn center(&self) -> (i32, i32) {
        let center_x = (self.x1 + self.x2) / 2;
        let center_y = (self.y1 + self.y2) / 2;
        (center_x, center_y)
    }

    pub fn intersects_with(&self, other: &Rect) -> bool {
        (self.x1 <= other.x2)
            && (self.x2 >= other.x1)
            && (self.y1 <= other.y2)
            && (self.y2 >= other.y1)
    }
}

pub fn is_blocked(x: i32, y: i32, map: &Map, objects: &[Object]) -> bool {
    if map[x as usize][y as usize].blocked {
        return true;
    }

    objects
        .iter()
        .any(|object| object.blocks && object.pos() == (x, y))
}

pub fn make_map(objects: &mut Vec<Object>) -> Map {
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];

    objects.truncate(1);

    let mut rooms = vec![];

    for _ in 0..MAX_ROOMS {
        let w = rand::rng().random_range(ROOM_MIN_SIZE..(ROOM_MAX_SIZE + 1));
        let h = rand::rng().random_range(ROOM_MIN_SIZE..(ROOM_MAX_SIZE + 1));

        let x = rand::rng().random_range(0..(MAP_WIDTH - w));
        let y = rand::rng().random_range(0..(MAP_HEIGHT - h));

        let new_room = Rect::new(x, y, w, h);

        let failed = rooms
            .iter()
            .any(|other_room| new_room.intersects_with(other_room));

        if !failed {
            create_room(new_room, &mut map);
            place_objects(new_room, &map, objects);

            let (new_x, new_y) = new_room.center();

            if rooms.is_empty() {
                objects[PLAYER].set_pos(new_x, new_y);
            } else {
                let (prev_x, prev_y) = rooms[rooms.len() - 1].center();

                if rand::random() {
                    create_h_tunnel(prev_x, new_x, prev_y, &mut map);
                    create_v_tunnel(prev_y, new_y, new_x, &mut map);
                } else {
                    create_v_tunnel(prev_y, new_y, prev_x, &mut map);
                    create_h_tunnel(prev_x, new_x, new_y, &mut map);
                }
            }

            rooms.push(new_room);
        }
    }

    let (last_room_x, last_room_y) = rooms[rooms.len() - 1].center();
    let mut stairs = Object::new(last_room_x, last_room_y, '<', WHITE, "stairs", false);
    stairs.always_visible = true;
    objects.push(stairs);

    map
}

fn create_room(room: Rect, map: &mut Map) {
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            map[x as usize][y as usize] = Tile::empty();
        }
    }
}

fn create_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
    for x in cmp::min(x1, x2)..(cmp::max(x1, x2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn create_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
    for y in cmp::min(y1, y2)..(cmp::max(y1, y2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn place_objects(room: Rect, map: &Map, objects: &mut Vec<Object>) {
    let mut rng = rand::rng();
    let num_monsters = rng.random_range(0..(MAX_ROOM_MONSTERS + 1));
    let num_items = rng.random_range(0..(MAX_ROOM_ITEMS + 1));

    let monster_choices = ["orc", "troll"];
    let monster_weights = [4, 1];
    let monster_dist = WeightedIndex::new(&monster_weights).unwrap();

    let item_choices = [
        Item::Heal,
        Item::Lightning,
        Item::Fireball,
        Item::Confuse,
        Item::Blink,
        Item::Freeze,
        Item::Sword,
        Item::Shield,
    ];
    let item_weights = [4, 3, 3, 3, 3, 3, 2, 2];
    let item_dist = WeightedIndex::new(&item_weights).unwrap();

    for _ in 0..num_monsters {
        let x = rng.random_range((room.x1 + 1)..room.x2);
        let y = rng.random_range((room.y1 + 1)..room.y2);

        if !is_blocked(x, y, map, objects) {
            let mut monster = match monster_choices[monster_dist.sample(&mut rng)] {
                "orc" => {
                    let mut orc = Object::new(x, y, 'o', DESATURATED_GREEN, "orc", true);
                    orc.fighter = Some(Fighter {
                        base_max_hp: 20,
                        hp: 20,
                        base_defense: 0,
                        base_power: 4,
                        xp: 35,
                        on_death: DeathCallback::Monster,
                    });
                    orc.ai = Some(Ai::Basic);
                    orc
                }
                "troll" => {
                    let mut troll = Object::new(x, y, 'T', DARKER_GREEN, "troll", true);
                    troll.fighter = Some(Fighter {
                        base_max_hp: 30,
                        hp: 30,
                        base_defense: 2,
                        base_power: 8,
                        xp: 100,
                        on_death: DeathCallback::Monster,
                    });
                    troll.ai = Some(Ai::Basic);
                    troll
                }
                _ => unreachable!(),
            };
            monster.alive = true;
            objects.push(monster);
        }
    }

    for _ in 0..num_items {
        let x = rand::rng().random_range((room.x1 + 1)..room.x2);
        let y = rand::rng().random_range((room.y1 + 1)..room.y2);

        if !is_blocked(x, y, map, objects) {
            let mut item = match item_choices[item_dist.sample(&mut rng)] {
                Item::Heal => {
                    let mut object = Object::new(x, y, '!', VIOLET, "healing potion", false);
                    object.item = Some(Item::Heal);
                    object
                }
                Item::Lightning => {
                    let mut object =
                        Object::new(x, y, '#', LIGHT_YELLOW, "scroll of lightning bolt", false);
                    object.item = Some(Item::Lightning);
                    object
                }
                Item::Fireball => {
                    let mut object =
                        Object::new(x, y, '#', LIGHT_YELLOW, "scroll of fireball", false);
                    object.item = Some(Item::Fireball);
                    object
                }
                Item::Confuse => {
                    let mut object =
                        Object::new(x, y, '#', LIGHT_YELLOW, "scroll of confusion", false);
                    object.item = Some(Item::Confuse);
                    object
                }
                Item::Blink => {
                    let mut object = Object::new(x, y, '#', LIGHT_YELLOW, "scroll of blink", false);
                    object.item = Some(Item::Blink);
                    object
                }
                Item::Freeze => {
                    let mut object =
                        Object::new(x, y, '#', LIGHT_YELLOW, "scroll of freeze", false);
                    object.item = Some(Item::Freeze);
                    object
                }
                Item::Sword => {
                    let mut object = Object::new(x, y, '/', SKY, "sword", false);
                    object.item = Some(Item::Sword);
                    object.equipment = Some(Equipment {
                        slot: Slot::RightHand,
                        equipped: false,
                        max_hp_bonus: 0,
                        defense_bonus: 0,
                        power_bonus: 3,
                    });
                    object
                }
                Item::Shield => {
                    let mut object = Object::new(x, y, '[', DARKER_ORANGE, "shield", false);
                    object.item = Some(Item::Shield);
                    object.equipment = Some(Equipment {
                        slot: Slot::LeftHand,
                        equipped: false,
                        max_hp_bonus: 0,
                        defense_bonus: 1,
                        power_bonus: 0,
                    });
                    object
                }
            };
            item.always_visible = true;
            objects.push(item);
        }
    }
}
