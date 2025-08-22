use std::{fs, io::Error};

use serde::Deserialize;

// Based on GameMaker runner version (so Linux v1.001 on Windows falls under Undertale_Windows_v1_001)
#[derive(Deserialize, Clone, Copy)]
#[expect(non_camel_case_types)]
pub enum ConfigRunnerVersion {
    Undertale_Windows_v1_0,
    Undertale_Windows_v1_001,
    Undertale_Linux_v1_001,
    Undertale_Windows_v1_08,
    // TODO: add more supported configurations?
}
impl ConfigRunnerVersion {
    pub fn rng_15bit(&self) -> bool {
        match self {
            Self::Undertale_Windows_v1_0 => true,
            Self::Undertale_Windows_v1_001 => true,
            Self::Undertale_Linux_v1_001 => false,
            Self::Undertale_Windows_v1_08 => false,
        }
    }
    pub fn rng_signed(&self) -> bool {
        match self {
            Self::Undertale_Windows_v1_0 => false,
            Self::Undertale_Windows_v1_001 => false,
            Self::Undertale_Linux_v1_001 => false,
            Self::Undertale_Windows_v1_08 => true,
        }
    }
    pub fn rng_old_poly(&self) -> bool {
        match self {
            Self::Undertale_Windows_v1_0 => true,
            Self::Undertale_Windows_v1_001 => false,
            Self::Undertale_Linux_v1_001 => false,
            Self::Undertale_Windows_v1_08 => false,
        }
    }
    pub fn circle_draw_offset(&self) -> i32 {
        match self {
            Self::Undertale_Windows_v1_0 => 1,
            Self::Undertale_Windows_v1_001 => 1,
            Self::Undertale_Linux_v1_001 => 0,
            Self::Undertale_Windows_v1_08 => 1,
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct Config {
    pub runner_version: ConfigRunnerVersion,
    pub server_port: u16,
    pub hotkey_1_name: String,
    pub hotkey_2_name: String,
    pub hotkey_3_name: String,
    pub hotkey_4_name: String,
}
impl Config {
    pub fn read() -> Result<Self, Error> {
        let contents = fs::read_to_string("config.json")?;
        let config: Self = serde_json::from_str(&contents)?;

        Ok(config)
    }
}
