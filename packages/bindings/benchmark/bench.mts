/**
 * Performance benchmark suite for the takeoff bindings (RFC-006).
 * Run via: pnpm bench (from repo root or packages/bindings).
 *
 * Workloads:
 * 1. Large polygon set: state creation and reading area/length for many polygons.
 * 2. Many groups: state creation and requesting aggregates for all groups.
 * 3. Round-trip: create scale/group/measurement from JS and get group aggregate (FFI overhead).
 */

import { Bench } from "tinybench";
import type {
	Group,
	Measurement,
	Page,
	Scale,
	StateOptions,
} from "../index.js";
import { TakeoffStateHandler } from "../index.js";

// --- Deterministic data helpers (no faker; fast and CI-stable) -----------------

const PAGE_ID = "page-1";
const SCALE_ID = "scale-1";
const DEFAULT_SCALE: Scale = {
	type: "Default",
	id: SCALE_ID,
	pageId: PAGE_ID,
	scale: { pixelDistance: 1, realDistance: 1, unit: "Meters" },
};

function createPage(): Page {
	return { id: PAGE_ID, name: "Page 1" };
}

function createScale(): Scale {
	return { ...DEFAULT_SCALE };
}

function createPolygonPoints(): Array<{ x: number; y: number }> {
	return [
		{ x: 0, y: 0 },
		{ x: 100, y: 0 },
		{ x: 100, y: 100 },
		{ x: 0, y: 100 },
	];
}

function createPolygonMeasurement(
	id: string,
	groupId: string,
	points: Array<{ x: number; y: number }> = createPolygonPoints(),
): Measurement {
	return {
		type: "Polygon",
		id,
		pageId: PAGE_ID,
		groupId,
		points,
	};
}

/** Build StateOptions with one page, one scale, one group, and N polygon measurements. */
function buildLargePolygonSetOptions(
	polygonCount: number,
	groupId: string = "group-1",
): StateOptions {
	const page = createPage();
	const scale = createScale();
	const group: Group = {
		id: groupId,
		name: "Group",
		measurementType: "Area",
	};
	const measurements: Measurement[] = [];
	for (let i = 0; i < polygonCount; i++) {
		measurements.push(createPolygonMeasurement(`m-${groupId}-${i}`, groupId));
	}
	return {
		pages: [page],
		scales: [scale],
		groups: [group],
		measurements,
	};
}

/** Build StateOptions with many groups, each with several measurements. */
function buildManyGroupsOptions(
	groupCount: number,
	measurementsPerGroup: number,
): StateOptions {
	const page = createPage();
	const scale = createScale();
	const groups: Group[] = [];
	const measurements: Measurement[] = [];
	for (let g = 0; g < groupCount; g++) {
		const groupId = `group-${g}`;
		groups.push({
			id: groupId,
			name: `Group ${g}`,
			measurementType: "Area",
		});
		for (let m = 0; m < measurementsPerGroup; m++) {
			measurements.push(createPolygonMeasurement(`m-${groupId}-${m}`, groupId));
		}
	}
	return {
		pages: [page],
		scales: [scale],
		groups,
		measurements,
	};
}

// --- Workload sizes (tuned for CI: complete in a few minutes; use larger locally if desired) ---

const LARGE_POLYGON_COUNT = 500;
const MANY_GROUPS_COUNT = 50;
const MEASUREMENTS_PER_GROUP = 20;

// --- Benchmarks --------------------------------------------------------------------------------

const bench = new Bench({ time: 500, iterations: 20 });

// 1) Large polygon set
bench.add("State creation (large polygon set)", () => {
	new TakeoffStateHandler(buildLargePolygonSetOptions(LARGE_POLYGON_COUNT));
});

let stateLargePolygons: TakeoffStateHandler;
bench.add("Read area/length (large polygon set)", () => {
	if (!stateLargePolygons) {
		stateLargePolygons = new TakeoffStateHandler(
			buildLargePolygonSetOptions(LARGE_POLYGON_COUNT),
		);
	}
	const measurements = stateLargePolygons.getMeasurementsByGroupId("group-1");
	for (const m of measurements) {
		m.area;
		m.length;
	}
});

// 2) Many groups
bench.add("State creation (many groups)", () => {
	new TakeoffStateHandler(
		buildManyGroupsOptions(MANY_GROUPS_COUNT, MEASUREMENTS_PER_GROUP),
	);
});

let stateManyGroups: TakeoffStateHandler;
const manyGroupIds: string[] = Array.from(
	{ length: MANY_GROUPS_COUNT },
	(_, i) => `group-${i}`,
);
bench.add("Group aggregates (many groups)", () => {
	if (!stateManyGroups) {
		stateManyGroups = new TakeoffStateHandler(
			buildManyGroupsOptions(MANY_GROUPS_COUNT, MEASUREMENTS_PER_GROUP),
		);
	}
	for (const gid of manyGroupIds) {
		const g = stateManyGroups.getGroup(gid);
		if (g) {
			g.area;
			g.length;
		}
	}
});

// 3) Round-trip (create from JS, get aggregate)
bench.add("Round-trip: scale + group + measurement → group aggregate", () => {
	const state = new TakeoffStateHandler();
	state.upsertPage(createPage());
	state.upsertScale(createScale());
	state.upsertGroup({
		id: "g1",
		name: "G1",
		measurementType: "Area",
	});
	state.upsertMeasurement(createPolygonMeasurement("m1", "g1"));
	const g = state.getGroup("g1");
	if (g) {
		g.area;
		g.length;
	}
});

// --- Sanity check: one known result (optional per RFC) ----------------------------------------

function sanityCheck(): void {
	const opts = buildLargePolygonSetOptions(1);
	const state = new TakeoffStateHandler(opts);
	const measurements = state.getMeasurementsByGroupId("group-1");
	if (measurements.length !== 1) {
		throw new Error(
			`Sanity: expected 1 measurement, got ${measurements.length}`,
		);
	}
	const first = measurements[0];
	const area = first ? first.area : null;
	if (area == null) {
		throw new Error("Sanity: expected area to be computed");
	}
	// 100x100 px with scale 1:1 Meters => area in m² should be 10000
	const converted = area.getConvertedValue("Meters");
	if (typeof converted !== "number" || converted <= 0) {
		throw new Error(`Sanity: expected positive area, got ${converted}`);
	}
}

sanityCheck();

// --- Run and report ----------------------------------------------------------------------------

await bench.run();
console.table(bench.table());
