use std::{collections::HashMap, sync::OnceLock};

use serde_json::Value;

use super::{DomainError, DomainResult};

const SKY_GAME_DATA_JSON: &str = include_str!("../data/skygame/selected-data.json");

static SKY_GAME_DATA: OnceLock<SkyGameData> = OnceLock::new();

#[derive(Debug)]
pub struct SkyGameData {
    bundle: Value,
    item_indexes_by_guid: HashMap<String, usize>,
    realm_indexes_by_guid: HashMap<String, usize>,
    area_indexes_by_guid: HashMap<String, usize>,
    realm_route_indexes_by_guid: HashMap<String, usize>,
    area_route_indexes_by_guid: HashMap<String, usize>,
    target_indexes_by_guid: HashMap<String, usize>,
    candle_run_indexes_by_guid: HashMap<String, usize>,
}

pub fn sky_game_data() -> &'static SkyGameData {
    SKY_GAME_DATA.get_or_init(|| SkyGameData::load().expect("bundled skygame data is valid"))
}

impl SkyGameData {
    fn load() -> DomainResult<Self> {
        let bundle = serde_json::from_str::<Value>(SKY_GAME_DATA_JSON)
            .map_err(|error| DomainError::new("invalid_skygame_data", error.to_string()))?;

        Ok(Self {
            item_indexes_by_guid: index_by_guid(array_at(&bundle, &["items"])?),
            realm_indexes_by_guid: index_by_guid(array_at(&bundle, &["realms"])?),
            area_indexes_by_guid: index_by_guid(array_at(&bundle, &["areas"])?),
            realm_route_indexes_by_guid: index_by_guid(array_at(&bundle, &["routes", "realms"])?),
            area_route_indexes_by_guid: index_by_guid(array_at(&bundle, &["routes", "areas"])?),
            target_indexes_by_guid: index_by_guid(array_at(&bundle, &["routes", "targets"])?),
            candle_run_indexes_by_guid: index_by_guid(array_at(
                &bundle,
                &["sourceGroups", "candles"],
            )?),
            bundle,
        })
    }

    pub fn bundle(&self) -> &Value {
        &self.bundle
    }

    pub fn meta(&self) -> &Value {
        &self.bundle["meta"]
    }

    pub fn stats(&self) -> &Value {
        &self.bundle["stats"]
    }

    pub fn source_stats(&self) -> &Value {
        &self.bundle["sourceStats"]
    }

    pub fn source_groups(&self) -> &Value {
        &self.bundle["sourceGroups"]
    }

    pub fn item_index(&self, guid: &str) -> Option<usize> {
        self.item_indexes_by_guid.get(guid).copied()
    }

    pub fn realm_index(&self, guid: &str) -> Option<usize> {
        self.realm_indexes_by_guid.get(guid).copied()
    }

    pub fn area_index(&self, guid: &str) -> Option<usize> {
        self.area_indexes_by_guid.get(guid).copied()
    }

    pub fn realm_route_index(&self, guid: &str) -> Option<usize> {
        self.realm_route_indexes_by_guid.get(guid).copied()
    }

    pub fn area_route_index(&self, guid: &str) -> Option<usize> {
        self.area_route_indexes_by_guid.get(guid).copied()
    }

    pub fn target_index(&self, guid: &str) -> Option<usize> {
        self.target_indexes_by_guid.get(guid).copied()
    }

    pub fn candle_run_index(&self, guid: &str) -> Option<usize> {
        self.candle_run_indexes_by_guid.get(guid).copied()
    }
}

fn array_at<'a>(bundle: &'a Value, path: &[&str]) -> DomainResult<&'a Vec<Value>> {
    let mut value = bundle;

    for key in path {
        value = value.get(*key).ok_or_else(|| {
            DomainError::new(
                "invalid_skygame_data",
                format!("Missing {}", path.join(".")),
            )
        })?;
    }

    value.as_array().ok_or_else(|| {
        DomainError::new(
            "invalid_skygame_data",
            format!("Expected array at {}", path.join(".")),
        )
    })
}

fn index_by_guid(items: &[Value]) -> HashMap<String, usize> {
    items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            item.get("guid")
                .and_then(Value::as_str)
                .map(|guid| (guid.to_string(), index))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_bundled_data() {
        let data = sky_game_data();

        assert_eq!(data.meta()["source"], "skygame-data");
        assert!(data.stats()["items"].as_u64().unwrap_or(0) > 1000);
    }

    #[test]
    fn builds_expected_source_stats_and_indexes() {
        let data = sky_game_data();

        assert!(data.source_stats()["realms"].as_u64().unwrap_or(0) > 0);
        assert!(data.source_stats()["wingedLights"].as_u64().unwrap_or(0) > 100);
        assert!(!data.item_indexes_by_guid.is_empty());
        assert!(!data.realm_indexes_by_guid.is_empty());
        assert!(!data.area_indexes_by_guid.is_empty());
        assert!(!data.candle_run_indexes_by_guid.is_empty());
    }
}
