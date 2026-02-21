use bingo::{
    serializers::{borsh::Borsh, json::Json, wincode::Wincode},
    storage::Storage,
};
use borsh::{BorshDeserialize, BorshSerialize};
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
struct Game {
    name: String,
    release_year: u16,
}

#[test]
fn test_borsh() {
    let game = Game {
        name: "Witcher 3".to_string(),
        release_year: 2015,
    };

    let mut storage = Storage::new(Borsh);

    storage.save(&game).unwrap();

    let load = storage.load().unwrap();

    assert_eq!(load, game);
}

#[test]
fn test_wincode() {
    let game = Game {
        name: "Witcher 3".to_string(),
        release_year: 2015,
    };

    let mut storage = Storage::new(Wincode);

    storage.save(&game).unwrap();

    let load = storage.load().unwrap();

    assert_eq!(load, game);
}

#[test]
fn test_json() {
    let game = Game {
        name: "Witcher 3".to_string(),
        release_year: 2015,
    };

    let mut storage = Storage::new(Json);

    storage.save(&game).unwrap();

    let load = storage.load().unwrap();

    assert_eq!(load, game);
}

#[test]
fn test_convert_from_json_to_wincode() {
    let game = Game {
        name: "Witcher 3".to_string(),
        release_year: 2015,
    };

    let mut storage = Storage::new(Json);

    storage.save(&game).unwrap();

    let storage2 = storage.convert(Wincode).unwrap();

    let load = storage2.load().unwrap();
    assert_eq!(load, game);
}
