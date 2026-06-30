import { invoke } from "@tauri-apps/api/core";
import type {
  SkyActiveRoute,
  SkyAreaRoute,
  SkyAreaSummary,
  SkyCalendarEntry,
  SkyCalendarEntryKind,
  SkyCandleRun,
  SkyCandleRunSummary,
  SkyDataBundle,
  SkyItemSummary,
  SkyMiniMapPin,
  SkyRealmRoute,
  SkyRealmSummary,
  SkyRouteFilters,
  SkyRouteProgress,
  SkyRouteTarget,
} from "@/data/skygame";
import { isTauriRuntime } from "@/tauri/overlay";

export interface SkyActiveRouteTargetState {
  target: SkyRouteTarget;
  targetIndex: number;
  targets: SkyRouteTarget[];
  completed: boolean;
  total: number;
  completedCount: number;
}

export interface SkyCalendarQuery {
  startDate: string;
  endDate: string;
  kinds?: SkyCalendarEntryKind[];
}

export interface SkyItemSearchQuery {
  query: string;
  types?: string[];
  wishlist?: Record<string, boolean>;
}

async function fallbackIndex() {
  return (await import("@/data/skygame")).skyDataIndex;
}

export async function getSkyGameMeta(): Promise<SkyDataBundle["meta"]> {
  if (!isTauriRuntime()) {
    return (await fallbackIndex()).getMeta();
  }

  return invoke<SkyDataBundle["meta"]>("skygame_get_meta");
}

export async function getSkyGameStats(): Promise<SkyDataBundle["stats"]> {
  if (!isTauriRuntime()) {
    return (await fallbackIndex()).getStats();
  }

  return invoke<SkyDataBundle["stats"]>("skygame_get_stats");
}

export async function getSkyGameSourceStats(): Promise<SkyDataBundle["sourceStats"]> {
  if (!isTauriRuntime()) {
    return (await fallbackIndex()).getSourceStats();
  }

  return invoke<SkyDataBundle["sourceStats"]>("skygame_get_source_stats");
}

export async function getSkyGameSourceGroups(): Promise<SkyDataBundle["sourceGroups"]> {
  if (!isTauriRuntime()) {
    return (await fallbackIndex()).getSourceGroups();
  }

  return invoke<SkyDataBundle["sourceGroups"]>("skygame_get_source_groups");
}

export async function getSkyCandleRuns(): Promise<SkyCandleRunSummary[]> {
  if (!isTauriRuntime()) {
    return (await fallbackIndex()).getCandleRuns();
  }

  return invoke<SkyCandleRunSummary[]>("skygame_get_candle_runs");
}

export async function getSkyCandleRun(guid: string): Promise<SkyCandleRun | null> {
  if (!isTauriRuntime()) {
    return (await fallbackIndex()).getCandleRun(guid);
  }

  return invoke<SkyCandleRun | null>("skygame_get_candle_run", { guid });
}

export async function getSkyRealms(): Promise<SkyRealmSummary[]> {
  if (!isTauriRuntime()) {
    return (await fallbackIndex()).getRealms();
  }

  return invoke<SkyRealmSummary[]>("skygame_get_realms");
}

export async function getSkyRealm(guid: string): Promise<SkyRealmSummary | null> {
  if (!isTauriRuntime()) {
    return (await fallbackIndex()).getRealm(guid);
  }

  return invoke<SkyRealmSummary | null>("skygame_get_realm", { guid });
}

export async function getSkyArea(guid: string): Promise<SkyAreaSummary | null> {
  if (!isTauriRuntime()) {
    return (await fallbackIndex()).getArea(guid);
  }

  return invoke<SkyAreaSummary | null>("skygame_get_area", { guid });
}

export async function getSkyAreasForRealm(realmGuid: string): Promise<SkyAreaSummary[]> {
  if (!isTauriRuntime()) {
    return (await fallbackIndex()).getAreasForRealm(realmGuid);
  }

  return invoke<SkyAreaSummary[]>("skygame_get_areas_for_realm", { realmGuid });
}

export async function getSkyCalendarEntries(
  query: SkyCalendarQuery,
): Promise<SkyCalendarEntry[]> {
  if (!isTauriRuntime()) {
    return (await fallbackIndex()).getCalendarEntries(query);
  }

  return invoke<SkyCalendarEntry[]>("skygame_get_calendar_entries", { query });
}

export async function searchSkyItems(
  query: SkyItemSearchQuery,
): Promise<SkyItemSummary[]> {
  if (!isTauriRuntime()) {
    return (await fallbackIndex()).searchItems(query.query, {
      types: query.types,
      wishlist: query.wishlist,
    });
  }

  return invoke<SkyItemSummary[]>("skygame_search_items", { query });
}

export async function getSkyItemDetail(guid: string): Promise<SkyItemSummary | null> {
  if (!isTauriRuntime()) {
    return (await fallbackIndex()).getItemDetail(guid);
  }

  return invoke<SkyItemSummary | null>("skygame_get_item_detail", { guid });
}

export async function getUpcomingSkySeasonalEntries(
  now: Date,
): Promise<SkyCalendarEntry[]> {
  const today = now.toISOString().slice(0, 10);
  return getSkyCalendarEntries({
    startDate: today,
    endDate: addDaysIso(today, 90),
    kinds: ["event", "season"],
  }).then((entries) => entries.sort((a, b) => a.date.localeCompare(b.date)));
}

export async function getTravelingSkySpiritEntries(
  range: Pick<SkyCalendarQuery, "startDate" | "endDate">,
): Promise<SkyCalendarEntry[]> {
  return getSkyCalendarEntries({
    ...range,
    kinds: ["traveling-spirit"],
  });
}

export async function getSkyRealmRoute(
  realmGuid: string,
): Promise<SkyRealmRoute | null> {
  if (!isTauriRuntime()) {
    return (await fallbackIndex()).getRealmRoute(realmGuid);
  }

  return invoke<SkyRealmRoute | null>("skygame_get_realm_route", { realmGuid });
}

export async function getSkyAreaRoute(areaGuid: string): Promise<SkyAreaRoute | null> {
  if (!isTauriRuntime()) {
    return (await fallbackIndex()).getAreaRoute(areaGuid);
  }

  return invoke<SkyAreaRoute | null>("skygame_get_area_route", { areaGuid });
}

export async function getSkyRouteTargets(
  areaGuid: string,
  filters: SkyRouteFilters = {},
): Promise<SkyRouteTarget[]> {
  if (!isTauriRuntime()) {
    return (await fallbackIndex()).getRouteTargets(areaGuid, filters);
  }

  return invoke<SkyRouteTarget[]>("skygame_get_route_targets", {
    areaGuid,
    filters,
  });
}

export async function getSkyRouteTarget(guid: string): Promise<SkyRouteTarget | null> {
  if (!isTauriRuntime()) {
    return (await fallbackIndex()).getRouteTarget(guid);
  }

  return invoke<SkyRouteTarget | null>("skygame_get_route_target", { guid });
}

export async function getSkyMiniMapPins(
  areaGuid: string,
  filters: SkyRouteFilters = {},
): Promise<SkyMiniMapPin[]> {
  if (!isTauriRuntime()) {
    return (await fallbackIndex()).getMiniMapPins(areaGuid, filters);
  }

  return invoke<SkyMiniMapPin[]>("skygame_get_mini_map_pins", {
    areaGuid,
    filters,
  });
}

export async function getSkyActiveRouteTarget(
  activeRoute: SkyActiveRoute | null | undefined,
  progress: SkyRouteProgress | null | undefined,
): Promise<SkyActiveRouteTargetState | null> {
  if (!isTauriRuntime()) {
    return (await fallbackIndex()).getActiveRouteTarget(activeRoute, progress);
  }

  return invoke<SkyActiveRouteTargetState | null>("skygame_get_active_route_target", {
    activeRoute,
    progress,
  });
}

function addDaysIso(date: string, days: number) {
  const parsed = new Date(`${date}T00:00:00.000Z`);
  parsed.setUTCDate(parsed.getUTCDate() + days);
  return parsed.toISOString().slice(0, 10);
}
