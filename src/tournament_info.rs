use std::collections::HashMap;
use std::str::FromStr;

use chrono::NaiveDate;
use enum_map::Enum;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all="lowercase")]
pub enum Belt {
    #[default]
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
        // number used for serialisation by the official application
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

#[derive(Default, Clone, Copy, Debug)]
pub enum WeightCategoryKind {
    #[default]
    Under,
    Over
}

#[derive(Clone, Copy, Debug)]
pub struct WeightCategory {
    kind: WeightCategoryKind,
    limit: u8
}

impl Default for WeightCategory {
    fn default() -> Self {
        Self { limit: 10, kind: WeightCategoryKind::default() }
    }
}

impl WeightCategory {
    pub fn render(self) -> String {
        // the official application renders weight categories weirdly,
        // but we have to render them accordingly
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

    #[allow(clippy::inherent_to_string)]
    fn to_string(self) -> String {
        match self.kind {
            WeightCategoryKind::Under => format!("-{}", self.limit),
            WeightCategoryKind::Over => format!("+{}", self.limit)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Athlete {
    #[serde(rename="given")]
    given_name: String,
    #[serde(rename="sur")]
    sur_name: String,
    belt: Belt,
    #[serde(skip)]
    weight_category: WeightCategory,
    #[serde(rename="year")]
    birth_year: u16,
    #[serde(default, serialize_with="crate::utils::serialize_gender_category",
    deserialize_with="crate::utils::deserialize_gender_category")]
    gender: GenderCategory
}

impl Athlete {
    pub fn new(given_name: String, sur_name: String, birth_year: u16, belt: Belt, weight_category: WeightCategory, gender: GenderCategory) -> Self {
        Self { given_name, sur_name, belt, weight_category, birth_year, gender }
    }

    pub fn render(&self) -> String {
        // the official application renders athletes weirdly, but 
        // we have to render them accordingly
        format!(include_str!("athlete-format"), self.sur_name, self.given_name, self.belt.render(), self.weight_category.render(), self.birth_year)
    }

    pub fn get_given_name(&self) -> &str {
        &self.given_name
    }

    pub fn get_sur_name(&self) -> &str {
        &self.sur_name
    }

    pub fn get_belt(&self) -> &Belt {
        &self.belt
    }

    pub fn get_belt_mut(&mut self) -> &mut Belt {
        &mut self.belt
    }

    pub fn get_birth_year(&self) -> u16 {
        self.birth_year
    }

    pub fn get_gender(&self) -> GenderCategory {
        self.gender
    }

    pub fn get_gender_mut(&mut self) -> &mut GenderCategory {
        &mut self.gender
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Sender {
    #[serde(rename="given")]
    given_name: String,
    #[serde(rename="sur")]
    sur_name: String,
    address: String,
    #[serde(rename="postal-code")]
    postal_code: u32,
    town: String,
    #[serde(rename="private")]
    private_phone: String,
    #[serde(rename="public")]
    public_phone: String,
    fax: String,
    mobile: String,
    mail: String
}

impl Sender {
    pub fn render(&self, club_name: &str) -> String {
        // the format here resembles toml, but is not toml
        format!(
            include_str!("sender-format"),
            club_name, self.given_name, self.sur_name, self.address, self.postal_code, self.town, self.private_phone, self.public_phone,
            self.fax, self.mobile, self.mail
        )
    }

    pub fn get_given_name_mut(&mut self) -> &mut String {
        &mut self.given_name
    }

    pub fn get_sur_name_mut(&mut self) -> &mut String {
        &mut self.sur_name
    }

    pub fn get_address_mut(&mut self) -> &mut String {
        &mut self.address
    }

    pub fn get_postal_code_mut(&mut self) -> &mut u32 {
        &mut self.postal_code
    }

    pub fn get_town_mut(&mut self) -> &mut String {
        &mut self.town
    }

    pub fn get_private_phone_mut(&mut self) -> &mut String {
        &mut self.private_phone
    }

    pub fn get_public_phone_mut(&mut self) -> &mut String {
        &mut self.public_phone
    }

    pub fn get_fax_mut(&mut self) -> &mut String {
        &mut self.fax
    }

    pub fn get_mobile_mut(&mut self) -> &mut String {
        &mut self.mobile
    }

    pub fn get_mail_mut(&mut self) -> &mut String {
        &mut self.mail
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Club {
    #[serde(rename="club")]
    name: String,
    #[serde(rename="club-number")]
    number: u64,
    #[serde(flatten)]
    sender: Sender,
    county: String,
    region: String,
    state: String,
    group: String,
    nation: String
}

impl Club {
    pub fn render(&self) -> String {
        format!(
            include_str!("club-format"),
            self.name, self.number, self.sender.sur_name, self.sender.given_name, self.sender.address,
            self.sender.postal_code, self.sender.town, self.sender.private_phone, self.sender.public_phone,
            self.sender.mobile, self.sender.mail, self.sender.fax, self.county, self.region, self.state, self.group,
            self.nation
        )
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    pub fn get_number_mut(&mut self) -> &mut u64 {
        &mut self.number
    }

    pub fn get_sender_mut(&mut self) -> &mut Sender {
        &mut self.sender
    }

    pub fn get_county_mut(&mut self) -> &mut String {
        &mut self.county
    }

    pub fn get_region_mut(&mut self) -> &mut String {
        &mut self.region
    }

    pub fn get_state_mut(&mut self) -> &mut String {
        &mut self.state
    }

    pub fn get_group_mut(&mut self) -> &mut String {
        &mut self.group
    }

    pub fn get_nation_mut(&mut self) -> &mut String {
        &mut self.nation
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Default, Enum)]
pub enum GenderCategory {
    #[default]
    Mixed,
    Male,
    Female
}

impl GenderCategory {
    pub fn render(self) -> &'static str {
        // the official application uses German abreviations
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
            _ => return None
        })
    }
}

pub struct Tournament {
    name: String,
    date: NaiveDate,
    place: String,
    age_category: String,
    gender_category: GenderCategory,
    club: Club,
    athletes: Vec<Athlete>
}

impl Tournament {
    pub fn new(name: String, date: NaiveDate, place: String, age_category: String, gender: GenderCategory, club: Club, athletes: Vec<Athlete>) -> Self {
        Self {
            name, date, place, age_category, gender_category: gender, club, athletes
        }
    }

    pub fn render(&self) -> String {
        // the formet here resembles toml, but is not toml, the date is in the usual German format
        format!(
            include_str!("tournament-format"),
            self.club.sender.render(self.club.get_name()), self.name, self.date.format("%d.%m.%Y"), self.place,
            self.age_category, self.gender_category.render(), self.gender_category.render(), self.club.render(), render(&self.athletes), self.athletes.len()
        )
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_age_category(&self) -> &str {
        &self.age_category
    }

    pub fn get_gender_category(&self) -> GenderCategory {
        self.gender_category
    }
}

fn render(athletes: &[Athlete]) -> String {
    let mut ret = String::new();
    for (i, athlete) in athletes.iter().enumerate() {
        ret.push_str(&format!(
            "{}=\"\"1\",{}\"",
            i + 1, athlete.render()
        ));
        if i < athletes.len() - 1 {
            ret.push('\n');
        }
    }
    ret
}

#[derive(Debug)]
pub struct RegisteringAthlete {
    given_name: String,
    sur_name: String,
    belt: Belt,
    weight_category: String,
    birth_year: u16,
    gender_category: GenderCategory,
    gender: GenderCategory,
    age_category: String
}

impl RegisteringAthlete {
    pub fn new(given_name: String, sur_name: String, belt: Belt, weight_category: String, birth_year: u16, gender: GenderCategory,
    age_category: String) -> Self {
        Self {
            given_name, sur_name, belt, weight_category, birth_year, gender_category: gender, gender, age_category
        }
    }

    pub fn from_athlete(athlete: &Athlete) -> Self {
        Self::new(athlete.given_name.clone(), athlete.sur_name.clone(), athlete.belt,
        athlete.weight_category.to_string(), athlete.birth_year, athlete.gender, String::new())
    }

    pub fn get_given_name(&self) -> &str {
        &self.given_name
    }

    pub fn get_sur_name(&self) -> &str {
        &self.sur_name
    }

    pub fn get_belt(&self) -> Belt {
        self.belt
    }

    pub fn get_weight_category_mut(&mut self) -> &mut String {
        &mut self.weight_category
    }

    pub fn get_birth_year(&self) -> u16 {
        self.birth_year
    }

    pub fn get_gender_category(&self) -> &GenderCategory {
        &self.gender_category
    }

    pub fn get_gender_category_mut(&mut self) -> &mut GenderCategory {
        &mut self.gender_category
    }

    pub fn get_age_category_mut(&mut self) -> &mut String {
        &mut self.age_category
    }

    pub fn get_gender(&self) -> GenderCategory {
        self.gender
    }
}

pub fn registering_athletes_to_tournaments(registering_athletes: &[RegisteringAthlete], name: &str, date: NaiveDate,
place: &str, club: &Club) -> Option<Vec<Tournament>> {
    let mut tournament_meta: HashMap<(&str, GenderCategory), usize> = HashMap::new();
    let mut ret: Vec<Tournament> = Vec::new();

    for registering_athlete in registering_athletes {
        let index_opt = tournament_meta.get(&(&registering_athlete.age_category, registering_athlete.gender_category));
        if let Some(index) = index_opt {
            ret[*index].athletes.push(Athlete::new(registering_athlete.given_name.clone(), registering_athlete.sur_name.clone(),
                registering_athlete.birth_year, registering_athlete.belt,
                WeightCategory::from_str(&registering_athlete.weight_category)?, registering_athlete.gender_category));
        }
        else {
            ret.push(
                Tournament::new(name.to_owned(), date, place.to_owned(), registering_athlete.age_category.clone(),
                registering_athlete.gender_category, club.clone(), vec![Athlete::new(
                    registering_athlete.given_name.clone(), registering_athlete.sur_name.clone(), registering_athlete.birth_year,
                    registering_athlete.belt, WeightCategory::from_str(&registering_athlete.weight_category)?, registering_athlete.gender
                )])
            );
            tournament_meta.insert((&registering_athlete.age_category, registering_athlete.gender_category), ret.len() - 1);
        }
    }
    Some(ret)
}
