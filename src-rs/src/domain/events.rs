use std::{cmp::Ordering, collections::HashMap};

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use super::{
    shards::{get_shard_windows, ShardColor},
    sky_time::{
        format_local_time, format_sky_time, sky_date_for_instant, sky_wall_time_to_instant, SkyDate,
    },
};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventGenerationSettings {
    #[serde(default)]
    pub events: HashMap<String, bool>,
    #[serde(default = "default_time_format")]
    pub time_format: String,
    pub local_time_zone: Option<String>,
    #[serde(default = "default_overlay_max_events")]
    pub overlay_max_events: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventInstance {
    pub definition_id: String,
    pub title: String,
    pub category: String,
    pub status: String,
    pub starts_at_utc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ends_at_utc: Option<String>,
    pub sky_time_label: String,
    pub local_time_label: String,
    pub countdown_ms: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    pub source: String,
    pub priority: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reward_label: Option<String>,
}

#[derive(Debug, Clone)]
struct EventDefinition {
    id: &'static str,
    location: Option<&'static str>,
    source: &'static str,
    priority: i32,
}

#[derive(Debug, Clone)]
struct ScheduledEvent {
    definition_id: &'static str,
    title: String,
    category: &'static str,
    starts_at: DateTime<Utc>,
    starts_at_ms: i64,
    starts_at_utc: String,
    active_at_ms: i64,
    ends_at_ms: Option<i64>,
    ends_at_utc: Option<String>,
    sky_time_label: String,
    phase_label: Option<String>,
    active_phase_label: Option<String>,
    location: Option<String>,
    reward_label: Option<String>,
}

const EVENT_DEFINITIONS: &[EventDefinition] = &[
    EventDefinition {
        id: "daily-reset",
        location: None,
        source: "official",
        priority: 20,
    },
    EventDefinition {
        id: "eden-reset",
        location: None,
        source: "wiki",
        priority: 18,
    },
    EventDefinition {
        id: "geyser",
        location: Some("Sanctuary Islands"),
        source: "wiki",
        priority: 100,
    },
    EventDefinition {
        id: "grandma",
        location: Some("Elevated Clearing"),
        source: "wiki",
        priority: 95,
    },
    EventDefinition {
        id: "turtle",
        location: Some("Sanctuary Islands"),
        source: "wiki",
        priority: 90,
    },
    EventDefinition {
        id: "forest-rainbow",
        location: Some("Forest Brook"),
        source: "wiki",
        priority: 64,
    },
    EventDefinition {
        id: "shard-eruption",
        location: None,
        source: "community",
        priority: 85,
    },
    EventDefinition {
        id: "traveling-spirit",
        location: None,
        source: "wiki",
        priority: 35,
    },
];

pub fn generate_event_instances(
    now: DateTime<Utc>,
    settings: &EventGenerationSettings,
) -> Vec<EventInstance> {
    let now_ms = now.timestamp_millis();
    let sky_date = sky_date_for_instant(now);
    let mut events = generate_scheduled_events(sky_date)
        .into_iter()
        .filter(|event| settings.events.get(event.definition_id) != Some(&false))
        .map(|event| to_instance(event, now_ms, settings))
        .filter(|event| event.status != "ended" || event.countdown_ms < 30 * 60_000)
        .collect::<Vec<_>>();

    events.sort_by(sort_events);
    events
}

pub fn get_overlay_events(
    now: DateTime<Utc>,
    settings: &EventGenerationSettings,
) -> Vec<EventInstance> {
    generate_event_instances(now, settings)
        .into_iter()
        .take(settings.overlay_max_events)
        .collect()
}

fn generate_scheduled_events(sky_date: SkyDate) -> Vec<ScheduledEvent> {
    let mut events = Vec::new();
    let start_date = sky_date.add_days(-1);

    for day_offset in 0..=15 {
        let date = start_date.add_days(day_offset);

        push_daily_reset(&mut events, date);
        push_weekly_reset(&mut events, date);
        push_social_wax(&mut events, date);
        push_forest_rainbow(&mut events, date);
        push_shard_events(&mut events, date);
    }

    events
}

fn push_daily_reset(events: &mut Vec<ScheduledEvent>, date: SkyDate) {
    let Some(starts_at) = sky_wall_time_to_instant(date, 0, 0) else {
        return;
    };
    let ends_at = starts_at + Duration::minutes(5);

    events.push(ScheduledEvent {
        definition_id: "daily-reset",
        title: "Daily Reset".to_string(),
        category: "reset",
        phase_label: Some("New quests and candles".to_string()),
        ..make_schedule_times(starts_at, starts_at, Some(ends_at))
    });
}

fn push_weekly_reset(events: &mut Vec<ScheduledEvent>, date: SkyDate) {
    if date.day_of_week_temporal() != 7 {
        return;
    }

    let Some(starts_at) = sky_wall_time_to_instant(date, 0, 0) else {
        return;
    };
    let ends_at = starts_at + Duration::minutes(5);

    events.push(ScheduledEvent {
        definition_id: "eden-reset",
        title: "Eden Reset".to_string(),
        category: "weekly",
        phase_label: Some("Statues refresh".to_string()),
        ..make_schedule_times(starts_at, starts_at, Some(ends_at))
    });
}

fn push_social_wax(events: &mut Vec<ScheduledEvent>, date: SkyDate) {
    for hour in (0..24).step_by(2) {
        push_window(
            events,
            WindowInput {
                definition_id: "geyser",
                title: "Sanctuary Geyser",
                category: "wax",
                date,
                hour,
                minute: 0,
                prep_minutes: 5,
                duration_minutes: 15,
                phase_label: "Pollution erupts",
                active_phase_label: "Wax window",
                location: Some("Sanctuary Islands"),
            },
        );
        push_window(
            events,
            WindowInput {
                definition_id: "grandma",
                title: "Grandma Dinner",
                category: "wax",
                date,
                hour,
                minute: 30,
                prep_minutes: 5,
                duration_minutes: 15,
                phase_label: "Table opens",
                active_phase_label: "Dinner wax",
                location: Some("Elevated Clearing"),
            },
        );
        push_window(
            events,
            WindowInput {
                definition_id: "turtle",
                title: "Sunset Turtle",
                category: "wax",
                date,
                hour,
                minute: 45,
                prep_minutes: 5,
                duration_minutes: 15,
                phase_label: "Sunset begins",
                active_phase_label: "Turtle wax",
                location: Some("Sanctuary Islands"),
            },
        );
    }
}

fn push_forest_rainbow(events: &mut Vec<ScheduledEvent>, date: SkyDate) {
    for hour in [5, 17] {
        push_window(
            events,
            WindowInput {
                definition_id: "forest-rainbow",
                title: "Forest Rainbow",
                category: "wax",
                date,
                hour,
                minute: 0,
                prep_minutes: 0,
                duration_minutes: 60,
                phase_label: "Rainbow candle",
                active_phase_label: "Rainbow candle",
                location: Some("Forest Brook"),
            },
        );
    }
}

fn push_shard_events(events: &mut Vec<ScheduledEvent>, date: SkyDate) {
    for shard in get_shard_windows(date) {
        events.push(ScheduledEvent {
            definition_id: "shard-eruption",
            title: match shard.color {
                ShardColor::Red => "Red Shard",
                ShardColor::Black => "Black Shard",
            }
            .to_string(),
            category: "shard",
            phase_label: Some("Visible at gate".to_string()),
            active_phase_label: Some("Landed".to_string()),
            location: Some(format!("{} - {}", shard.realm, shard.location)),
            reward_label: Some(shard.reward_label),
            ..make_schedule_times(shard.gate_visible_at, shard.lands_at, Some(shard.clears_at))
        });
    }
}

#[derive(Debug, Clone, Copy)]
struct WindowInput {
    definition_id: &'static str,
    title: &'static str,
    category: &'static str,
    date: SkyDate,
    hour: u32,
    minute: u32,
    prep_minutes: i64,
    duration_minutes: i64,
    phase_label: &'static str,
    active_phase_label: &'static str,
    location: Option<&'static str>,
}

fn push_window(events: &mut Vec<ScheduledEvent>, input: WindowInput) {
    let Some(starts_at) = sky_wall_time_to_instant(input.date, input.hour, input.minute) else {
        return;
    };

    events.push(ScheduledEvent {
        definition_id: input.definition_id,
        title: input.title.to_string(),
        category: input.category,
        phase_label: Some(input.phase_label.to_string()),
        active_phase_label: Some(input.active_phase_label.to_string()),
        location: input.location.map(str::to_string),
        ..make_schedule_times(
            starts_at,
            starts_at + Duration::minutes(input.prep_minutes),
            Some(starts_at + Duration::minutes(input.duration_minutes)),
        )
    });
}

fn make_schedule_times(
    starts_at: DateTime<Utc>,
    active_at: DateTime<Utc>,
    ends_at: Option<DateTime<Utc>>,
) -> ScheduledEvent {
    ScheduledEvent {
        definition_id: "",
        title: String::new(),
        category: "",
        starts_at,
        starts_at_ms: starts_at.timestamp_millis(),
        starts_at_utc: utc_label(starts_at),
        active_at_ms: active_at.timestamp_millis(),
        ends_at_ms: ends_at.map(|value| value.timestamp_millis()),
        ends_at_utc: ends_at.map(utc_label),
        sky_time_label: format_sky_time(starts_at),
        phase_label: None,
        active_phase_label: None,
        location: None,
        reward_label: None,
    }
}

fn to_instance(
    event: ScheduledEvent,
    now_ms: i64,
    settings: &EventGenerationSettings,
) -> EventInstance {
    let definition = event_definition(event.definition_id);
    let status = status_for(
        now_ms,
        event.starts_at_ms,
        event.active_at_ms,
        event.ends_at_ms,
    );
    let target = if matches!(
        status,
        EventStatus::Active | EventStatus::EndingSoon | EventStatus::Preparing
    ) {
        event.ends_at_ms.unwrap_or(event.starts_at_ms)
    } else {
        event.starts_at_ms
    };

    EventInstance {
        definition_id: event.definition_id.to_string(),
        title: event.title,
        category: event.category.to_string(),
        status: status.as_str().to_string(),
        starts_at_utc: event.starts_at_utc,
        ends_at_utc: event.ends_at_utc,
        sky_time_label: event.sky_time_label,
        local_time_label: format_local_time(
            event.starts_at,
            &settings.time_format,
            settings.local_time_zone.as_deref(),
        ),
        countdown_ms: (target - now_ms).abs(),
        phase_label: if matches!(status, EventStatus::Active | EventStatus::EndingSoon) {
            event.active_phase_label.or(event.phase_label)
        } else {
            event.phase_label
        },
        location: event
            .location
            .or_else(|| definition.and_then(|value| value.location.map(str::to_string))),
        source: definition
            .map(|value| value.source)
            .unwrap_or("wiki")
            .to_string(),
        priority: definition.map(|value| value.priority).unwrap_or(0),
        reward_label: event.reward_label,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EventStatus {
    Upcoming,
    Preparing,
    Active,
    EndingSoon,
    Ended,
}

impl EventStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Upcoming => "upcoming",
            Self::Preparing => "preparing",
            Self::Active => "active",
            Self::EndingSoon => "endingSoon",
            Self::Ended => "ended",
        }
    }
}

fn status_for(
    now_ms: i64,
    starts_at_ms: i64,
    active_at_ms: i64,
    ends_at_ms: Option<i64>,
) -> EventStatus {
    if now_ms < starts_at_ms {
        return EventStatus::Upcoming;
    }

    if now_ms < active_at_ms {
        return EventStatus::Preparing;
    }

    let Some(ends_at_ms) = ends_at_ms else {
        return EventStatus::Ended;
    };

    if now_ms >= ends_at_ms {
        return EventStatus::Ended;
    }

    if ends_at_ms - now_ms <= 5 * 60_000 {
        EventStatus::EndingSoon
    } else {
        EventStatus::Active
    }
}

fn sort_events(a: &EventInstance, b: &EventInstance) -> Ordering {
    let state_delta = state_score(&a.status).cmp(&state_score(&b.status));
    if state_delta != Ordering::Equal {
        return state_delta;
    }

    if a.status == "upcoming" && b.status == "upcoming" {
        return a.countdown_ms.cmp(&b.countdown_ms);
    }

    let priority_delta = b.priority.cmp(&a.priority);
    if priority_delta != Ordering::Equal {
        return priority_delta;
    }

    a.countdown_ms.cmp(&b.countdown_ms)
}

fn state_score(status: &str) -> i32 {
    match status {
        "active" | "endingSoon" => 0,
        "preparing" => 1,
        "upcoming" => 2,
        _ => 3,
    }
}

fn event_definition(id: &str) -> Option<&'static EventDefinition> {
    EVENT_DEFINITIONS
        .iter()
        .find(|definition| definition.id == id)
}

fn utc_label(instant: DateTime<Utc>) -> String {
    instant.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

fn default_time_format() -> String {
    "system".to_string()
}

fn default_overlay_max_events() -> usize {
    5
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings() -> EventGenerationSettings {
        EventGenerationSettings {
            time_format: "24h".to_string(),
            local_time_zone: Some("UTC".to_string()),
            overlay_max_events: 5,
            ..EventGenerationSettings::default()
        }
    }

    fn parse_utc(value: &str) -> DateTime<Utc> {
        value.parse::<DateTime<Utc>>().unwrap()
    }

    #[test]
    fn geyser_is_every_even_sky_hour_with_a_fifteen_minute_duration() {
        let events = generate_event_instances(parse_utc("2026-01-15T08:00:00Z"), &settings());
        let geyser = events
            .iter()
            .find(|event| {
                event.definition_id == "geyser" && event.starts_at_utc == "2026-01-15T08:00:00Z"
            })
            .unwrap();

        assert_eq!(geyser.ends_at_utc.as_deref(), Some("2026-01-15T08:15:00Z"));
    }

    #[test]
    fn grandma_runs_from_thirty_to_forty_five() {
        let events = generate_event_instances(parse_utc("2026-01-15T08:00:00Z"), &settings());
        let grandma = events
            .iter()
            .find(|event| {
                event.definition_id == "grandma" && event.starts_at_utc == "2026-01-15T08:30:00Z"
            })
            .unwrap();

        assert_eq!(grandma.ends_at_utc.as_deref(), Some("2026-01-15T08:45:00Z"));
    }

    #[test]
    fn turtle_pre_event_starts_at_forty_five_and_active_window_ends_at_the_hour() {
        let events = generate_event_instances(parse_utc("2026-01-15T08:00:00Z"), &settings());
        let turtle = events
            .iter()
            .find(|event| {
                event.definition_id == "turtle" && event.starts_at_utc == "2026-01-15T08:45:00Z"
            })
            .unwrap();

        assert_eq!(turtle.ends_at_utc.as_deref(), Some("2026-01-15T09:00:00Z"));
    }

    #[test]
    fn weekly_reset_is_generated_on_sunday_sky_time() {
        let events = generate_event_instances(parse_utc("2026-01-18T08:00:00Z"), &settings());

        assert!(events.iter().any(|event| {
            event.definition_id == "eden-reset" && event.starts_at_utc == "2026-01-18T08:00:00Z"
        }));
    }

    #[test]
    fn dst_fall_back_does_not_duplicate_even_hour_geyser_events() {
        let events = generate_event_instances(parse_utc("2026-11-01T07:00:00Z"), &settings());
        let starts = events
            .iter()
            .filter(|event| {
                event.definition_id == "geyser" && event.starts_at_utc.starts_with("2026-11-01")
            })
            .map(|event| event.starts_at_utc.clone())
            .collect::<Vec<_>>();
        let unique = starts.iter().collect::<std::collections::HashSet<_>>();

        assert_eq!(unique.len(), starts.len());
    }

    #[test]
    fn overlay_events_are_limited_by_settings() {
        let mut settings = settings();
        settings.overlay_max_events = 3;

        assert_eq!(
            get_overlay_events(parse_utc("2026-01-15T08:00:00Z"), &settings).len(),
            3
        );
    }
}
