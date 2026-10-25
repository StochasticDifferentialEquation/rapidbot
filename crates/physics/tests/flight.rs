//! Creative flight, ported from `LocalPlayer.aiStep` and `Player.travel`.

use rapidbot_physics::math::Vec3;
use rapidbot_physics::{Keys, Player};
use rapidbot_world::chunk::Chunk;
use rapidbot_world::tags::Tags;
use rapidbot_world::{DimensionHeight, Registry, World};

/// Stone floor with its top at y = 64.
fn flat_world() -> World {
    let r = Registry::get();
    let block = r.block_by_name("minecraft:stone").unwrap();
    let stone = r.states.iter().position(|s| s.block == block.id).unwrap() as u32;
    let mut world = World::new(DimensionHeight::OVERWORLD);
    for cx in -2..=2 {
        for cz in -2..=2 {
            world.insert_chunk(cx, cz, Chunk::empty(24));
            for x in 0..16 {
                for z in 0..16 {
                    world.set_block_state(cx * 16 + x, 63, cz * 16 + z, stone);
                }
            }
        }
    }
    world
}

fn flyer_at(y: f64) -> Player {
    let mut p = Player::new();
    p.y_rot = 0.0;
    p.set_pos(Vec3::new(0.5, y, 0.5));
    p.abilities.may_fly = true;
    p
}

fn jump(down: bool) -> Keys {
    Keys {
        jump: down,
        ..Keys::default()
    }
}

/// One press-release-press of jump, `gap` ticks between the two presses.
fn double_tap(p: &mut Player, world: &World, tags: &Tags, gap: usize) {
    p.held_keys = jump(true);
    p.tick(world, tags);
    p.held_keys = jump(false);
    for _ in 0..gap {
        p.tick(world, tags);
    }
    p.held_keys = jump(true);
    p.tick(world, tags);
}

#[test]
fn double_tap_jump_toggles_flight() {
    let (world, tags) = (flat_world(), Tags::default());
    let mut p = flyer_at(70.0);
    double_tap(&mut p, &world, &tags, 3);
    assert!(p.abilities.flying);
    assert!(p.abilities_changed);

    // Same again turns it off.
    p.abilities_changed = false;
    p.held_keys = jump(false);
    p.tick(&world, &tags);
    double_tap(&mut p, &world, &tags, 3);
    assert!(!p.abilities.flying);
    assert!(p.abilities_changed);
}

#[test]
fn a_single_tap_or_a_slow_second_tap_does_not_toggle() {
    let (world, tags) = (flat_world(), Tags::default());
    let mut p = flyer_at(70.0);
    p.held_keys = jump(true);
    p.tick(&world, &tags);
    p.held_keys = jump(false);
    for _ in 0..20 {
        p.tick(&world, &tags);
    }
    assert!(!p.abilities.flying);

    // The window is 7 ticks: 10 ticks apart is too slow.
    double_tap(&mut p, &world, &tags, 10);
    assert!(!p.abilities.flying);
    assert!(!p.abilities_changed);
}

#[test]
fn without_may_fly_nothing_toggles() {
    let (world, tags) = (flat_world(), Tags::default());
    let mut p = flyer_at(70.0);
    p.abilities.may_fly = false;
    double_tap(&mut p, &world, &tags, 3);
    assert!(!p.abilities.flying);
    assert!(!p.abilities_changed);
}

#[test]
fn flying_hovers_with_no_input() {
    let (world, tags) = (flat_world(), Tags::default());
    let mut p = flyer_at(70.0);
    p.abilities.flying = true;
    for _ in 0..40 {
        p.tick(&world, &tags);
    }
    // Player.travel keeps the pre-move vertical speed, so gravity never
    // builds up.
    assert_eq!(p.pos.y, 70.0);
    assert_eq!(p.delta_movement.y, 0.0);
}

#[test]
fn jump_climbs_and_shift_descends_while_flying() {
    let (world, tags) = (flat_world(), Tags::default());
    let mut p = flyer_at(70.0);
    p.abilities.flying = true;

    p.held_keys = jump(true);
    p.tick(&world, &tags);
    // flying speed 0.05 * 3 added before the move, then damped by 0.6.
    let added = (1.0f32 * 0.05f32 * 3.0f32) as f64;
    assert_eq!(p.delta_movement.y, added * 0.6);
    assert!(p.pos.y > 70.0);

    let mut q = flyer_at(70.0);
    q.abilities.flying = true;
    q.held_keys = Keys {
        shift: true,
        ..Keys::default()
    };
    q.tick(&world, &tags);
    assert!(q.pos.y < 70.0);
}

#[test]
fn landing_ends_flight() {
    let (world, tags) = (flat_world(), Tags::default());
    let mut p = flyer_at(66.0);
    p.abilities.flying = true;
    p.held_keys = Keys {
        shift: true,
        ..Keys::default()
    };
    for _ in 0..200 {
        p.tick(&world, &tags);
        if !p.abilities.flying {
            break;
        }
    }
    assert!(!p.abilities.flying);
    assert!(p.abilities_changed);
    assert!(p.on_ground);
    assert_eq!(p.pos.y, 64.0);
}

#[test]
fn a_spectator_is_always_flying_and_never_lands() {
    let (world, tags) = (flat_world(), Tags::default());
    let mut p = flyer_at(70.0);
    p.spectator = true;
    p.tick(&world, &tags);
    assert!(p.abilities.flying);
    assert!(p.abilities_changed);

    p.abilities_changed = false;
    p.held_keys = Keys {
        shift: true,
        ..Keys::default()
    };
    for _ in 0..60 {
        p.tick(&world, &tags);
    }
    assert!(p.abilities.flying);
    assert!(!p.abilities_changed);
}
