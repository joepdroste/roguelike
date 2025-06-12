use crate::constants::*;
use crate::game::{Game, Tcod};
use crate::map::is_blocked;
use crate::object::{Ai, Item, Object, Slot};
use crate::ui::{menu, render_all};
use tcod::colors::*;
use tcod::input::{self, Event};

pub fn create_item(item_type: Item, x: i32, y: i32) -> Object {
    match item_type {
        Item::Heal => {
            let mut object = Object::new(x, y, '#', LIGHT_YELLOW, "healing potion", false);
            object.item = Some(Item::Heal);
            object
        }
        Item::Lightning => {
            let mut object =
                Object::new(x, y, '#', LIGHT_YELLOW, "scroll of lightning bolt", false);
            object.item = Some(Item::Lightning);
            object
        }
        Item::Confuse => {
            let mut object = Object::new(x, y, '#', LIGHT_YELLOW, "scroll of confusion", false);
            object.item = Some(Item::Confuse);
            object
        }
        Item::Fireball => {
            let mut object = Object::new(x, y, '#', LIGHT_YELLOW, "scroll of fireball", false);
            object.item = Some(Item::Fireball);
            object
        }
        Item::Blink => {
            let mut object = Object::new(x, y, '#', LIGHT_YELLOW, "scroll of blink", false);
            object.item = Some(Item::Blink);
            object
        }
        Item::Freeze => {
            let mut object = Object::new(x, y, '#', LIGHT_YELLOW, "scroll of freeze", false);
            object.item = Some(Item::Freeze);
            object
        }
        Item::Sword => {
            let mut object = Object::new(x, y, '/', SKY, "sword", false);
            object.item = Some(Item::Sword);
            object.equipment = Some(crate::object::Equipment {
                slot: Slot::RightHand,
                equipped: false,
                max_hp_bonus: 0,
                defense_bonus: 0,
                power_bonus: 1000,
            });
            object
        }
        Item::Shield => {
            let mut object = Object::new(x, y, '/', DARKER_ORANGE, "shield", false);
            object.item = Some(Item::Shield);
            object.equipment = Some(crate::object::Equipment {
                slot: Slot::LeftHand,
                equipped: false,
                max_hp_bonus: 0,
                defense_bonus: 1000,
                power_bonus: 0,
            });
            object
        }
    }
}

pub fn item_spawner_menu(root: &mut tcod::console::Root) -> Option<Item> {
    let options = vec![
        "Healing Potion",
        "Lightning Bolt Scroll",
        "Freeze Scroll",
        "Confusion Scroll",
        "Fireball Scroll",
        "Blink Scroll",
    ];

    let item_types = vec![
        Item::Heal,
        Item::Lightning,
        Item::Freeze,
        Item::Confuse,
        Item::Fireball,
        Item::Blink,
    ];

    let selected_index = menu("Choose an item to spawn:", &options, INVENTORY_WIDTH, root);

    if let Some(index) = selected_index {
        Some(item_types[index])
    } else {
        None
    }
}

pub fn spawn_item_at_player(game: &mut Game, objects: &mut Vec<Object>, item_type: Item) {
    let (player_x, player_y) = objects[PLAYER].pos();

    let directions = vec![
        (0, 0),
        (0, -1),
        (0, 1),
        (-1, 0),
        (1, 0),
        (-1, -1),
        (1, -1),
        (-1, 1),
        (1, 1),
    ];

    for (dx, dy) in directions {
        let spawn_x = player_x + dx;
        let spawn_y = player_y + dy;

        if spawn_x >= 0
            && spawn_x < MAP_WIDTH
            && spawn_y >= 0
            && spawn_y < MAP_HEIGHT
            && !is_blocked(spawn_x, spawn_y, &game.map, objects)
        {
            let item = create_item(item_type, spawn_x, spawn_y);
            game.messages.add(
                format!("Spawned {} at ({}, {})", item.name, spawn_x, spawn_y),
                LIGHT_CYAN,
            );
            objects.push(item);
            return;
        }
    }

    game.messages.add("No valid position to spawn item!", RED);
}

pub fn drop_item(inventory_id: usize, game: &mut Game, objects: &mut Vec<Object>) {
    let mut item = game.inventory.remove(inventory_id);
    if item.equipment.is_some() {
        item.dequip(&mut game.messages);
    }
    item.set_pos(objects[PLAYER].x, objects[PLAYER].y);
    game.messages
        .add(format!("You dropped a {}.", item.name), YELLOW);
    objects.push(item);
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UseResult {
    UsedUp,
    Cancelled,
    UsedAndKept,
}

pub fn use_item(inventory_id: usize, tcod: &mut Tcod, game: &mut Game, objects: &mut [Object]) {
    use Item::*;

    if let Some(item) = game.inventory[inventory_id].item {
        let on_use = match item {
            Heal => cast_heal,
            Lightning => cast_lightning,
            Confuse => cast_confuse,
            Fireball => cast_fireball,
            Blink => cast_blink,
            Freeze => cast_freeze,
            Sword => toggle_equipment,
            Shield => toggle_equipment,
        };
        match on_use(inventory_id, tcod, game, objects) {
            UseResult::UsedUp => {
                game.inventory.remove(inventory_id);
            }
            UseResult::Cancelled => {
                game.messages.add("Cancelled", WHITE);
            }
            UseResult::UsedAndKept => {}
        }
    } else {
        game.messages.add(
            format!("The {} cannot be used.", game.inventory[inventory_id].name),
            WHITE,
        );
    }
}

pub fn get_equipped_in_slot(slot: Slot, inventory: &[Object]) -> Option<usize> {
    for (inventory_id, item) in inventory.iter().enumerate() {
        if item
            .equipment
            .as_ref()
            .map_or(false, |e| e.equipped && e.slot == slot)
        {
            return Some(inventory_id);
        }
    }
    None
}

fn toggle_equipment(
    inventory_id: usize,
    _tcod: &mut Tcod,
    game: &mut Game,
    _objects: &mut [Object],
) -> UseResult {
    let equipment = match game.inventory[inventory_id].equipment {
        Some(equipment) => equipment,
        None => return UseResult::Cancelled,
    };
    if equipment.equipped {
        game.inventory[inventory_id].dequip(&mut game.messages);
    } else {
        if let Some(current) = get_equipped_in_slot(equipment.slot, &game.inventory) {
            game.inventory[current].dequip(&mut game.messages);
        }
        game.inventory[inventory_id].equip(&mut game.messages);
    }
    UseResult::UsedAndKept
}

fn cast_heal(
    _inventory_id: usize,
    _tcod: &mut Tcod,
    game: &mut Game,
    objects: &mut [Object],
) -> UseResult {
    if let Some(fighter) = objects[PLAYER].fighter {
        if fighter.hp == objects[PLAYER].max_hp(game) {
            game.messages.add("You are already at full health.", RED);
            return UseResult::Cancelled;
        }
        game.messages
            .add("Your wounds start to feel better!", LIGHT_VIOLET);
        objects[PLAYER].heal(HEAL_AMOUNT, game);
        return UseResult::UsedUp;
    }
    UseResult::Cancelled
}

fn cast_lightning(
    _inventory_id: usize,
    tcod: &mut Tcod,
    game: &mut Game,
    objects: &mut [Object],
) -> UseResult {
    let monster_id = closest_monster(tcod, objects, LIGHTNING_RANGE);
    if let Some(monster_id) = monster_id {
        game.messages.add(
            format!(
                "A lightning bolt strikes the {} with a loud thunder! \
                 The damage is {} hit points.",
                objects[monster_id].name, LIGHTNING_DAMAGE
            ),
            LIGHT_BLUE,
        );
        if let Some(xp) = objects[monster_id].take_damage(LIGHTNING_DAMAGE, game) {
            objects[PLAYER].fighter.as_mut().unwrap().xp += xp;
        };
        UseResult::UsedUp
    } else {
        game.messages
            .add("No enemy is close enough to strike.", RED);
        UseResult::Cancelled
    }
}

fn cast_confuse(
    _inventory_id: usize,
    tcod: &mut Tcod,
    game: &mut Game,
    objects: &mut [Object],
) -> UseResult {
    let monster_id = target_monster(tcod, game, objects, Some(CONFUSE_RANGE as f32));
    if let Some(monster_id) = monster_id {
        let old_ai = objects[monster_id].ai.take().unwrap_or(Ai::Basic);
        objects[monster_id].ai = Some(Ai::Confused {
            previous_ai: Box::new(old_ai),
            num_turns: CONFUSE_NUM_TURNS,
        });
        game.messages.add(
            format!(
                "The eyes of {} look vacant, as he starts to stumble around!",
                objects[monster_id].name
            ),
            LIGHT_GREEN,
        );
        UseResult::UsedUp
    } else {
        game.messages.add("No enemy is close enough to strike", RED);
        UseResult::Cancelled
    }
}

fn cast_fireball(
    _inventory_id: usize,
    tcod: &mut Tcod,
    game: &mut Game,
    objects: &mut [Object],
) -> UseResult {
    game.messages.add(
        "Left-click a target tile for the fireball, or right-click to cancel.",
        LIGHT_CYAN,
    );

    let (x, y) = match target_tile(tcod, game, objects, None) {
        Some(tile_pos) => tile_pos,
        None => return UseResult::Cancelled,
    };

    game.messages.add(
        format!(
            "The fireball explodes, burning everything within {} tiles!",
            FIREBALL_RADIUS
        ),
        ORANGE,
    );

    let mut xp_to_gain = 0;
    for (id, obj) in objects.iter_mut().enumerate() {
        if obj.distance(x, y) <= FIREBALL_RADIUS as f32 && obj.fighter.is_some() {
            game.messages.add(
                format!(
                    "The {} gets burned for {} hit points.",
                    obj.name, FIREBALL_DAMAGE
                ),
                ORANGE,
            );
            if let Some(xp) = obj.take_damage(FIREBALL_DAMAGE, game) {
                if id != PLAYER {
                    xp_to_gain += xp;
                }
            }
        }
    }

    objects[PLAYER].fighter.as_mut().unwrap().xp += xp_to_gain;

    UseResult::UsedUp
}

fn cast_blink(
    _inventory_id: usize,
    tcod: &mut Tcod,
    game: &mut Game,
    objects: &mut [Object],
) -> UseResult {
    game.messages.add(
        "Left-click a target tile to blink to that location.",
        LIGHT_CYAN,
    );

    let (x, y) = match target_tile(tcod, game, objects, Some(BLINK_RADIUS as f32)) {
        Some(tile_pos) => tile_pos,
        None => {
            game.messages.add("Blink cancelled.", WHITE);
            return UseResult::Cancelled;
        }
    };

    if is_blocked(x, y, &game.map, objects) {
        game.messages.add("Cannot blink to a blocked tile.", RED);
        return UseResult::Cancelled;
    }

    game.messages
        .add("You teleport to the new location!", LIGHT_GREEN);

    objects[PLAYER].set_pos(x, y);

    UseResult::UsedUp
}

fn cast_freeze(
    _inventory_id: usize,
    tcod: &mut Tcod,
    game: &mut Game,
    objects: &mut [Object],
) -> UseResult {
    let monster_id = target_monster(tcod, game, objects, Some(FREEZE_RANGE as f32));
    if let Some(monster_id) = monster_id {
        let old_ai = objects[monster_id].ai.take().unwrap_or(Ai::Basic);
        objects[monster_id].ai = Some(Ai::Frozen {
            previous_ai: Box::new(old_ai),
            num_turns: FREEZE_NUM_TURNS,
        });
        game.messages.add(
            format!(
                "The {} is frozen solid and cannot move!",
                objects[monster_id].name
            ),
            LIGHT_BLUE,
        );
        UseResult::UsedUp
    } else {
        game.messages
            .add("No enemy is close enough to freeze.", RED);
        UseResult::Cancelled
    }
}

fn closest_monster(tcod: &Tcod, objects: &[Object], max_range: i32) -> Option<usize> {
    let mut closest_enemy = None;
    let mut closest_dist = (max_range + 1) as f32;

    for (id, object) in objects.iter().enumerate() {
        if (id != PLAYER)
            && object.fighter.is_some()
            && object.ai.is_some()
            && tcod.fov.is_in_fov(object.x, object.y)
        {
            let dist = objects[PLAYER].distance_to(object);
            if dist < closest_dist {
                closest_enemy = Some(id);
                closest_dist = dist;
            }
        }
    }
    closest_enemy
}

fn target_tile(
    tcod: &mut Tcod,
    game: &mut Game,
    objects: &[Object],
    max_range: Option<f32>,
) -> Option<(i32, i32)> {
    use tcod::input::KeyCode::Escape;
    loop {
        tcod.root.flush();
        let event = input::check_for_event(input::KEY_PRESS | input::MOUSE).map(|e| e.1);
        match event {
            Some(Event::Mouse(m)) => tcod.mouse = m,
            Some(Event::Key(k)) => tcod.key = k,
            None => tcod.key = Default::default(),
        }
        render_all(tcod, game, objects, false);

        let (x, y) = (tcod.mouse.cx as i32, tcod.mouse.cy as i32);

        let in_fov = (x < MAP_WIDTH) && (y < MAP_HEIGHT) && tcod.fov.is_in_fov(x, y);
        let in_range = max_range.map_or(true, |range| objects[PLAYER].distance(x, y) <= range);
        if tcod.mouse.lbutton_pressed && in_fov && in_range {
            return Some((x, y));
        }

        if tcod.mouse.rbutton_pressed || tcod.key.code == Escape {
            return None;
        }
    }
}

fn target_monster(
    tcod: &mut Tcod,
    game: &mut Game,
    objects: &[Object],
    max_range: Option<f32>,
) -> Option<usize> {
    loop {
        match target_tile(tcod, game, objects, max_range) {
            Some((x, y)) => {
                for (id, obj) in objects.iter().enumerate() {
                    if obj.pos() == (x, y) && obj.fighter.is_some() && id != PLAYER {
                        return Some(id);
                    }
                }
            }
            None => return None,
        }
    }
}
