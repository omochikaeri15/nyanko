### Architecture Note: In The Event of Necessary True Native Engine Parity

This timeline implementation currently uses a hybrid "Float Cast to Int" approach for easing curves (`ease_mode == 2`, exponential). It casts integers to `f32`, calculates the percentage/curve, and truncates back to an integer. This is the industry-standard way to emulate integer-bound timelines while retaining smooth interpolation logic, and fixes many snapping and smoothing bugs that you would find in Float implementations by enforcing rigid `i32` frames at the API boundary.

**However**, if this hybrid approach ever causes any bug or discrepancy, minor or major, compared to the live game, it means the original engine's `double` casting quirks are bleeding into the render state in ways `f32` cannot perfectly emulate.

In the event that this Float implementation causes either a bug that I can not fix within a reasonable amount of time, or a bug I cannot figure out how to fix period, this API will transition to true native engine parity.

True native engine parity will cause some small benefits of Floats, such as inherent delta time handling and the ability to interpolate past the games native 30fps, to be lost.

**To Revert to True Native Parity (If Unsolvable Issues Arise):**

1. **Review the Original Logic:** Reference the native engine's core animation and timeline resolution procedures.
2. **Acknowledge Precision Differences:** The original engine utilizes standard double-precision (`f64`/`double`) for its temporary accumulation inside exponential easing blocks before saving the result back to the integer limb struct.
3. **Upgrade `interpolate_curve`:** Modify `interpolate_curve` to calculate the mathematical progress and exponential factors using `f64` (double-precision) instead of `f32` (single-precision). This mimics the exact memory footprint of the native operations.
4. **Delay Single-Precision Casting:** Only cast to `f32` at the very last step when returning the final truncated value. Alternatively, change the return type of `interpolate_curve` to `i32` entirely, and push the `f32` cast out to the main `animate` match block.
5. **Preserve Lagrange Logic:** Retain the `Into::into()` lossless conversions for the `ease_mode == 3` (Lagrange polynomial) logic. This relies purely on `i64` integer bit-shifting and is already mathematically identical to the native engine's logic.