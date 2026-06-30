import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { SkyDataResolver } from "skygame-data";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, "..");
const packagePath = path.join(root, "node_modules", "skygame-data", "package.json");
const sourcePath = path.join(
  root,
  "node_modules",
  "skygame-data",
  "assets",
  "everything.json",
);
const outPaths = [
  path.join(root, "src-ui", "data", "skygame", "selected-data.json"),
  path.join(root, "src-rs", "src", "data", "skygame", "selected-data.json"),
];

const sourcePackage = JSON.parse(fs.readFileSync(packagePath, "utf8"));
const rawData = JSON.parse(fs.readFileSync(sourcePath, "utf8"));
const resolvedData = SkyDataResolver.resolve(JSON.parse(JSON.stringify(rawData)));

const sourceGroupMap = {
  realms: "realms",
  areas: "areas",
  wingedLights: "wingedLights",
  mapShrines: "mapShrines",
  constellations: "constellations",
  seasons: "seasons",
  events: "events",
  eventInstances: "eventInstances",
  eventInstanceSpirits: "eventInstanceSpirits",
  spirits: "spirits",
  spiritTrees: "spiritTrees",
  spiritTreeTiers: "spiritTreeTiers",
  nodes: "nodes",
  travelingSpirits: "travelingSpirits",
  specialVisits: "specialVisits",
  specialVisitSpirits: "specialVisitSpirits",
  items: "items",
  itemLists: "itemLists",
  shops: "shops",
  iaps: "iaps",
  candles: "candles",
};

const sourceGroups = Object.fromEntries(
  Object.entries(sourceGroupMap).map(([key, sourceKey]) => [
    key,
    rawData[sourceKey]?.items ?? [],
  ]),
);

const sourceStats = Object.fromEntries(
  Object.entries(sourceGroups).map(([key, items]) => [key, items.length]),
);

const realms = resolvedData.realms.items.map((realm) => ({
  guid: realm.guid,
  name: realm.name,
  shortName: realm.shortName,
  imageUrl: realm.imageUrl,
  hidden: realm.hidden === true,
  areaGuids: realm.areas?.map((area) => area.guid) ?? [],
  mapPosition: point(realm.mapData?.position),
  wikiUrl: realm._wiki?.href,
}));

const areas = resolvedData.areas.items.map((area) => ({
  guid: area.guid,
  name: area.name,
  realmGuid: area.realm?.guid,
  realmName: area.realm?.name,
  imageUrl: area.imageUrl,
  imagePosition: area.imagePosition,
  mapPosition: point(area.mapData?.position),
  connectionGuids: area.connections?.map((connection) => connection.area.guid) ?? [],
  spiritGuids: area.spirits?.map((spirit) => spirit.guid) ?? [],
  wingedLightGuids: area.wingedLights?.map((wingedLight) => wingedLight.guid) ?? [],
  mapShrineGuids: area.mapShrines?.map((mapShrine) => mapShrine.guid) ?? [],
  wikiUrl: area._wiki?.href,
}));

const spiritSummaries = resolvedData.spirits.items.map((spirit) => ({
  guid: spirit.guid,
  name: spirit.name,
  type: spirit.type,
  imageUrl: spirit.imageUrl,
  areaGuid: spirit.area?.guid,
  areaName: spirit.area?.name,
  realmGuid: spirit.area?.realm?.guid,
  realmName: spirit.area?.realm?.name,
  seasonGuid: spirit.season?.guid,
  treeGuid: spirit.tree?.guid,
  wikiUrl: spirit._wiki?.href,
}));

const wingedLightSummaries = resolvedData.wingedLights.items.map((wingedLight, index) => ({
  guid: wingedLight.guid,
  order: wingedLight.order ?? index,
  name: wingedLight.name,
  description: wingedLight.description,
  areaGuid: wingedLight.area?.guid,
  areaName: wingedLight.area?.name,
  realmGuid: wingedLight.area?.realm?.guid,
  realmName: wingedLight.area?.realm?.name,
  mapPosition: point(wingedLight.mapData?.position),
  videoUrl: wingedLight.mapData?.videoUrl,
  wikiUrl: wingedLight._wiki?.href,
}));

const routeTargets = [
  ...spiritSummaries
    .filter((spirit) => spirit.areaGuid && spirit.realmGuid)
    .map((spirit, index) => ({
      guid: `spirit:${spirit.guid}`,
      sourceGuid: spirit.guid,
      kind: "spirit",
      name: spirit.name,
      description: spirit.type ? `${spirit.type} spirit` : "Spirit",
      realmGuid: spirit.realmGuid,
      realmName: spirit.realmName,
      areaGuid: spirit.areaGuid,
      areaName: spirit.areaName,
      imageUrl: spirit.imageUrl,
      order: index,
    })),
  ...wingedLightSummaries
    .filter((wingedLight) => wingedLight.areaGuid && wingedLight.realmGuid)
    .map((wingedLight, index) => ({
      guid: `winged-light:${wingedLight.guid}`,
      sourceGuid: wingedLight.guid,
      kind: "winged-light",
      name: wingedLight.name || `Winged Light ${index + 1}`,
      description: wingedLight.description || "Winged light",
      realmGuid: wingedLight.realmGuid,
      realmName: wingedLight.realmName,
      areaGuid: wingedLight.areaGuid,
      areaName: wingedLight.areaName,
      mapPosition: wingedLight.mapPosition,
      videoUrl: wingedLight.videoUrl,
      order: wingedLight.order ?? index,
    })),
];

const routeTargetsByArea = groupBy(routeTargets, "areaGuid");
const miniMapPins = buildMiniMapPins(areas, routeTargetsByArea);
const miniMapPinsByArea = groupBy(miniMapPins, "areaGuid");

const areaRoutes = areas.map((area) => {
  const targetGuids = (routeTargetsByArea.get(area.guid) ?? []).map(
    (target) => target.guid,
  );

  return {
    guid: area.guid,
    name: area.name,
    realmGuid: area.realmGuid,
    realmName: area.realmName,
    imageUrl: area.imageUrl,
    mapPosition: area.mapPosition,
    connectionGuids: area.connectionGuids,
    targetGuids,
    counts: countTargetKinds(targetGuids, routeTargets),
  };
});

const realmRoutes = realms.map((realm) => {
  const targetGuids = routeTargets
    .filter((target) => target.realmGuid === realm.guid)
    .map((target) => target.guid);

  return {
    guid: realm.guid,
    name: realm.name,
    shortName: realm.shortName,
    imageUrl: realm.imageUrl,
    areaGuids: realm.areaGuids,
    targetGuids,
    counts: countTargetKinds(targetGuids, routeTargets),
  };
});

const calendarEntries = [
  ...resolvedData.seasons.items.map((season) => ({
    guid: season.guid,
    kind: "season",
    name: season.name,
    shortName: season.shortName,
    date: isoDate(season.date),
    endDate: isoDate(season.endDate),
    imageUrl: season.iconUrl,
    wikiUrl: season._wiki?.href,
  })),
  ...resolvedData.eventInstances.items.map((instance) => ({
    guid: instance.guid,
    kind: "event",
    name: instance.name || instance.event?.name,
    shortName: instance.shortName || instance.event?.shortName,
    date: isoDate(instance.date),
    endDate: isoDate(instance.endDate),
    imageUrl: instance.event?.imageUrl,
    wikiUrl: instance.event?._wiki?.href,
    calendarUrl: instance._calendar?.href,
  })),
  ...resolvedData.travelingSpirits.items.map((travelingSpirit) => ({
    guid: travelingSpirit.guid,
    kind: "traveling-spirit",
    name: travelingSpirit.spirit?.name ?? "Traveling Spirit",
    date: isoDate(travelingSpirit.date),
    endDate: isoDate(addDays(travelingSpirit.date, 3)),
    imageUrl: travelingSpirit.spirit?.imageUrl,
    wikiUrl: travelingSpirit.spirit?._wiki?.href,
    spiritGuid: travelingSpirit.spirit?.guid,
  })),
].filter((entry) => entry.name && entry.date && entry.endDate);

const items = resolvedData.items.items.map((item) => ({
  guid: item.guid,
  id: item.id,
  name: item.name,
  type: item.type,
  icon: item.icon,
  previewUrl: item.previewUrl,
  origins: itemOrigins(item),
}));

const bundle = {
  meta: {
    source: "skygame-data",
    version: sourcePackage.version,
    generatedAt: new Date().toISOString(),
    license: sourcePackage.license,
    sourceUrl: "https://github.com/Silverfeelin/SkyGame-Data",
  },
  sourceStats,
  sourceGroups,
  calendarEntries: calendarEntries.sort((a, b) => a.date.localeCompare(b.date)),
  items,
  realms,
  areas,
  spirits: spiritSummaries,
  wingedLights: wingedLightSummaries,
  routes: {
    realms: realmRoutes,
    areas: areaRoutes,
    targets: routeTargets.sort(sortRouteTargets),
    pins: miniMapPins.sort(sortPins),
  },
  stats: {
    events: sourceStats.events,
    eventInstances: sourceStats.eventInstances,
    seasons: sourceStats.seasons,
    travelingSpirits: sourceStats.travelingSpirits,
    items: sourceStats.items,
    realms: sourceStats.realms,
    areas: sourceStats.areas,
    spirits: sourceStats.spirits,
    wingedLights: sourceStats.wingedLights,
    mapShrines: sourceStats.mapShrines,
  },
};

for (const outPath of outPaths) {
  fs.mkdirSync(path.dirname(outPath), { recursive: true });
  fs.writeFileSync(outPath, `${JSON.stringify(bundle, null, 2)}\n`);
}

console.log(`Generated ${outPaths.map((outPath) => path.relative(root, outPath)).join(", ")}`);
console.log(JSON.stringify(bundle.stats, null, 2));

function point(value) {
  if (!Array.isArray(value) || value.length !== 2) {
    return undefined;
  }

  const [x, y] = value;
  return typeof x === "number" && typeof y === "number" ? [x, y] : undefined;
}

function groupBy(items, key) {
  const grouped = new Map();

  for (const item of items) {
    const value = item[key];
    if (!value) {
      continue;
    }

    grouped.set(value, [...(grouped.get(value) ?? []), item]);
  }

  return grouped;
}

function countTargetKinds(targetGuids, targets) {
  const targetSet = new Set(targetGuids);
  const counts = { spirits: 0, wingedLights: 0, total: 0 };

  for (const target of targets) {
    if (!targetSet.has(target.guid)) {
      continue;
    }

    counts.total += 1;
    if (target.kind === "spirit") {
      counts.spirits += 1;
    }
    if (target.kind === "winged-light") {
      counts.wingedLights += 1;
    }
  }

  return counts;
}

function buildMiniMapPins(areaSummaries, targetsByArea) {
  const pins = [];

  for (const area of areaSummaries) {
    const targets = targetsByArea.get(area.guid) ?? [];
    const positionedTargets = targets.filter((target) => target.mapPosition);
    const positions = positionedTargets.map((target) => target.mapPosition);
    const bounds = boundsForPositions(positions, area.mapPosition);
    const spiritTargets = targets.filter((target) => target.kind === "spirit");

    for (const target of targets) {
      const position =
        target.mapPosition ??
        syntheticSpiritPosition(area.mapPosition, spiritTargets.indexOf(target), spiritTargets.length);

      if (!position || !bounds) {
        continue;
      }

      const [x, y] = project(position, bounds);
      pins.push({
        guid: `pin:${target.guid}`,
        targetGuid: target.guid,
        sourceGuid: target.sourceGuid,
        kind: target.kind,
        label: target.name,
        areaGuid: area.guid,
        realmGuid: area.realmGuid,
        x,
        y,
      });
    }
  }

  return pins;
}

function boundsForPositions(positions, fallbackCenter) {
  const useful = positions.length > 0 ? positions : fallbackCenter ? [fallbackCenter] : [];
  if (useful.length === 0) {
    return null;
  }

  const xs = useful.map(([x]) => x);
  const ys = useful.map(([, y]) => y);
  let minX = Math.min(...xs);
  let maxX = Math.max(...xs);
  let minY = Math.min(...ys);
  let maxY = Math.max(...ys);
  const xPad = Math.max(2, (maxX - minX) * 0.25);
  const yPad = Math.max(2, (maxY - minY) * 0.25);

  minX -= xPad;
  maxX += xPad;
  minY -= yPad;
  maxY += yPad;

  return { minX, maxX, minY, maxY };
}

function syntheticSpiritPosition(center, index, total) {
  if (!center || index < 0) {
    return undefined;
  }

  const angle = (Math.PI * 2 * index) / Math.max(1, total);
  const radius = total <= 1 ? 0 : 1.6;
  return [center[0] + Math.cos(angle) * radius, center[1] + Math.sin(angle) * radius];
}

function project(position, bounds) {
  const width = Math.max(0.001, bounds.maxX - bounds.minX);
  const height = Math.max(0.001, bounds.maxY - bounds.minY);
  const x = 8 + ((position[0] - bounds.minX) / width) * 84;
  const y = 8 + ((position[1] - bounds.minY) / height) * 84;
  return [roundPercent(x), roundPercent(y)];
}

function roundPercent(value) {
  return Math.round(Math.max(4, Math.min(96, value)) * 100) / 100;
}

function isoDate(value) {
  if (!value) {
    return "";
  }

  if (typeof value === "string") {
    return value.slice(0, 10);
  }

  if (value instanceof Date) {
    return value.toISOString().slice(0, 10);
  }

  if (typeof value.toISODate === "function") {
    return value.toISODate();
  }

  if (typeof value.toISO === "function") {
    return value.toISO().slice(0, 10);
  }

  return String(value).slice(0, 10);
}

function addDays(value, days) {
  const date = new Date(`${isoDate(value)}T00:00:00.000Z`);
  date.setUTCDate(date.getUTCDate() + days);
  return date;
}

function itemOrigins(item) {
  const origins = [];
  const seen = new Set();

  const add = (origin) => {
    const key = `${origin.kind}:${origin.name}:${origin.date ?? ""}`;
    if (!origin.name || seen.has(key)) {
      return;
    }

    seen.add(key);
    origins.push(origin);
  };

  if (item.season) {
    add({
      kind: "season",
      name: item.season.name,
      date: isoDate(item.season.date),
      endDate: isoDate(item.season.endDate),
    });
  }

  for (const node of [...(item.nodes ?? []), ...(item.hiddenNodes ?? [])]) {
    const tree = node.tree ?? node.root?.tree;
    const spirit = tree?.spirit;
    const travelingSpirit = tree?.travelingSpirit;
    const specialVisit = tree?.specialVisitSpirit?.specialVisit;
    const eventInstance = tree?.eventInstanceSpirit?.eventInstance;

    if (spirit) {
      add({ kind: "spirit", name: spirit.name });
    } else if (travelingSpirit?.spirit) {
      add({
        kind: "traveling-spirit",
        name: travelingSpirit.spirit.name,
        date: isoDate(travelingSpirit.date),
      });
    } else if (specialVisit) {
      add({
        kind: "returning-spirit",
        name: specialVisit.name,
        date: isoDate(specialVisit.date),
        endDate: isoDate(specialVisit.endDate),
      });
    } else if (eventInstance) {
      add({
        kind: "event",
        name: eventInstance.name || eventInstance.event?.name,
        date: isoDate(eventInstance.date),
        endDate: isoDate(eventInstance.endDate),
      });
    } else {
      add({ kind: "node", name: tree?.name ?? "Spirit tree" });
    }
  }

  for (const iap of item.iaps ?? []) {
    add({ kind: "iap", name: iap.name });
  }

  for (const listNode of item.listNodes ?? []) {
    add({ kind: "item-list", name: listNode.list?.name ?? "Item list" });
  }

  return origins.length > 0 ? origins : [{ kind: "unknown", name: "Unknown" }];
}

function sortRouteTargets(a, b) {
  return (
    String(a.realmName).localeCompare(String(b.realmName)) ||
    String(a.areaName).localeCompare(String(b.areaName)) ||
    kindSort(a.kind) - kindSort(b.kind) ||
    a.order - b.order ||
    a.name.localeCompare(b.name)
  );
}

function sortPins(a, b) {
  return (
    String(a.realmGuid).localeCompare(String(b.realmGuid)) ||
    String(a.areaGuid).localeCompare(String(b.areaGuid)) ||
    kindSort(a.kind) - kindSort(b.kind) ||
    a.label.localeCompare(b.label)
  );
}

function kindSort(kind) {
  return kind === "winged-light" ? 0 : 1;
}
