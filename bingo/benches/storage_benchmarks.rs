use bingo::{
    serializers::{borsh::Borsh, json::Json, wincode::Wincode},
    storage::Storage,
};
use borsh::{BorshDeserialize, BorshSerialize};
use criterion::{Criterion, criterion_group, criterion_main};
use serde::{Deserialize, Serialize};
use wincode::{SchemaRead, SchemaWrite};

#[derive(
    Debug,
    PartialEq,
    BorshSerialize,
    BorshDeserialize,
    SchemaWrite,
    SchemaRead,
    Serialize,
    Deserialize,
)]
pub struct GameFlat {
    pub name: String,
    pub release_year: u16,

    pub slug: String,
    pub franchise: String,

    pub developer: String,
    pub publisher: String,
    pub director: String,
    pub producer: String,
    pub writer: String,
    pub composer: String,

    pub genre_primary: String,
    pub genre_secondary: String,
    pub engine: String,
    pub age_rating: String,

    pub pc: bool,
    pub ps4: bool,
    pub ps5: bool,
    pub xbox_one: bool,
    pub xbox_series: bool,
    pub nintendo_switch: bool,

    pub main_story_hours: u16,
    pub completionist_hours: u16,
    pub side_quest_count: u16,
    pub dlc_count: u16,

    pub supports_controller: bool,
    pub supports_mods: bool,
    pub supports_cross_save: bool,
    pub supports_cloud_save: bool,
    pub max_players: u8,
    pub online_features: bool,

    pub cover_image_url: String,
    pub trailer_url: String,
    pub store_page_url: String,

    pub metacritic_score: u8,
    pub user_score: f32,
    pub review_count: u32,

    pub price_usd: f32,
    pub discount_percent: u8,
    pub is_free_to_play: bool,

    pub release_date_iso: String,
    pub last_patch_date_iso: String,
    pub is_supported: bool,

    pub description: String,
    pub short_description: String,
    pub tags: String,
}

fn benchmark_borsh(c: &mut Criterion) {
    let game = GameFlat {
        name: "Witcher 3".to_string(),
        release_year: 2015,

        slug: "the-witcher-3-wild-hunt".to_string(),
        franchise: "The Witcher".to_string(),

        developer: "CD Projekt Red".to_string(),
        publisher: "CD Projekt".to_string(),
        director: "Konrad Tomaszkiewicz".to_string(),
        producer: "Jędrzej Mróz".to_string(),
        writer: "Marcin Blacha".to_string(),
        composer: "Marcin Przybyłowicz".to_string(),

        genre_primary: "RPG".to_string(),
        genre_secondary: "Open World".to_string(),
        engine: "REDengine 3".to_string(),
        age_rating: "M".to_string(),

        pc: true,
        ps4: true,
        ps5: true,
        xbox_one: true,
        xbox_series: true,
        nintendo_switch: true,

        main_story_hours: 51,
        completionist_hours: 172,
        side_quest_count: 100,
        dlc_count: 2,

        supports_controller: true,
        supports_mods: true,
        supports_cross_save: true,
        supports_cloud_save: true,
        max_players: 1,
        online_features: false,

        cover_image_url: "https://example.com/cover.jpg".to_string(),
        trailer_url: "https://example.com/trailer".to_string(),
        store_page_url: "https://example.com/store".to_string(),

        metacritic_score: 93,
        user_score: 9.3,
        review_count: 1_000_000,

        price_usd: 39.99,
        discount_percent: 0,
        is_free_to_play: false,

        release_date_iso: "2015-05-19".to_string(),
        last_patch_date_iso: "2024-01-01".to_string(),
        is_supported: true,

        description: "Open-world RPG set in a dark fantasy universe.".to_string(),
        short_description: "Story-driven open-world RPG.".to_string(),
        tags: "rpg,open-world,story-rich,fantasy".to_string(),
    };
    c.bench_function("borsh serialize", |b| {
        b.iter(|| {
            let mut storage = Storage::new(Borsh);
            storage.save(&game).unwrap();
        })
    });
    let mut storage = Storage::new(Borsh);
    storage.save(std::hint::black_box(&game)).unwrap();
    c.bench_function("borsh deserialize", |b| {
        b.iter(|| {
            let load = std::hint::black_box(storage.load().unwrap());
        })
    });
}

fn benchmark_wincode(c: &mut Criterion) {
    let game = GameFlat {
        name: "Witcher 3".to_string(),
        release_year: 2015,

        slug: "the-witcher-3-wild-hunt".to_string(),
        franchise: "The Witcher".to_string(),

        developer: "CD Projekt Red".to_string(),
        publisher: "CD Projekt".to_string(),
        director: "Konrad Tomaszkiewicz".to_string(),
        producer: "Jędrzej Mróz".to_string(),
        writer: "Marcin Blacha".to_string(),
        composer: "Marcin Przybyłowicz".to_string(),

        genre_primary: "RPG".to_string(),
        genre_secondary: "Open World".to_string(),
        engine: "REDengine 3".to_string(),
        age_rating: "M".to_string(),

        pc: true,
        ps4: true,
        ps5: true,
        xbox_one: true,
        xbox_series: true,
        nintendo_switch: true,

        main_story_hours: 51,
        completionist_hours: 172,
        side_quest_count: 100,
        dlc_count: 2,

        supports_controller: true,
        supports_mods: true,
        supports_cross_save: true,
        supports_cloud_save: true,
        max_players: 1,
        online_features: false,

        cover_image_url: "https://example.com/cover.jpg".to_string(),
        trailer_url: "https://example.com/trailer".to_string(),
        store_page_url: "https://example.com/store".to_string(),

        metacritic_score: 93,
        user_score: 9.3,
        review_count: 1_000_000,

        price_usd: 39.99,
        discount_percent: 0,
        is_free_to_play: false,

        release_date_iso: "2015-05-19".to_string(),
        last_patch_date_iso: "2024-01-01".to_string(),
        is_supported: true,

        description: "Open-world RPG set in a dark fantasy universe.".to_string(),
        short_description: "Story-driven open-world RPG.".to_string(),
        tags: "rpg,open-world,story-rich,fantasy".to_string(),
    };
    c.bench_function("wincode serialize", |b| {
        b.iter(|| {
            let mut storage = Storage::new(Wincode);
            storage.save(std::hint::black_box(&game)).unwrap();
        })
    });
    let mut storage = Storage::new(Wincode);
    storage.save(&game).unwrap();
    c.bench_function("wincode deserialize", |b| {
        b.iter(|| {
            let load = std::hint::black_box(storage.load().unwrap());
        })
    });
}

fn benchmark_json(c: &mut Criterion) {
    let game = GameFlat {
        name: "Witcher 3".to_string(),
        release_year: 2015,

        slug: "the-witcher-3-wild-hunt".to_string(),
        franchise: "The Witcher".to_string(),

        developer: "CD Projekt Red".to_string(),
        publisher: "CD Projekt".to_string(),
        director: "Konrad Tomaszkiewicz".to_string(),
        producer: "Jędrzej Mróz".to_string(),
        writer: "Marcin Blacha".to_string(),
        composer: "Marcin Przybyłowicz".to_string(),

        genre_primary: "RPG".to_string(),
        genre_secondary: "Open World".to_string(),
        engine: "REDengine 3".to_string(),
        age_rating: "M".to_string(),

        pc: true,
        ps4: true,
        ps5: true,
        xbox_one: true,
        xbox_series: true,
        nintendo_switch: true,

        main_story_hours: 51,
        completionist_hours: 172,
        side_quest_count: 100,
        dlc_count: 2,

        supports_controller: true,
        supports_mods: true,
        supports_cross_save: true,
        supports_cloud_save: true,
        max_players: 1,
        online_features: false,

        cover_image_url: "https://example.com/cover.jpg".to_string(),
        trailer_url: "https://example.com/trailer".to_string(),
        store_page_url: "https://example.com/store".to_string(),

        metacritic_score: 93,
        user_score: 9.3,
        review_count: 1_000_000,

        price_usd: 39.99,
        discount_percent: 0,
        is_free_to_play: false,

        release_date_iso: "2015-05-19".to_string(),
        last_patch_date_iso: "2024-01-01".to_string(),
        is_supported: true,

        description: "Open-world RPG set in a dark fantasy universe.".to_string(),
        short_description: "Story-driven open-world RPG.".to_string(),
        tags: "rpg,open-world,story-rich,fantasy".to_string(),
    };
    c.bench_function("json serialize", |b| {
        b.iter(|| {
            let mut storage = Storage::new(Json);
            storage.save(std::hint::black_box(&game)).unwrap();
        })
    });
    let mut storage = Storage::new(Json);
    storage.save(&game).unwrap();
    c.bench_function("json deserialize", |b| {
        b.iter(|| {
            let load = std::hint::black_box(storage.load().unwrap());
        })
    });
}
criterion_group!(benches, benchmark_borsh, benchmark_wincode, benchmark_json);
criterion_main!(benches);
