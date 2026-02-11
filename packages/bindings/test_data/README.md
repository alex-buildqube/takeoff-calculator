# Baseline test data (RFC-007)

This directory contains versioned baseline data for accuracy (golden) tests. The master document for tolerance policy and how to add/update baselines is **`../ACCURACY.md`**.

## Schema

**File**: `baseline.json`

Root: array of baseline entries. Each entry:

| Field         | Type   | Description                                                                                                                                                                                                  |
| ------------- | ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `id`          | string | Unique identifier for the case.                                                                                                                                                                              |
| `kind`        | string | `"Polygon"`, `"Polyline"`, `"Rectangle"`, or `"Count"`.                                                                                                                                                      |
| `points`      | array  | For Polygon/Polyline: array of `{ "x": number, "y": number }`. For Rectangle: exactly 2 points (corners). For Count: exactly 1 point.                                                                        |
| `scale`       | object | `{ "pixel_distance": number, "real_distance": number, "unit": string }`. Unit: one of "Feet", "Inches", "Yards", "Meters", "Centimeters".                                                                    |
| `output_unit` | string | Unit for expected values (same set as scale.unit).                                                                                                                                                           |
| `expected`    | object | `{ "length"?: number, "area"?: number, "count"?: number }`. Only include keys applicable to the kind (e.g. Polygon: area and optionally length; Polyline: length; Rectangle: area and length; Count: count). |

- **Polygon**: area (Shoelace); length = perimeter. Both converted via scale and output_unit.
- **Polyline**: length = sum of segment lengths; no area.
- **Rectangle**: area = width×height, length = perimeter, in real-world units.
- **Count**: count = 1 (single point); no length/area conversion.

Expected values are in `output_unit`. Units are case-insensitive when parsed (e.g. "Feet" or "feet").

## Adding a new baseline

1. Append an entry to `baseline.json` with a unique `id`.
2. Run the golden test or a small conversion to get actual values; set `expected` accordingly.
3. Run `cargo test -p takeoff_calculator golden` and commit.
