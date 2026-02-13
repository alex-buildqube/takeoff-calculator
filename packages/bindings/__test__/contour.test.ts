import { describe, expect, test } from "vitest";
import { ContourWrapper, TakeoffStateHandler } from "../index.js";

describe("ContourWrapper", () => {
	test("should create a contour wrapper without scale (mesh is null)", () => {
		const contour = new ContourWrapper({
			id: "test-contour",
			name: "Test Contour",
			pageId: "test-page",
			lines: [
				{
					elevation: 0,
					unit: "Feet",
					points: [
						{ x: 0, y: 0 },
						{ x: 99.9, y: 0 },
						{ x: 99.9, y: 99.9 },
						{ x: 0, y: 99.9 },
					],
				},
			],
			pointsOfInterest: [],
		});
		expect(contour).toBeDefined();
		expect(contour.getSurfacePoints()).toBeNull();
		expect(contour.getScatterData(10)).toBeNull();
	});

	test("should work with state handler and deferred scale", () => {
		const state = new TakeoffStateHandler();

		state.upsertContour({
			id: "c1",
			pageId: "p1",
			lines: [
				{
					elevation: 0,
					unit: "Feet",
					points: [
						{ x: 0, y: 0 },
						{ x: 99.9, y: 0 },
						{ x: 99.9, y: 99.9 },
						{ x: 0, y: 99.9 },
					],
				},
			],
			pointsOfInterest: [],
		});

		// No scale yet
		let contour = state.getContour("c1");
		expect(contour).toBeDefined();
		expect(contour?.getSurfacePoints()).toBeNull();

		// Add scale
		state.upsertScale({
			type: "Default",
			id: "s1",
			pageId: "p1",
			scale: {
				pixelDistance: 1,
				realDistance: 1,
				unit: "Feet",
			},
		});

		// Now mesh should be available
		contour = state.getContour("c1");
		expect(contour?.getSurfacePoints()).toBeDefined();
		expect(contour?.getSurfacePoints()?.length).toBe(4);
	});

	test("should generate scatter data with scale", () => {
		const state = new TakeoffStateHandler();

		state.upsertScale({
			type: "Default",
			id: "s1",
			pageId: "p1",
			scale: {
				pixelDistance: 1,
				realDistance: 1,
				unit: "Feet",
			},
		});

		state.upsertContour({
			id: "c1",
			pageId: "p1",
			lines: [
				{
					elevation: 0,
					unit: "Feet",
					points: [
						{ x: 0, y: 0 },
						{ x: 99.9, y: 0 },
						{ x: 99.9, y: 99.9 },
						{ x: 0, y: 99.9 },
					],
				},
			],
			pointsOfInterest: [],
		});

		const contour = state.getContour("c1");
		const scatterData = contour?.getScatterData(10);
		expect(scatterData).toBeDefined();
		expect(scatterData?.length).toBe(100);
		for (const point of scatterData ?? []) {
			expect(point.x).toBeDefined();
			expect(point.y).toBeDefined();
			expect(point.z).toBeDefined();
			expect(point.z).toBe(0);
		}
	});

	test("should compute raw volume against reference", () => {
		const state = new TakeoffStateHandler();

		state.upsertScale({
			type: "Default",
			id: "s1",
			pageId: "p1",
			scale: {
				pixelDistance: 1,
				realDistance: 1,
				unit: "Feet",
			},
		});

		state.upsertContour({
			id: "c1",
			pageId: "p1",
			lines: [
				{
					elevation: 0,
					unit: "Feet",
					points: [
						{ x: 0, y: 0 },
						{ x: 99.9, y: 0 },
						{ x: 99.9, y: 99.9 },
						{ x: 0, y: 99.9 },
					],
				},
			],
			pointsOfInterest: [],
		});

		const contour = state.getContour("c1");

		const volume = contour?.rawVolumeAgainst({
			type: "Rectangle",
			points: [
				{ x: 25, y: 25 },
				{ x: 75, y: 75 },
			],
			elevation: 0,
		});
		expect(volume).toBeDefined();
		expect(volume?.cut).toBe(0);
		expect(volume?.fill).toBe(0);
		expect(volume?.uncoveredArea).toBe(0);

		const volume2 = contour?.rawVolumeAgainst(
			{
				type: "Rectangle",
				points: [
					{ x: 0, y: 0 },
					{ x: 10, y: 10 },
				],
				elevation: 1,
			},
			1,
		);
		expect(volume2).toBeDefined();
		expect(volume2?.cut).toBe(0);
		expect(volume2?.fill).toBe(100);
		expect(volume2?.uncoveredArea).toBe(0);

		const volume3 = contour?.rawVolumeAgainst(
			{
				type: "Rectangle",
				points: [
					{ x: 25, y: 25 },
					{ x: 35, y: 35 },
				],
				elevation: -1,
			},
			1,
		);
		expect(volume3).toBeDefined();
		expect(volume3?.cut).toBe(100);
		expect(volume3?.fill).toBe(0);
		expect(volume3?.uncoveredArea).toBe(0);
	});

	test("should compute unit-aware volume", () => {
		const state = new TakeoffStateHandler();

		state.upsertScale({
			type: "Default",
			id: "s1",
			pageId: "p1",
			scale: {
				pixelDistance: 1,
				realDistance: 1,
				unit: "Feet",
			},
		});

		state.upsertContour({
			id: "c1",
			pageId: "p1",
			lines: [
				{
					elevation: 0,
					unit: "Feet",
					points: [
						{ x: 0, y: 0 },
						{ x: 99.9, y: 0 },
						{ x: 99.9, y: 99.9 },
						{ x: 0, y: 99.9 },
					],
				},
			],
			pointsOfInterest: [],
		});

		const contour = state.getContour("c1");

		const unitVolume = contour?.volumeAgainst({
			type: "Rectangle",
			points: [
				{ x: 25, y: 25 },
				{ x: 75, y: 75 },
			],
			elevation: 0,
		});
		expect(unitVolume).toBeDefined();
		expect(unitVolume?.cut).toBeDefined();
		expect(unitVolume?.fill).toBeDefined();
		expect(unitVolume?.uncoveredArea).toBeDefined();
		// With 1:1 scale, unit values should equal raw values
		expect(unitVolume?.cut.getConvertedValue("Feet")).toBe(0);
		expect(unitVolume?.fill.getConvertedValue("Feet")).toBe(0);
	});

	test("should get z data for a raised point in a contour", () => {
		const state = new TakeoffStateHandler();

		state.upsertScale({
			type: "Default",
			id: "s1",
			pageId: "p1",
			scale: {
				pixelDistance: 1,
				realDistance: 1,
				unit: "Feet",
			},
		});

		state.upsertContour({
			id: "c1",
			pageId: "p1",
			lines: [
				{
					elevation: 0,
					unit: "Feet",
					points: [
						{ x: 0, y: 0 },
						{ x: 99.9, y: 0 },
						{ x: 99.9, y: 99.9 },
						{ x: 0, y: 99.9 },
					],
				},
			],
			pointsOfInterest: [
				{
					elevation: 100,
					unit: "Feet",
					point: { x: 50, y: 50 },
				},
			],
		});

		const contour = state.getContour("c1");
		const scatterData = contour?.getScatterData(10);
		expect(scatterData).toBeDefined();
		expect(scatterData?.length).toBe(100);

		const z = contour?.getZAt(50, 50);
		expect(z).toBeDefined();
		expect(z).toBe(100);
	});
});
