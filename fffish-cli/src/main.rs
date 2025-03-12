use std::time::SystemTime;

use chrono::Local;
use ffxivfishing::{carbuncledata::fishes, eorzea_time::EorzeaTime};

fn main() {
    let fishes = fishes().expect("Parsing the fish data failed");
    for f in fishes {
        if let Some(next_window) =
            f.next_window(EorzeaTime::from_time(&SystemTime::now()).expect("F"), 1_000)
        {
            let real_time: chrono::DateTime<Local> = next_window.start().to_system_time().into();
            println!("{}: {}", f.name(), real_time);
        }
    }
}
