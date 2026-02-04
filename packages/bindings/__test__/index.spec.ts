import { expect, test, describe } from "vitest";

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
});
