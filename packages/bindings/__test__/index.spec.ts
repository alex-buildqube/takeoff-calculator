import { describe, expect, test } from "vitest";

import { plus100, TakeoffStateHandler } from "../index.js";

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
