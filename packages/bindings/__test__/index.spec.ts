/** biome-ignore-all lint/style/noNonNullAssertion: Needed for efficient testing */
import { describe, expect, test } from "vitest";
import {
	type Measurement,
	plus100,
	type Scale,
	TakeoffStateHandler,
	type Unit,
} from "../index.js";
import baseline from "../test_data/baseline.json";

test("sync function from native code", () => {
	const fixture = 42;
	expect(plus100(fixture)).toEqual(fixture + 100);
});

describe("TakeoffStateHandler", () => {
	test("should get measurement scale", () => {
		const state = new TakeoffStateHandler({
			pages: [],
			groups: [],
			measurements: [
				{
					id: "1",
					type: "Rectangle",
					pageId: "1",
					groupId: "1",
					points: [
						{ x: 0, y: 0 },
						{ x: 1, y: 1 },
					],
				},
			],
			scales: [
				{
					id: "1",
					type: "Default",
					pageId: "1",
					scale: {
						pixelDistance: 1,
						realDistance: 1,
						unit: "Feet",
					},
				},
			],
		});
		const scale = state.getMeasurementScale("1");
		expect(scale).toEqual({
			id: "1",
			type: "Default",
			pageId: "1",
			scale: {
				pixelDistance: 1,
				realDistance: 1,
				unit: "Feet",
			},
		});
	});
	test("should get measurement length", () => {
		const state = new TakeoffStateHandler({
			pages: [],
			groups: [],
			measurements: [
				{
					id: "1",
					type: "Polyline",
					pageId: "1",
					groupId: "1",
					points: [
						{ x: 0, y: 0 },
						{ x: 0, y: 1 },
					],
				},
			],
			scales: [
				{
					id: "1",
					type: "Default",
					pageId: "1",
					scale: {
						pixelDistance: 1,
						realDistance: 1,
						unit: "Feet",
					},
				},
			],
		});
		const measurement = state.getMeasurement("1");
		expect(measurement?.rawPerimeter).toEqual(1);
		const length = measurement?.length;
		console.log(length?.display("Feet"));
		expect(length?.display("Feet")).toEqual("1 ft");
	});
	test("should retain scale when updating measurement", () => {
		const state = new TakeoffStateHandler({
			pages: [],
			groups: [],
			measurements: [
				{
					id: "1",
					type: "Polyline",
					pageId: "1",
					groupId: "1",
					points: [
						{ x: 0, y: 0 },
						{ x: 0, y: 1 },
					],
				},
			],
			scales: [
				{
					id: "1",
					type: "Default",
					pageId: "1",
					scale: {
						pixelDistance: 1,
						realDistance: 1,
						unit: "Feet",
					},
				},
			],
		});
		const measurement = state.getMeasurement("1");
		expect(measurement?.scale?.id).toEqual("1");
		expect(measurement?.rawPerimeter).toEqual(1);

		expect(measurement?.length?.display("Feet")).toEqual("1 ft");
		state.upsertMeasurement({
			id: "1",
			type: "Polyline",
			pageId: "1",
			groupId: "1",
			points: [
				{ x: 0, y: 0 },
				{ x: 0, y: 1 },
			],
		});
		const updatedMeasurement = state.getMeasurement("1");
		expect(updatedMeasurement?.scale?.id).toEqual("1");
		expect(measurement?.rawPerimeter).toEqual(1);

		expect(measurement?.length?.display("Feet")).toEqual("1 ft");
	});
});

/**
 * RFC-007: Accuracy validation – bindings smoke tests.
 * Run a subset of baseline cases through NAPI and assert results match core (same as golden baseline).
 */
const RELATIVE_TOLERANCE = 0.0001; // 0.01%
const ABSOLUTE_EPSILON = 1e-8;

function withinTolerance(actual: number, expected: number): boolean {
	const diff = Math.abs(actual - expected);
	if (Math.abs(expected) < 1e-9) return diff <= ABSOLUTE_EPSILON;
	return (
		diff <= ABSOLUTE_EPSILON || diff / Math.abs(expected) <= RELATIVE_TOLERANCE
	);
}

const BASELINE_DATA: [
	string,
	{
		measurement: Measurement;
		scale: Scale;
		outputUnit: Unit;
		expected: { area?: number; length?: number; count?: number };
	},
][] = baseline.map((entry) => {
	const measurement = {
		id: entry.id,
		type: entry.kind,
		pageId: "1",
		groupId: "1",
		points: entry.points,
	} as Measurement;
	const scale: Scale = {
		id: "1",
		type: "Default",
		pageId: "1",
		scale: {
			pixelDistance: entry.scale.pixel_distance,
			realDistance: entry.scale.real_distance,
			unit: entry.scale.unit as Unit,
		},
	};
	return [
		entry.id,
		{
			measurement,
			scale,
			outputUnit: entry.output_unit as Unit,
			expected: entry.expected,
		},
	];
});

describe("RFC-007 baseline smoke", () => {
	test.each(BASELINE_DATA)("baseline %s", (id, data) => {
		const { expected, scale, measurement, outputUnit } = data;
		const state = new TakeoffStateHandler({
			pages: [],
			groups: [],
			measurements: [measurement],
			scales: [scale],
		});
		const m = state.getMeasurement(id);
		expect(m).not.toBeNull();

		if (expected.area) {
			const area = m!.convertArea(outputUnit);
			expect(area).not.toBeNull();
			expect(area).toBeCloseTo(expected.area);
		}
		if (expected.length) {
			const length = m!.convertLength(outputUnit);

			expect(length).not.toBeNull();
			expect(withinTolerance(length!, expected.length)).toBe(true);
		}
		if (expected.count) {
			const count = m!.count;
			expect(count).not.toBeNull();
			expect(expected.count).toBe(count);
		}
	});
});
