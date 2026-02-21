use bingo::serializers::json::Json;
use borsh::{BorshDeserialize, BorshSerialize};
use wincode::{SchemaRead, SchemaWrite};

use bingo::serializers::borsh::Borsh;
use bingo::serializers::wincode::Wincode;
use bingo::storage::Storage;

#[derive(SchemaRead, SchemaWrite, Debug, BorshSerialize, BorshDeserialize)]
struct Game {
    name: String,
    release_year: u32,
}

fn main() {
    let game = Game {
        name: "Withcher 3".to_string(),
        release_year: 2015,
    };

    let mut storage = Storage::new(Wincode);

    storage.save(&game).unwrap();

    let loaded = storage.load().unwrap();

    println!("{:?}", loaded);

    let storage2 = storage.convert(Borsh).unwrap();

    let loaded = storage2.load().unwrap();

    println!("{:?}", loaded);
}
