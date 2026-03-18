use bevy_ecs::prelude::*;
use bevy_spliff::prelude::*;
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

#[derive(Component)]
struct Character;

#[derive(Component)]
struct Legendary;

#[derive(Component, Joinable, Default)]
#[relationship_target(relationship = InventoryItemOf)]
struct InventoryItems(Vec<Entity>);

#[derive(Component, Joinable)]
#[relationship(relationship_target = InventoryItems)]
struct InventoryItemOf(pub Entity);

const ROOT_COUNT: usize = 10_000;
const ITEMS_PER_ROOT: usize = 100;

const ROOT_RANGE: core::ops::Range<usize> = 0..ROOT_COUNT;
const ITEMS_PER_ROOT_RANGE: core::ops::Range<usize> = 0..ITEMS_PER_ROOT;

fn setup(legendary_index: usize) -> World {
    let mut world = World::new();

    for _ in ROOT_RANGE {
        let items = ITEMS_PER_ROOT_RANGE
            .map(|i| {
                let mut e = world.spawn_empty();
                if i == legendary_index {
                    e.insert(Legendary);
                }
                e.id()
            })
            .collect::<Vec<_>>();

        world.spawn((Character, InventoryItems(items)));
    }

    world
}

fn query_joins(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("joins");

    group.bench_function("join_full_fetch", |bencher| {
        let mut world = setup(50);
        let mut query_state = world.query::<J<InventoryItems, Entity>>();

        bencher.iter(|| {
            for items in query_state.iter(&world) {
                black_box(items);
            }
        });
    });

    group.bench_function("join_first_early_exit", |bencher| {
        let mut world = setup(0);
        let mut query_state = world.query::<JF<InventoryItems, (Entity, &Legendary)>>();

        bencher.iter(|| {
            for item in query_state.iter(&world) {
                black_box(item);
            }
        });
    });

    group.bench_function("join_first_late_exit", |bencher| {
        let mut world = setup(ITEMS_PER_ROOT - 1);
        let mut query_state = world.query::<JF<InventoryItems, (Entity, &Legendary)>>();

        bencher.iter(|| {
            for item in query_state.iter(&world) {
                black_box(item);
            }
        });
    });

    group.bench_function("join_early_exit_filter", |bencher| {
        let mut world = setup(0);
        let mut query_state = world.query_filtered::<Entity, JC<InventoryItems, With<Legendary>>>();

        bencher.iter(|| {
            for entity in query_state.iter(&world) {
                black_box(entity);
            }
        });
    });

    group.finish();
}

criterion_group!(benches, query_joins);
criterion_main!(benches);
