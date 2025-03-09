use std::{error::Error, time::SystemTime};

use crate::{
    eorzea_time::{
        self, EORZEA_WEATHER_PERIOD, EORZEA_ZERO_TIME, EorzeaDuration, EorzeaTime, EorzeaTimeSpan,
        SUN_IN_ESEC,
    },
    weather::{Weather, WeatherForecast},
};

pub struct Region<'a> {
    name: String,
    weather: &'a WeatherForecast,
}

pub struct FishingHole<'a> {
    name: String,
    region: &'a Region<'a>,
}

pub enum Tug {
    Light,
    Medium,
    Heavy,
}

pub enum Hookset {
    Precision,
    Powerful,
}

pub enum Bait<'a> {
    Fish(&'a Fish<'a>),
    Bait(String),
}

pub struct Fish<'a> {
    name: String,
    location: &'a FishingHole<'a>,
    window_start: EorzeaDuration,
    window_end: EorzeaDuration,
    bait: Bait<'a>,
    previous_weather_set: Vec<Weather>,
    weather_set: Vec<Weather>,
    tug: Tug,
    hookset: Hookset,
    snagging: bool,
    gig: bool,
    folklore: bool,
    fish_eyes: bool,
    patch: (u8, u8),
}

impl<'a> Fish<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new<'b>(
        name: String,
        location: &'b FishingHole<'b>,
        window_start: EorzeaDuration,
        window_end: EorzeaDuration,
        bait: Bait<'a>,
        previous_weather_set: Vec<Weather>,
        weather_set: Vec<Weather>,
        tug: Tug,
        hookset: Hookset,
        snagging: bool,
        gig: bool,
        folklore: bool,
        fish_eyes: bool,
        patch: (u8, u8),
    ) -> Option<Fish<'a>>
    where
        'b: 'a,
    {
        if window_start >= EorzeaDuration::from_esecs(SUN_IN_ESEC) {
            return None;
        }
        Some(Self {
            name,
            location,
            window_start,
            window_end,
            bait,
            previous_weather_set,
            weather_set,
            tug,
            hookset,
            snagging,
            gig,
            folklore,
            fish_eyes,
            patch,
        })
    }

    pub fn window_on_day(&self, etime: EorzeaTime) -> EorzeaTimeSpan {
        let mut day = etime;
        day.round(EorzeaDuration::new_ext(0, 0, 1, 0, 0, 0).unwrap());
        let start = day + self.window_start;
        let mut end = day + self.window_end;
        if end < start {
            end += EorzeaDuration::from_esecs(SUN_IN_ESEC);
        }
        EorzeaTimeSpan::new_start_end(start, end).unwrap()
    }

    pub fn next_window(&self, time: EorzeaTime, limit: u32) -> Option<EorzeaTimeSpan> {
        let next_weather = self.location.region.weather.find_pattern(
            time,
            &self.previous_weather_set,
            &self.weather_set,
            limit,
        )?;
        let weather_span = EorzeaTimeSpan::new(next_weather, EORZEA_WEATHER_PERIOD);
        match self.window_on_day(time).overlap(&weather_span) {
            Ok(o) => Some(o),
            Err(_) => self.next_window(time + EORZEA_WEATHER_PERIOD, limit - 1),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    pub fn next_window() {
        let weather = WeatherForecast::new(
            "Region".to_string(),
            vec![(50, Weather::Clouds), (100, Weather::Sunny)],
        );
        let fishing_hole = FishingHole {
            name: "Fishing Hole".to_string(),
            region: &Region {
                name: "Region".to_string(),
                weather: &weather,
            },
        };
        let fish = Fish {
            name: "".to_string(),
            location: &fishing_hole,
            window_start: EorzeaDuration::new(1, 0, 0).unwrap(),
            window_end: EorzeaDuration::new(2, 0, 0).unwrap(),
            bait: Bait::Bait("Bait".to_string()),
            previous_weather_set: vec![Weather::Clouds],
            weather_set: vec![Weather::Clouds],
            tug: Tug::Light,
            hookset: Hookset::Precision,
            snagging: false,
            gig: false,
            folklore: false,
            fish_eyes: false,
            patch: (7, 0),
        };
        let result = fish
            .next_window(EorzeaTime::new(1, 1, 2, 2, 0, 0).unwrap(), 1000)
            .unwrap();
        assert_eq!(result.start(), EorzeaTime::new(1, 1, 3, 1, 0, 0).unwrap());
        assert_eq!(result.end(), EorzeaTime::new(1, 1, 3, 2, 0, 0).unwrap());
    }

    #[test]
    pub fn next_window_weather_border() {
        let weather = WeatherForecast::new(
            "Region".to_string(),
            vec![(50, Weather::Clouds), (100, Weather::Sunny)],
        );
        let fishing_hole = FishingHole {
            name: "Fishing Hole".to_string(),
            region: &Region {
                name: "Region".to_string(),
                weather: &weather,
            },
        };
        let fish = Fish {
            name: "".to_string(),
            location: &fishing_hole,
            window_start: EorzeaDuration::new(7, 30, 0).unwrap(),
            window_end: EorzeaDuration::new(8, 30, 0).unwrap(),
            bait: Bait::Bait("Bait".to_string()),
            previous_weather_set: vec![Weather::Clouds],
            weather_set: vec![Weather::Clouds],
            tug: Tug::Light,
            hookset: Hookset::Precision,
            snagging: false,
            gig: false,
            folklore: false,
            fish_eyes: false,
            patch: (7, 0),
        };
        let result = fish
            .next_window(EorzeaTime::new(1, 1, 2, 0, 0, 0).unwrap(), 1000)
            .unwrap();
        assert_eq!(result.start(), EorzeaTime::new(1, 1, 3, 7, 30, 0).unwrap());
        assert_eq!(result.end(), EorzeaTime::new(1, 1, 3, 8, 0, 0).unwrap());
    }
}
