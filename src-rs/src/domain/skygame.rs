use std::{collections::HashMap, sync::OnceLock};

use serde::Deserialize;
use serde_json::json;
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

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkyCalendarQuery {
    pub start_date: String,
    pub end_date: String,
    pub kinds: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkyItemSearchQuery {
    #[serde(default)]
    pub query: String,
    pub types: Option<Vec<String>>,
    pub wishlist: Option<HashMap<String, bool>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkyRouteFilters {
    pub spirits: Option<bool>,
    pub winged_lights: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkyActiveRoute {
    pub area_guid: Option<String>,
    #[serde(default)]
    pub target_index: usize,
    #[serde(default)]
    pub filters: SkyRouteFilters,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkyRouteProgress {
    #[serde(default)]
    pub completed_targets: HashMap<String, bool>,
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

    pub fn candle_runs(&self) -> Vec<Value> {
        self.raw_candle_runs()
            .iter()
            .map(|run| {
                json!({
                    "guid": string_field(run, "guid").unwrap_or_default(),
                    "name": string_field(run, "name").unwrap_or_default(),
                    "imageUrl": run.get("imageUrl").cloned().unwrap_or(Value::Null),
                    "groupCount": array_field(run, "groups").map_or(0, Vec::len),
                    "waxCount": count_candle_run_wax(run),
                })
            })
            .collect()
    }

    pub fn candle_run(&self, guid: &str) -> Option<Value> {
        self.candle_run_index(guid)
            .and_then(|index| self.raw_candle_runs().get(index))
            .cloned()
    }

    pub fn realms(&self) -> Vec<Value> {
        array_at(&self.bundle, &["realms"])
            .expect("bundled skygame realms are valid")
            .iter()
            .filter(|realm| realm.get("hidden").and_then(Value::as_bool) != Some(true))
            .cloned()
            .collect()
    }

    pub fn realm(&self, guid: &str) -> Option<Value> {
        self.realm_index(guid)
            .and_then(|index| array_at(&self.bundle, &["realms"]).ok()?.get(index))
            .cloned()
    }

    pub fn area(&self, guid: &str) -> Option<Value> {
        self.area_index(guid)
            .and_then(|index| array_at(&self.bundle, &["areas"]).ok()?.get(index))
            .cloned()
    }

    pub fn areas_for_realm(&self, realm_guid: &str) -> Vec<Value> {
        array_at(&self.bundle, &["areas"])
            .expect("bundled skygame areas are valid")
            .iter()
            .filter(|area| string_field(area, "realmGuid") == Some(realm_guid))
            .cloned()
            .collect()
    }

    pub fn calendar_entries(&self, query: &SkyCalendarQuery) -> Vec<Value> {
        let allowed_kinds = query
            .kinds
            .as_ref()
            .map(|kinds| kinds.iter().map(String::as_str).collect::<Vec<_>>());

        array_at(&self.bundle, &["calendarEntries"])
            .expect("bundled skygame calendar entries are valid")
            .iter()
            .filter(|entry| {
                if let Some(allowed_kinds) = &allowed_kinds {
                    if !string_field(entry, "kind")
                        .is_some_and(|kind| allowed_kinds.contains(&kind))
                    {
                        return false;
                    }
                }

                string_field(entry, "date").is_some_and(|date| date <= query.end_date.as_str())
                    && string_field(entry, "endDate")
                        .is_some_and(|end_date| end_date >= query.start_date.as_str())
            })
            .cloned()
            .collect()
    }

    pub fn search_items(&self, query: &SkyItemSearchQuery) -> Vec<Value> {
        let normalized_query = query.query.trim().to_lowercase();
        let allowed_types = query
            .types
            .as_ref()
            .map(|types| types.iter().map(String::as_str).collect::<Vec<_>>());

        array_at(&self.bundle, &["items"])
            .expect("bundled skygame items are valid")
            .iter()
            .filter(|item| {
                if let Some(wishlist) = &query.wishlist {
                    if !string_field(item, "guid")
                        .is_some_and(|guid| wishlist.get(guid) == Some(&true))
                    {
                        return false;
                    }
                }

                if let Some(allowed_types) = &allowed_types {
                    if !string_field(item, "type")
                        .is_some_and(|item_type| allowed_types.contains(&item_type))
                    {
                        return false;
                    }
                }

                normalized_query.is_empty()
                    || string_field(item, "name")
                        .is_some_and(|name| contains_lower(name, &normalized_query))
                    || string_field(item, "type")
                        .is_some_and(|item_type| contains_lower(item_type, &normalized_query))
                    || array_field(item, "origins").is_some_and(|origins| {
                        origins.iter().any(|origin| {
                            string_field(origin, "name")
                                .is_some_and(|name| contains_lower(name, &normalized_query))
                        })
                    })
            })
            .take(80)
            .cloned()
            .collect()
    }

    pub fn item_detail(&self, guid: &str) -> Option<Value> {
        self.item_index(guid)
            .and_then(|index| array_at(&self.bundle, &["items"]).ok()?.get(index))
            .cloned()
    }

    pub fn realm_route(&self, realm_guid: &str) -> Option<Value> {
        self.realm_route_index(realm_guid)
            .and_then(|index| {
                array_at(&self.bundle, &["routes", "realms"])
                    .ok()?
                    .get(index)
            })
            .cloned()
    }

    pub fn area_route(&self, area_guid: &str) -> Option<Value> {
        self.area_route_index(area_guid)
            .and_then(|index| {
                array_at(&self.bundle, &["routes", "areas"])
                    .ok()?
                    .get(index)
            })
            .cloned()
    }

    pub fn route_targets(&self, area_guid: &str, filters: &SkyRouteFilters) -> Vec<Value> {
        array_at(&self.bundle, &["routes", "targets"])
            .expect("bundled skygame route targets are valid")
            .iter()
            .filter(|target| string_field(target, "areaGuid") == Some(area_guid))
            .filter(|target| kind_allowed(string_field(target, "kind"), filters))
            .cloned()
            .collect()
    }

    pub fn route_target(&self, guid: &str) -> Option<Value> {
        self.target_index(guid)
            .and_then(|index| {
                array_at(&self.bundle, &["routes", "targets"])
                    .ok()?
                    .get(index)
            })
            .cloned()
    }

    pub fn mini_map_pins(&self, area_guid: &str, filters: &SkyRouteFilters) -> Vec<Value> {
        array_at(&self.bundle, &["routes", "pins"])
            .expect("bundled skygame mini-map pins are valid")
            .iter()
            .filter(|pin| string_field(pin, "areaGuid") == Some(area_guid))
            .filter(|pin| kind_allowed(string_field(pin, "kind"), filters))
            .cloned()
            .collect()
    }

    pub fn active_route_target(
        &self,
        active_route: Option<&SkyActiveRoute>,
        progress: Option<&SkyRouteProgress>,
    ) -> Option<Value> {
        let active_route = active_route?;
        let area_guid = active_route.area_guid.as_deref()?;
        let targets = self.route_targets(area_guid, &active_route.filters);
        if targets.is_empty() {
            return None;
        }

        let target_index = active_route
            .target_index
            .min(targets.len().saturating_sub(1));
        let target = targets[target_index].clone();
        let completed_targets = progress.map(|value| &value.completed_targets);
        let completed = target_completed(&target, completed_targets);
        let completed_count = targets
            .iter()
            .filter(|candidate| target_completed(candidate, completed_targets))
            .count();

        Some(json!({
            "target": target,
            "targetIndex": target_index,
            "targets": targets,
            "completed": completed,
            "total": targets.len(),
            "completedCount": completed_count,
        }))
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

    fn raw_candle_runs(&self) -> &Vec<Value> {
        array_at(&self.bundle, &["sourceGroups", "candles"])
            .expect("bundled skygame candle runs are valid")
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

fn string_field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn array_field<'a>(value: &'a Value, key: &str) -> Option<&'a Vec<Value>> {
    value.get(key).and_then(Value::as_array)
}

fn contains_lower(value: &str, needle: &str) -> bool {
    value.to_lowercase().contains(needle)
}

fn kind_allowed(kind: Option<&str>, filters: &SkyRouteFilters) -> bool {
    match kind {
        Some("spirit") => filters.spirits != Some(false),
        Some("winged-light") => filters.winged_lights != Some(false),
        _ => true,
    }
}

fn target_completed(target: &Value, completed_targets: Option<&HashMap<String, bool>>) -> bool {
    let Some(completed_targets) = completed_targets else {
        return false;
    };

    string_field(target, "guid").is_some_and(|guid| completed_targets.get(guid) == Some(&true))
        || string_field(target, "sourceGuid")
            .is_some_and(|guid| completed_targets.get(guid) == Some(&true))
}

fn count_candle_run_wax(run: &Value) -> i64 {
    array_field(run, "groups")
        .into_iter()
        .flatten()
        .map(count_candle_group_wax)
        .sum()
}

fn count_candle_group_wax(group: &Value) -> i64 {
    array_field(group, "candles")
        .into_iter()
        .flatten()
        .filter_map(|candle| candle.get("c").and_then(Value::as_i64))
        .sum()
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

    #[test]
    fn returns_dated_calendar_entries() {
        let entries = sky_game_data().calendar_entries(&SkyCalendarQuery {
            start_date: "2026-01-01".to_string(),
            end_date: "2026-12-31".to_string(),
            kinds: None,
        });

        assert!(entries
            .iter()
            .all(|entry| string_field(entry, "date").is_some()
                && string_field(entry, "endDate").is_some()));
    }

    #[test]
    fn searches_items_by_name_or_origin() {
        let results = sky_game_data().search_items(&SkyItemSearchQuery {
            query: "cape".to_string(),
            ..SkyItemSearchQuery::default()
        });

        assert!(!results.is_empty());
    }

    #[test]
    fn returns_route_targets_grouped_by_area() {
        let data = sky_game_data();
        let realm = data
            .realms()
            .into_iter()
            .find(|realm| {
                string_field(realm, "guid")
                    .and_then(|guid| data.realm_route(guid))
                    .and_then(|route| route["counts"]["total"].as_u64())
                    .unwrap_or(0)
                    > 0
            })
            .unwrap();
        let area = data
            .areas_for_realm(string_field(&realm, "guid").unwrap())
            .into_iter()
            .find(|area| {
                string_field(area, "guid")
                    .and_then(|guid| data.area_route(guid))
                    .and_then(|route| route["counts"]["total"].as_u64())
                    .unwrap_or(0)
                    > 0
            })
            .unwrap();
        let area_guid = string_field(&area, "guid").unwrap();
        let targets = data.route_targets(area_guid, &SkyRouteFilters::default());

        assert!(!targets.is_empty());
        assert!(targets
            .iter()
            .all(|target| string_field(target, "areaGuid") == Some(area_guid)));
    }

    #[test]
    fn returns_mini_map_pins_for_route_targets_with_map_support() {
        let data = sky_game_data();
        let area = data
            .realms()
            .into_iter()
            .flat_map(|realm| data.areas_for_realm(string_field(&realm, "guid").unwrap()))
            .find(|area| {
                string_field(area, "guid")
                    .map(|guid| {
                        !data
                            .mini_map_pins(guid, &SkyRouteFilters::default())
                            .is_empty()
                    })
                    .unwrap_or(false)
            })
            .unwrap();
        let pins = data.mini_map_pins(
            string_field(&area, "guid").unwrap(),
            &SkyRouteFilters::default(),
        );

        assert!(!pins.is_empty());
        assert!(pins.iter().all(|pin| pin["x"]
            .as_f64()
            .is_some_and(|x| (0.0..=100.0).contains(&x))));
        assert!(pins.iter().all(|pin| pin["y"]
            .as_f64()
            .is_some_and(|y| (0.0..=100.0).contains(&y))));
    }
}
