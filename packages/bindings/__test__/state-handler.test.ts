import { faker } from "@faker-js/faker";
import { describe, expect, test } from "vitest";
import { type Measurement, type Scale, TakeoffStateHandler } from "../index.js";
import {
	createManyGroups,
	createManyMeasurements,
	createManyScales,
	generatePageIds,
	type UpsertHandler,
} from "../utils/testing-utils.js";

describe("TakeoffStateHandler", () => {
	test("should get measurements by group id", () => {
		const state = new TakeoffStateHandler({
			pages: [],
			groups: [],
			measurements: [],
			scales: [],
		});
		const measurements = state.getMeasurementsByGroupId("1");
		expect(measurements).toEqual([]);
	});

	test("should process measurements and groups", () => {
		const state = new TakeoffStateHandler({
			pages: [],
			groups: [],
			measurements: [],
			scales: [],
		});
		const pageIds = generatePageIds(10);
		const scales = pageIds.flatMap((pageId) => createManyScales(1, { pageId }));
		const groups = createManyGroups(10);
		const measurements = createManyMeasurements(100).map((measurement) => {
			const scale = faker.helpers.arrayElement(scales);

			const group = faker.helpers.arrayElement(groups);
			return {
				...measurement,
				scaleId: scale.id,
				pageId: scale.pageId,
				groupId: group.id,
			};
		});

		for (const group of groups) {
			state.upsertGroup(group);
		}

		for (const scale of scales) {
			state.upsertScale(scale);
		}
		for (const measurement of measurements) {
			state.upsertMeasurement(measurement);
		}
		const testMeasure = state.getMeasurement(
			faker.helpers.arrayElement(measurements).id,
		);
		expect(testMeasure).toBeDefined();

		expect(testMeasure?.area).toBeDefined();
		expect(testMeasure?.length).toBeDefined();

		expect(testMeasure?.area?.display("Meters")).toBe("1 m²");
		const testGroup = state.getGroup(testMeasure!.measurement.groupId)!;
		expect(testGroup?.area).toBeDefined();

		const groupMeasurementCount = testGroup.count!;
		expect(testGroup.points).toBe(groupMeasurementCount * 4);
		expect(testGroup.length?.getConvertedValue("Meters")).toBe(
			groupMeasurementCount * 4,
		);
		expect(testGroup.area?.getConvertedValue("Meters")).toBe(
			groupMeasurementCount,
		);

		const newTestScale: Scale = {
			type: "Default",
			id: faker.database.mongodbObjectId(),
			pageId: faker.string.uuid(),
			scale: {
				pixelDistance: 10,
				realDistance: 0.5,
				unit: "Feet",
			},
		};
		const newTestMeasure: Measurement = {
			type: "Rectangle",
			id: faker.database.mongodbObjectId(),
			pageId: newTestScale.pageId,
			groupId: faker.helpers.arrayElement(groups).id,
			points: [
				{ x: 0, y: 0 },
				{ x: 10, y: 10 },
			],
		};

		state.upsertMeasurement(newTestMeasure);
		const newMeasureWrapper = state.getMeasurement(newTestMeasure.id);
		console.log(newMeasureWrapper?.area, newMeasureWrapper?.length);
		expect(newMeasureWrapper?.area).toBeNull();
		expect(newMeasureWrapper?.length).toBeNull();
		expect(state.getMeasurementsMissingScale().length).toBe(1);
		state.upsertScale(newTestScale);
		expect(newMeasureWrapper?.area).toBeDefined();
		expect(newMeasureWrapper?.length).toBeDefined();
		expect(newMeasureWrapper?.area?.display("Feet")).toBe("0.25 ft²");
		expect(newMeasureWrapper?.length?.display("Feet")).toBe("2 ft");
	});

	test("should handle items in random order", () => {
		const state = new TakeoffStateHandler();
		const calls: UpsertHandler[] = [];
		const pageIds = generatePageIds(25);
		const scales = pageIds.flatMap((pageId) => createManyScales(1, { pageId }));
		const groups = createManyGroups(10);

		const measurements: Measurement[] = createManyMeasurements(1000).map(
			(measurement) => {
				const scale = faker.helpers.arrayElement(scales);
				const group = faker.helpers.arrayElement(groups);
				return {
					...measurement,

					scaleId: scale.id,
					pageId: scale.pageId,
					groupId: group.id,
				};
			},
		);
		for (const scale of scales) {
			calls.push({
				type: "scale",
				value: scale,
			});
		}
		for (const group of groups) {
			calls.push({
				type: "group",
				value: group,
			});
		}
		for (const measurement of measurements) {
			calls.push({
				type: "measurement",
				value: measurement,
			});
		}
		for (const call of faker.helpers.shuffle(calls)) {
			switch (call.type) {
				case "scale":
					state.upsertScale(call.value);
					break;
				case "group":
					state.upsertGroup(call.value);
					break;
				case "measurement":
					state.upsertMeasurement(call.value);
					break;
			}
		}

		expect(state.getMeasurementsMissingScale().length).toBe(0);
		const sampledMeasurements = faker.helpers.arrayElements(measurements, 10);
		for (const measurement of sampledMeasurements) {
			const testMeasure = state.getMeasurement(measurement.id);
			expect(testMeasure).toBeDefined();
			expect(testMeasure?.area).toBeDefined();
			expect(testMeasure?.length).toBeDefined();
			expect(testMeasure?.area?.display("Meters")).toBe("1 m²");
			const testGroup = state.getGroup(testMeasure!.measurement.groupId)!;
			const groupMeasurements = state.getMeasurementsByGroupId(
				testMeasure!.groupId,
			);
			for (const measurement of groupMeasurements) {
				expect(measurement.groupId).toBe(testGroup.id);
			}
			expect(groupMeasurements.length).toBe(testGroup.count);
			for (const measurement of groupMeasurements) {
				expect(measurement.area).toBeDefined();
				expect(measurement.length).toBeDefined();
				expect(measurement.area?.display("Meters")).toBe("1 m²");
				expect(measurement.length?.display("Meters")).toBe("4 m");
				expect(measurement.points).toBe(4);
			}
			expect(testGroup?.area).toBeDefined();
			expect(testGroup?.length).toBeDefined();
		}
	});

	test("should handle initially added data", () => {
		const pageIds = generatePageIds(25);
		const scales = pageIds.flatMap((pageId) => createManyScales(1, { pageId }));
		const groups = createManyGroups(10);

		const measurements: Measurement[] = createManyMeasurements(1000).map(
			(measurement) => {
				const scale = faker.helpers.arrayElement(scales);
				const group = faker.helpers.arrayElement(groups);
				return {
					...measurement,

					scaleId: scale.id,
					pageId: scale.pageId,
					groupId: group.id,
				};
			},
		);
		const state = new TakeoffStateHandler({
			pages: [],
			groups,
			measurements,
			scales,
		});
		expect(state.getMeasurementsMissingScale().length).toBe(0);

		const sampledMeasurements = faker.helpers.arrayElements(measurements, 10);
		for (const measurement of sampledMeasurements) {
			const testMeasure = state.getMeasurement(measurement.id);
			expect(testMeasure).toBeDefined();
			expect(testMeasure?.area).toBeDefined();
			expect(testMeasure?.length).toBeDefined();
			expect(testMeasure?.area?.display("Meters")).toBe("1 m²");
			const testGroup = state.getGroup(testMeasure!.measurement.groupId)!;
			const groupMeasurements = state.getMeasurementsByGroupId(
				testMeasure!.groupId,
			);
			expect(groupMeasurements.length).toBe(testGroup.count);
			for (const measurement of groupMeasurements) {
				expect(measurement.area).toBeDefined();
				expect(measurement.length).toBeDefined();
				expect(measurement.area?.display("Meters")).toBe("1 m²");
				expect(measurement.length?.display("Meters")).toBe("4 m");
				expect(measurement.points).toBe(4);
			}
			expect(testGroup?.area).toBeDefined();
			expect(testGroup?.length).toBeDefined();
		}
	});
});
