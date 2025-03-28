use std::{collections::HashMap, error::Error, rc::Rc, time::Duration};

use serde::{Deserialize, Serialize};

use crate::{
    eorzea_time::EorzeaDuration,
    fish::{Bait, Fish, FishData, FishingHole, FishingItem, Intuition, Lure, Region},
    weather::{Weather, WeatherForecast},
};

const DATA: &str = include_str!("data.json");

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
enum OneOrVec<T> {
    One(T),
    Vec(Vec<T>),
}

#[derive(Debug, Serialize, Deserialize)]
struct CarbuncleData {
    #[serde(rename = "FISH")]
    fishes: HashMap<String, CarbuncleFish>,
    #[serde(rename = "WEATHER_RATES")]
    weather_rates: HashMap<String, CarbuncleWeatherRates>,
    #[serde(rename = "FISHING_SPOTS")]
    fishing_spots: HashMap<String, CarbuncleFishingSpot>,
    #[serde(rename = "ITEMS")]
    items: HashMap<String, CarbuncleItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct CarbuncleFish {
    #[serde(rename = "_id")]
    id: u32,
    #[serde(rename = "previousWeatherSet")]
    previous_weather_set: Vec<u32>,
    #[serde(rename = "weatherSet")]
    weather_set: Vec<u32>,
    #[serde(rename = "bestCatchPath")]
    best_catch_path: Vec<OneOrVec<u32>>,
    #[serde(rename = "startHour")]
    start_hour: f32,
    #[serde(rename = "endHour")]
    end_hour: f32,
    #[serde(rename = "location")]
    location: Option<u32>,
    #[serde(rename = "intuitionLength")]
    intuition_length: Option<u32>,
    #[serde(rename = "predators")]
    predators: Vec<[u32; 2]>,
    #[serde(rename = "tug")]
    tug: Option<String>,
    #[serde(rename = "hookset")]
    hookset: Option<String>,
    #[serde(rename = "lure")]
    lure: Option<String>,
    #[serde(rename = "fishEyes")]
    fish_eyes: bool,
    #[serde(rename = "bigFish")]
    bg_fish: bool,
    #[serde(rename = "snagging")]
    snagging: Option<bool>,
    #[serde(rename = "patch")]
    patch: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct CarbuncleFishingSpot {
    #[serde(rename = "_id")]
    id: u32,
    #[serde(rename = "name_en")]
    name: String,
    #[serde(rename = "map_coords")]
    map_coords: [f32; 3],
    #[serde(rename = "territory_id")]
    territory_id: u32,
    #[serde(rename = "placename_id")]
    placename_id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct CarbuncleItem {
    #[serde(rename = "_id")]
    id: u32,
    #[serde(rename = "name_en")]
    name: String,
    #[serde(rename = "icon")]
    icon: String,
    #[serde(rename = "ilvl")]
    ilvl: u32,
}
impl CarbuncleItem {
    fn to_fishing_item(&self, fishes: &[Fish]) -> FishingItem {
        match fishes.iter().find(|f| f.id == self.id) {
            Some(f) => FishingItem::Fish(self.name.clone(), f.id),
            None => FishingItem::Bait(self.name.clone(), self.id),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct CarbuncleWeatherRates {
    #[serde(rename = "map_id")]
    map_id: u32,
    #[serde(rename = "map_scale")]
    map_scale: u32,
    #[serde(rename = "zone_id")]
    zone_id: u32,
    #[serde(rename = "region_id")]
    region_id: u32,
    #[serde(rename = "weather_rates")]
    weather_rates: Vec<(u32, u8)>,
}

impl From<&CarbuncleWeatherRates> for WeatherForecast {
    fn from(cwr: &CarbuncleWeatherRates) -> Self {
        WeatherForecast::new(
            cwr.map_id.to_string(),
            cwr.weather_rates
                .iter()
                .map(|(weather_id, rate)| (*rate, Weather::Id(*weather_id)))
                .collect(),
        )
    }
}

impl CarbuncleFishingSpot {
    fn to_fishinghole(&self, regions: &[Rc<Region>]) -> Option<FishingHole> {
        let region = regions
            .iter()
            .find(|r| r.name() == self.territory_id.to_string())?;
        Some(FishingHole::new(self.id.to_string(), region.clone()))
    }
}

impl CarbuncleFish {
    fn try_get_intuition(&self) -> Option<Intuition> {
        self.intuition_length.map(|l| {
            Intuition::new(
                Duration::from_secs(l as u64),
                self.predators.iter().map(|p| (p[1] as u8, p[0])).collect(),
            )
        })
    }

    fn to_fish(&self, fishing_holes: &[Rc<FishingHole>], items: &[&CarbuncleItem]) -> Option<Fish> {
        let fish_hole = fishing_holes
            .iter()
            .find(|fh| fh.name() == self.location.unwrap_or(0).to_string())?;
        let item = items.iter().find(|i| self.id == i.id)?;

        let bait = match self.best_catch_path.last() {
            Some(OneOrVec::One(o)) => Bait::Bait(*o),
            Some(OneOrVec::Vec(o)) if o.is_empty() => Bait::Unknown,
            Some(OneOrVec::Vec(o)) => Bait::Bait(*o.last().unwrap()),
            None => Bait::Unknown,
        };
        Some(Fish::new(
            self.id,
            item.name.clone(),
            Rc::clone(fish_hole),
            EorzeaDuration::from_esecs((self.start_hour * 3600.0) as u64),
            EorzeaDuration::from_esecs((self.end_hour * 3600.0) as u64),
            bait,
            self.previous_weather_set
                .iter()
                .map(|id| Weather::Id(*id))
                .collect(),
            self.weather_set.iter().map(|id| Weather::Id(*id)).collect(),
            self.tug.clone().unwrap_or("".to_string()).as_str().into(),
            self.hookset
                .clone()
                .unwrap_or("".to_string())
                .as_str()
                .into(),
            self.try_get_intuition(),
            Lure::Moderate,
            self.lure.is_some(),
            self.snagging.unwrap_or(false),
            false,
            false,
            self.fish_eyes,
            (self.patch.trunc() as u8, self.patch.fract() as u8),
        ))
    }
}

fn parse_fishes() -> Result<Vec<CarbuncleFish>, serde_json::Error> {
    let data: serde_json::Value = serde_json::from_str(DATA)?;

    let fishes = match data["FISH"].as_object() {
        Some(f) => f.clone(),
        None => return Ok(vec![]),
    };

    Ok(fishes
        .values()
        .filter_map(|f| serde_json::from_value::<CarbuncleFish>(f.clone()).ok())
        .collect())
}

fn parse_fishing_spots() -> Result<Vec<CarbuncleFishingSpot>, serde_json::Error> {
    let data: serde_json::Value = serde_json::from_str(DATA)?;

    let fish_spots = match data["FISHING_SPOTS"].as_object() {
        Some(f) => f.clone(),
        None => return Ok(vec![]),
    };

    Ok(fish_spots
        .values()
        .filter_map(|f| serde_json::from_value::<CarbuncleFishingSpot>(f.clone()).ok())
        .collect())
}

fn parse_weather() -> Result<Vec<CarbuncleWeatherRates>, serde_json::Error> {
    let data: serde_json::Value = serde_json::from_str(DATA)?;

    let fishes = match data["WEATHER_RATES"].as_object() {
        Some(f) => f.clone(),
        None => return Ok(vec![]),
    };

    Ok(fishes
        .values()
        .filter_map(|f| serde_json::from_value::<CarbuncleWeatherRates>(f.clone()).ok())
        .collect())
}

fn parse_data() -> Result<CarbuncleData, serde_json::Error> {
    serde_json::from_str(DATA)
}

impl CarbuncleData {
    fn convert_to_fishdata(&self) -> FishData {
        let weather_rates: HashMap<String, WeatherForecast> = self
            .weather_rates
            .clone()
            .into_iter()
            .map(|(id, w)| (id, (&w).into()))
            .collect();

        let items: Vec<&CarbuncleItem> = self.items.values().collect();

        let regions: Vec<Rc<Region>> = weather_rates
            .iter()
            .map(|(id, w)| Rc::new(Region::new(id.to_string(), w.clone())))
            .collect();

        let fishing_holes: Vec<Rc<FishingHole>> = self
            .fishing_spots
            .values()
            .filter_map(|fs| fs.to_fishinghole(&regions))
            .map(Rc::new)
            .collect();

        let fishes: Vec<Fish> = self
            .fishes
            .values()
            .filter_map(|f| f.to_fish(&fishing_holes, &items))
            .collect();
        let fishing_items = items
            .iter()
            .map(|item| item.to_fishing_item(&fishes))
            .collect();
        FishData::new(fishes, fishing_holes, regions, fishing_items)
    }
}

pub fn carbuncle_fishes() -> Result<FishData, Box<dyn Error>> {
    let data = parse_data()?;
    Ok(data.convert_to_fishdata())
}

#[cfg(test)]
mod tests {

    use std::time::SystemTime;

    use crate::eorzea_time::EorzeaTime;

    use super::*;
    #[test]
    fn parse_fishing_spots_test() {
        let fish_spots = parse_fishing_spots().unwrap();
        assert!(!fish_spots.is_empty());
        for s in fish_spots {
            println!("{}", s.territory_id);
        }
    }

    #[test]
    fn weather_at() {
        let weathers = parse_weather().unwrap();
        assert!(!weathers.is_empty());
        for w in weathers {
            let eorzea_weather: WeatherForecast = (&w).into();
            let _ = eorzea_weather.weather_at(EorzeaTime::from_time(&SystemTime::now()).unwrap());
        }
    }

    #[test]
    fn parse_data_test() {
        let data = parse_data().unwrap();
        let fishes = data.convert_to_fishdata();
        for fish in fishes.fishes() {
            let window = fish.next_window(
                EorzeaTime::from_time(&SystemTime::now()).unwrap(),
                false,
                1_000,
            );
            if window.is_some() {
                let w = window.unwrap();
                println!(
                    "{:?}: {} - {:?}",
                    fish.name(),
                    w,
                    w.start().to_system_time()
                );
            } else {
                println!("{:?}: !!!", fish.name());
            }
        }
    }
}
