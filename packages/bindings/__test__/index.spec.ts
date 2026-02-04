import { expect, test } from "vitest";

import { plus100 } from "../index.js";

test("sync function from native code", () => {
  const fixture = 42;
  expect(plus100(fixture)).toEqual(fixture + 100);
});
