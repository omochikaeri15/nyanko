## `engine.rs`

`engine.rs` is a straight translation of the native engine's animation pass, meaning the easing function and the per-part deployment it calls into, decompiled from **version 14.2** of the game and cross-checked against 15.5.1 where both were available. Everything the live game does in that pass, this does, down to which divisions truncate and exactly where the integers stop being integers.

What follows is only the part I **cannot** guarantee. Everything not listed here was read out of the decompiled binary rather than reasoned about.

**1. The division faults.** A model whose unit columns are zero, or two keyframes inside a polynomial run that land on the same frame, both make the native engine divide by zero and take the fault. I am not reproducing a crash. We leave the value alone in the first case and skip the term in the second. There is no correct result to copy, because the engine does not produce one, it dies.

**2. The global mirror flag.** The engine keeps a flag on the model that negates the root's horizontal scale and the rotation angle. That is the faction-facing flag, and it belongs to whatever places the entity in the world, so `resolve_frame` neither takes it nor applies it. A caller that wants a mirrored unit mirrors its own camera.

**3. The whole-texture sizing branch.** A sheet with the flag set is sized from the texture instead of a cutout. The sheet loader ends by writing the flag to false unconditionally on every path, so anything built from a `.png` and an `.imgcut` always comes back with it clear. Dead by the loader, not by the data.

Premultiplying the atlas in `SpriteSheet::parse`. The engine's default blend is `GL_ONE, GL_ONE_MINUS_SRC_ALPHA`, which only composites correctly against premultiplied source, so that is confirmed behavior rather than a workaround for edge fringing.
