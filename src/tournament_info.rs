use std::str::FromStr;

use chrono::DateTime;
use chrono::Local;
use serde_json::Map;
use serde_json::Value;

#[derive(Clone, Copy)]
pub enum Belt {
    Kyu9,
    Kyu8,
    Kyu7,
    Kyu6,
    Kyu5,
    Kyu4,
    Kyu3,
    Kyu2,
    Kyu1,
    Dan1,
    Dan2,
    Dan3,
    Dan4,
    Dan5,
    Dan6,
    Dan7,
    Dan8,
    Dan9,
    Dan10
}

impl Belt {
    pub fn to_number(self) -> u8 {
        match self {
            Self::Kyu9 => 1,
            Self::Kyu8 => 2,
            Self::Kyu7 => 3,
            Self::Kyu6 => 4,
            Self::Kyu5 => 5,
            Self::Kyu4 => 6,
            Self::Kyu3 => 7,
            Self::Kyu2 => 8,
            Self::Kyu1 => 9,
            Self::Dan1 => 10,
            Self::Dan2 => 11,
            Self::Dan3 => 12,
            Self::Dan4 => 13,
            Self::Dan5 => 14,
            Self::Dan6 => 15,
            Self::Dan7 => 16,
            Self::Dan8 => 17,
            Self::Dan9 => 18,
            Self::Dan10 => 19
        }
    }

    pub fn render(self) -> String {
        format!("{}", self.to_number())
    }

    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "kyu9" => Self::Kyu9,
            "kyu8" => Self::Kyu8,
            "kyu7" => Self::Kyu7,
            "kyu6" => Self::Kyu6,
            "kyu5" => Self::Kyu5,
            "kyu4" => Self::Kyu4,
            "kyu3" => Self::Kyu3,
            "kyu2" => Self::Kyu2,
            "kyu1" => Self::Kyu1,
            "dan1" => Self::Dan1,
            "dan2" => Self::Dan2,
            "dan3" => Self::Dan3,
            "dan4" => Self::Dan4,
            "dan5" => Self::Dan5,
            "dan6" => Self::Dan6,
            "dan7" => Self::Dan7,
            "dan8" => Self::Dan8,
            "dan9" => Self::Dan9,
            "dan10" => Self::Dan10,
            _ => {
                return None;
            }
        })
    }

    pub fn inc(self) -> Self {
        match self {
            Self::Kyu9 => Self::Kyu8,
            Self::Kyu8 => Self::Kyu7,
            Self::Kyu7 => Self::Kyu6,
            Self::Kyu6 => Self::Kyu5,
            Self::Kyu5 => Self::Kyu4,
            Self::Kyu4 => Self::Kyu3,
            Self::Kyu3 => Self::Kyu2,
            Self::Kyu2 => Self::Kyu1,
            Self::Kyu1 => Self::Dan1,
            Self::Dan1 => Self::Dan2,
            Self::Dan2 => Self::Dan3,
            Self::Dan3 => Self::Dan4,
            Self::Dan4 => Self::Dan5,
            Self::Dan5 => Self::Dan6,
            Self::Dan6 => Self::Dan7,
            Self::Dan7 => Self::Dan8,
            Self::Dan8 => Self::Dan9,
            Self::Dan9 | Self::Dan10 => Self::Dan10
        }
    }
    
    pub fn serialise(self) -> String {
        String::from(match self {
            Self::Kyu9 => "kyu9",
            Self::Kyu8 => "kyu8",
            Self::Kyu7 => "kyu7",
            Self::Kyu6 => "kyu6",
            Self::Kyu5 => "kyu5",
            Self::Kyu4 => "kyu4",
            Self::Kyu3 => "kyu3",
            Self::Kyu2 => "kyu2",
            Self::Kyu1 => "kyu1",
            Self::Dan1 => "dan1",
            Self::Dan2 => "dan2",
            Self::Dan3 => "dan3",
            Self::Dan4 => "dan4",
            Self::Dan5 => "dan5",
            Self::Dan6 => "dan6",
            Self::Dan7 => "dan7",
            Self::Dan8 => "dan8",
            Self::Dan9 => "dan9",
            Self::Dan10 => "dan10"
        })
    }
}

impl FromStr for Belt {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Belt::from_str(s).ok_or("not a valid belt")
    }
}

#[derive(Default)]
pub enum WeightCategoryKind {
    #[default]
    Under,
    Over
}

#[derive(Default)]
pub struct WeightCategory {
    kind: WeightCategoryKind,
    limit: u8
}

impl WeightCategory {
    pub fn render(&self) -> String {
        match self.kind {
            WeightCategoryKind::Over => String::new(),
            WeightCategoryKind::Under => format!("{}", self.limit)
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        let kind = if s.starts_with('-') {
            WeightCategoryKind::Under
        } else if s.starts_with('+') {
            WeightCategoryKind::Over
        } else {
            return None;
        };
        let limit = s[1..(s.len())].parse().ok()?;
        Some(Self { kind, limit })
    }
}

pub struct Athlete {
    pub given_name: String,
    pub sur_name: String,
    pub belt: Belt,
    pub weight_category: WeightCategory,
    birth_year: u16
}

impl Athlete {
    pub fn new(given_name: String, sur_name: String, birth_year: u16, belt: Belt, weight_category: WeightCategory) -> Self {
        Self { given_name, sur_name, belt, weight_category, birth_year }
    }

    pub fn render(&self) -> String {
        format!(include_str!("athlete-format"), self.sur_name, self.given_name, self.belt.render(), self.weight_category.render(), self.birth_year)
    }

    pub fn serialise(&self) -> Value {
        let mut map = Map::new();
        map.insert(String::from("given"), Value::String(self.given_name.clone()));
        map.insert(String::from("sur"), Value::String(self.sur_name.clone()));
        map.insert(String::from("belt"), Value::String(self.belt.serialise()));
        map.insert(String::from("year"), Value::Number(self.birth_year.into()));
        Value::Object(map)
    }
}

pub struct Sender {
    club_name: String,
    given_name: String,
    sur_name: String,
    address: String,
    postal_code: u16,
    town: String,
    private_phone: String,
    public_phone: String,
    fax: String,
    mobile: String,
    mail: String
}

impl Sender {
    #[allow(clippy::too_many_arguments)]
    pub fn new(club_name: String, given_name: String, sur_name: String, address: String,
            postal_code: u16, town: String, private_phone: String, public_phone: String,
            fax: String, mobile: String, mail: String) -> Self {
        Sender {
            club_name, given_name, sur_name, address, postal_code, town, private_phone,
            public_phone, fax, mobile, mail
        } 
    }

    pub fn render(&self) -> String {
        format!(
            include_str!("sender-format"),
            self.club_name, self.given_name, self.sur_name, self.address, self.postal_code, self.town, self.private_phone, self.public_phone,
            self.fax, self.mobile, self.mail
        )
    }
}

pub struct Club {
    name: String,
    number: u64,
    sender: Sender,
    county: String,
    region: String,
    state: String,
    group: String,
    nation: String
}

impl Club {
    #[allow(clippy::too_many_arguments)]
    pub fn new(club_name: String, club_number: u64, sender: Sender, county: String, region: String, state: String, group: String, nation: String) -> Self {
        Club {
            name: club_name, number: club_number, sender, county, region, state, group, nation
        }
    }

    pub fn render(&self) -> String {
        format!(
            include_str!("club-format"),
            self.name, self.number, self.sender.sur_name, self.sender.given_name, self.sender.address,
            self.sender.postal_code, self.sender.town, self.sender.private_phone, self.sender.public_phone,
            self.sender.mobile, self.sender.mail, self.sender.fax, self.county, self.region, self.state, self.group,
            self.nation
        )
    }
}

pub enum GenderCategory {
    Mixed,
    Male,
    Female
}

impl GenderCategory {
    pub fn render(&self) -> &'static str {
        match self {
            Self::Female => "w",
            Self::Male => "m",
            Self::Mixed => "g"
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "w" => Self::Female,
            "m" => Self::Male,
            "g" => Self::Mixed,
            _ => {
                return None;
            }
        })
    }
}

pub struct Tournament {
    name: String,
    date: DateTime<Local>,
    place: String,
    age_category: String,
    gender_category: GenderCategory,
    club: Club,
    athletes: Vec<Athlete>
}

impl Tournament {
    pub fn new(name: String, date: DateTime<Local>, place: String, age_category: String, gender: GenderCategory, club: Club, athletes: Vec<Athlete>) -> Self {
        Self {
            name, date, place, age_category, gender_category: gender, club, athletes
        }
    }

    pub fn render(&self) -> String {
        format!(
            include_str!("tournament-format"),
            self.club.sender.render(), self.name, self.date.format("%d.%m.%Y"), self.place,
            self.age_category, self.gender_category.render(), self.gender_category.render(), self.club.render(), render(&self.athletes), self.athletes.len()
        )
    }
}

fn render(athletes: &[Athlete]) -> String {
    let mut ret = String::new();
    for (i, athlete) in athletes.iter().enumerate() {
        ret.push_str(&format!(
            "{}=\"\"{}\",{}\"",
            i + 1, i + 1, athlete.render()
        ));
        if i < athletes.len() - 1 {
            ret.push('\n');
        }
    }
    ret
}
