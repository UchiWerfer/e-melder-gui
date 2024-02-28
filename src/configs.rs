use std::fs::File;
use std::io;
use std::io::ErrorKind::Other as Other;
use std::io::Read;
use std::io::Write;
use std::path::Path;

use crate::tournament_info::{Athlete, Belt, Club, Sender, Tournament, WeightCategory};

fn string_to_iso_8859_1_bytes(s: &str) -> Vec<u8> {
    s.chars().map(|c| { c as u8 }).collect()
}

#[allow(clippy::cast_possible_truncation)]
pub fn read_club(path: impl AsRef<Path>) -> io::Result<Club> {
    let mut file = File::options().read(true).open(path)?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;
    let serde_value: serde_json::Value = serde_json::from_str(&s)?;
    let value = serde_value.as_object().ok_or(io::Error::new(Other, "did not find a map"))?;
    let club_name = String::from(value.get("club").ok_or(io::Error::new(Other, "club-name not provided"))?
        .as_str().ok_or(io::Error::new(Other, "club-name not a string"))?);
    let sender_given_name = String::from(value.get("given").ok_or(io::Error::new(Other, "sender's given name not provided"))?
        .as_str().ok_or(io::Error::new(Other, "given name not a string"))?);
    let sender_sur_name = String::from(value.get("sur").ok_or(io::Error::new(Other, "sender's surname not provided"))?
        .as_str().ok_or(io::Error::new(Other, "surname not a string"))?);
    let address = String::from(value.get("address").ok_or(io::Error::new(Other, "address not provided"))?
        .as_str().ok_or(io::Error::new(Other, "address not a string"))?);
    let postal_code = value.get("postal-code").ok_or(io::Error::new(Other, "postal code not provided"))?.as_u64()
        .ok_or(io::Error::new(Other, "postal code not a number"))? as u16;
    let town = String::from(value.get("town").ok_or(io::Error::new(Other, "town not provided"))?
        .as_str().ok_or(io::Error::new(Other, "town not a string"))?);
    let private = String::from(value.get("private").ok_or(io::Error::new(Other, "private phone number not provided"))?
        .as_str().ok_or(io::Error::new(Other, "private phone number not a string"))?);
    let public = String::from(value.get("public").ok_or(io::Error::new(Other, "public phone number not provided"))?
        .as_str().ok_or(io::Error::new(Other, "public phone number not a string"))?);
    let fax = String::from(value.get("fax").ok_or(io::Error::new(Other, "fax number not provided"))?
        .as_str().ok_or(io::Error::new(Other, "fax number not a string"))?);
    let mobile = String::from(value.get("mobile").ok_or(io::Error::new(Other, "mobile number not provided"))?
        .as_str().ok_or(io::Error::new(Other, "mobile number not a string"))?);
    let mail = String::from(value.get("mail").ok_or(io::Error::new(Other, "e-mail address not provided"))?
        .as_str().ok_or(io::Error::new(Other, "e-mail address not a string"))?);
    let club_number = value.get("club-number").ok_or(io::Error::new(Other, "club-number not provided"))?.as_u64()
        .ok_or(io::Error::new(Other, "club-number not a number"))?;
    let county = String::from(value.get("county").ok_or(io::Error::new(Other, "county not provided"))?
        .as_str().ok_or(io::Error::new(Other, "county not a string"))?);
    let region = String::from(value.get("region").ok_or(io::Error::new(Other, "region not provided"))?
        .as_str().ok_or(io::Error::new(Other, "region not a string"))?);
    let state = String::from(value.get("state").ok_or(io::Error::new(Other, "state not provided"))?
        .as_str().ok_or(io::Error::new(Other, "state not a string"))?);
    let group = String::from(value.get("group").ok_or(io::Error::new(Other, "group not provided"))?
        .as_str().ok_or(io::Error::new(Other, "group not a string"))?);
    let nation = String::from(value.get("nation").ok_or(io::Error::new(Other, "nation not provided"))?
        .as_str().ok_or(io::Error::new(Other, "nation not a string"))?);
    let sender = Sender::new(club_name.clone(), sender_given_name, sender_sur_name, address, postal_code, town, private, public,
        fax, mobile, mail);
    Ok(Club::new(club_name, club_number, sender, county, region, state, group, nation))
}

#[allow(clippy::cast_possible_truncation)]
pub fn read_athletes(path: impl AsRef<Path>) -> io::Result<Vec<Athlete>> {
    let mut file = File::options().read(true).open(path)?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;
    let serde_value: serde_json::Value = serde_json::from_str(&s)?;
    let value = serde_value.as_array().ok_or(io::Error::new(Other, "did not find a list"))?;
    let mut ret = Vec::with_capacity(value.len());
    for serde_athlete in value {
        let athlete = serde_athlete.as_object().ok_or(io::Error::new(Other, "athlete not a map"))?;
        let athlete_given_name = String::from(athlete.get("given").ok_or(io::Error::new(Other, "athlete's given name not provided"))?
            .as_str().ok_or(io::Error::new(Other, "athlete's given name not a string"))?);
        let athlete_sur_name =  String::from(athlete.get("sur").ok_or(io::Error::new(Other, "athlete's surname not provided"))?
            .as_str().ok_or(io::Error::new(Other, "athlete's surname not a string"))?);
        let belt_str = athlete.get("belt").ok_or(io::Error::new(Other, "athlete's belt not provided"))?
            .as_str().ok_or(io::Error::new(Other, "athlete's belt not a string"))?;
        let belt = Belt::from_str(belt_str).ok_or(io::Error::new(Other, "belt format not understood"))?;
        let year = athlete.get("year").ok_or(io::Error::new(Other, "athlete's birth year not provided"))?.as_u64()
            .ok_or(io::Error::new(Other, "athlete's birth year not an integer"))? as u16;
        ret.push(Athlete::new(athlete_given_name, athlete_sur_name, year, belt, WeightCategory::default()));
    }
    Ok(ret)
}

pub fn write_tournament(path: impl AsRef<Path>, tournament: &Tournament) -> io::Result<()> {
    let mut file = File::options().write(true).create(true).truncate(true).open(path)?;
    file.write_all(&string_to_iso_8859_1_bytes(&tournament.render()))?;
    Ok(())
}

pub fn write_athletes(path: impl AsRef<Path>, athletes: &[Athlete]) -> io::Result<()> {
    let mut values: Vec<serde_json::Value> = Vec::with_capacity(athletes.len());
    for athlete in athletes {
        values.push(athlete.serialise());
    }

    let mut file = File::options().write(true).create(true).truncate(true).open(path)?;
    file.write_all(serde_json::to_string(&values)?.as_bytes())?;
    Ok(())
}
