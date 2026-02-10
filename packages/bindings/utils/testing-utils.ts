import { faker } from "@faker-js/faker";
import type {
	Group,
	Measurement,
	Page,
	Scale,
	TakeoffStateHandler,
	Unit,
} from "../index.js";

export const generatePageIds = (count: number): string[] => {
	return Array.from({ length: count }, (_, i) => `page-${i}`);
};

export const createManyScales = (
	count: number,
	options?: {
		id?: string;
		pageId?: string;
		scale?: {
			pixelDistance: number;
			realDistance: number;
			unit: Unit;
		};
	},
): Scale[] => {
	const {
		id,
		pageId = "1",
		scale = { pixelDistance: 1, realDistance: 1, unit: "Meters" },
	} = options || {};
	return Array.from({ length: count }, (_, i) => ({
		id: id || `${faker.database.mongodbObjectId()}-${i}`,
		type: "Default",
		pageId,
		scale: {
			pixelDistance: scale.pixelDistance,
			realDistance: scale.realDistance,
			unit: scale.unit,
		},
	}));
};

export const createManyGroups = (
	count: number,
	options?: Partial<Group>,
): Group[] => {
	const {
		id = `group-${count}`,
		name,
		measurementType = "Area",
	} = options || {};
	return Array.from({ length: count }, (_, i) => ({
		id: `${id}-${i}`,
		name: name || `Group ${i}`,
		measurementType,
	}));
};

export const generatePoints = (
	type: Measurement["type"],
): Pick<Measurement, "type" | "points"> => {
	switch (type) {
		case "Rectangle": {
			const x1 = faker.number.float({ min: 0, max: 1000 });
			const y1 = faker.number.float({ min: 0, max: 1000 });
			return {
				type,
				points: [
					{ x: x1, y: y1 },
					{ x: x1 + 1, y: y1 + 1 },
				],
			};
		}
		case "Polygon":
			return {
				type,
				points: [
					{ x: 0, y: 0 },
					{ x: 1, y: 0 },
					{ x: 1, y: 1 },
					{ x: 0, y: 1 },
				],
			};
		case "Polyline":
			return {
				type,
				points: [
					{ x: 0, y: 0 },
					{ x: 1, y: 1 },
					{ x: 2, y: 2 },
				],
			};
		case "Count":
			return {
				type,
				points: [{ x: 0, y: 0 }],
			};
	}
};

export const createManyMeasurements = (
	count: number,
	options?: Partial<Omit<Measurement, "points">>,
): Measurement[] => {
	const {
		id,
		type = faker.helpers.arrayElement(["Rectangle", "Polygon"]),
		pageId = "1",
		groupId = "1",
	} = options || {};

	return Array.from({ length: count }, (_, i) => ({
		id: id || `${faker.database.mongodbObjectId()}-${i}`,

		pageId,
		groupId,
		...(generatePoints(type) as any),
	}));
};

export type UpsertHandler =
	| {
			type: "scale";
			value: Scale;
	  }
	| {
			type: "group";
			value: Group;
	  }
	| {
			type: "measurement";
			value: Measurement;
	  }
	| {
			type: "page";
			value: Page;
	  };

export const setupCalls = ({
	pages = [],
	groups = [],
	measurements = [],
	scales = [],
}: {
	pages?: Page[];
	groups?: Group[];
	measurements?: Measurement[];
	scales?: Scale[];
}): UpsertHandler[] => {
	const calls: UpsertHandler[] = [];

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
	for (const page of pages) {
		calls.push({
			type: "page",
			value: page,
		});
	}
	return calls;
};

export const executeCalls = (
	state: TakeoffStateHandler,
	calls: UpsertHandler[],
): void => {
	for (const call of calls) {
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
			case "page":
				state.upsertPage(call.value);
				break;
			default:
				throw new Error(`Unknown call type: ${call}`);
		}
	}
};
