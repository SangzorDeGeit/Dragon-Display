use std::fs::File;
use serde::{Deserialize, Serialize};
use toml::to_string;
use std::env;

#[derive(Serialize)]
struct Campaign {
    name: String,
    path: String,
    sync_option: String
}

pub fn read_campaign_from_config() -> Option<List<Campaign>> {
    todo!();
}

pub fn write_campaign_to_config(campaign: Campaign) -> Bool {
    todo!();
}

pub fn remove_campaign_from_config(campaign: Campaign) -> Bool {
    todo!();
}

fn get_campaign_config() -> Option<File> {
    todo!();
}

