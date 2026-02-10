import { describe, expect, test } from "vitest";
import { ContourWrapper } from "../index.js";

// import { TEST_CONTOUR } from "../utils/testing-utils.js";

describe("ContourWrapper", () => {
	test("should create a contour wrapper", () => {
		const contour = new ContourWrapper({
			id: "test-contour",
			name: "Test Contour",
			pageId: "test-page",
			lines: [
				{
					elevation: 0,
					points: [
						{
							x: 0,
							y: 0,
						},
						{
							x: 99.9,
							y: 0,
						},
						{
							x: 99.9,
							y: 99.9,
						},
						{
							x: 0,
							y: 99.9,
						},
					],
				},
			],
			pointsOfInterest: [],
		});
		expect(contour).toBeDefined();
		const scatterData = contour.getScatterData(10);
		expect(scatterData).toBeDefined();
		expect(scatterData?.length).toBe(100);
		for (const point of scatterData ?? []) {
			expect(point.x).toBeDefined();
			expect(point.y).toBeDefined();
			expect(point.z).toBeDefined();
			expect(point.z).toBe(0);
		}
		const volume = contour.volumeAgainst({
			type: "Rectangle",
			points: [
				{
					x: 25,
					y: 25,
				},
				{
					x: 75,
					y: 75,
				},
			],
			elevation: 0,
		});
		expect(volume).toBeDefined();
		expect(volume?.cut).toBe(0);
		expect(volume?.fill).toBe(0);
		expect(volume?.uncoveredArea).toBe(0);

		const volume2 = contour.volumeAgainst(
			{
				type: "Rectangle",
				points: [
					{
						x: 0,
						y: 0,
					},
					{
						x: 10,
						y: 10,
					},
				],
				elevation: 1,
			},
			1,
		);
		expect(volume2).toBeDefined();
		expect(volume2?.cut).toBe(0);
		expect(volume2?.fill).toBe(100);
		expect(volume2?.uncoveredArea).toBe(0);

		const volume3 = contour.volumeAgainst(
			{
				type: "Rectangle",
				points: [
					{
						x: 25,
						y: 25,
					},
					{
						x: 35,
						y: 35,
					},
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
});
