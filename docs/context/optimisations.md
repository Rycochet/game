# Optimisations (tried, not adopted)

Findings worth revisiting later. Nothing here is in the code. Versions matter: re-measure if Bevy, Rust or the GPU driver change.

## Client memory (2026-10-28)

**Versions:** Bevy 0.20.0 (wgpu 30), Rust 1.99.1, Windows 11 (26200), NVIDIA GeForce RTX 4060 Laptop GPU, driver 617.42, Vulkan backend.

**Method:** start the client, wait 10 s, read memory with PowerShell (`WorkingSet64`, `PrivateMemorySize64`, and `WorkingSetPrivate`, which is what Task Manager's "Memory" column shows). Same terrain at position 0,0.

**Starting point:** debug build ~585 MB working set / ~1,065 MB private / 63 threads. Task Manager showed ~430 MB.

| Change | Effect (debug, working set) | Notes |
| --- | --- | --- |
| Explicit Bevy feature list instead of defaults | −30 MB | Drops ~16 plugins (audio, gamepads, glTF, scenes, animation, picking, gizmos, sprites, post-processing, ...). Needs care: e.g. PNG screenshots need the `png` feature, and the default `Tonemapping` needs `tonemapping_luts`. Feature list used: `std, multi_threaded, async_executor, bevy_log, bevy_asset, bevy_window, bevy_winit, x11, wayland, webgl2, bevy_render, bevy_core_pipeline, bevy_camera, bevy_light, bevy_pbr, bevy_mesh, bevy_image, bevy_shader, bevy_material, bevy_color, bevy_ui, bevy_ui_render, bevy_text, default_font, png`. |
| `TaskPoolOptions::with_num_threads(4)` | −30 MB, −17 threads | Bevy otherwise starts roughly one thread per core per pool. |
| One shadow cascade (`CascadeShadowConfigBuilder { num_cascades: 1, maximum_distance: CAMERA_DISTANCE * 2.0 }`) | −15 MB | No visible change: the camera distance is fixed. |
| Disable `PipelinedRenderingPlugin` (native only) | small | Not needed at the ~30 fps cap. `disable::<T>()` panics if the plugin isn't in the group, so gate it with `#[cfg(not(target_arch = "wasm32"))]`. |
| MSAA off | −10 MB | Not worth it: terrain detail shimmers. |
| wgpu validation off (`WGPU_VALIDATION=0`, `WGPU_DEBUG=0`) | none | |
| DirectX 12 (`WGPU_BACKEND=dx12`) | **+270 MB** | Stay on Vulkan. |
| Shadow map 1024 instead of 2048 | negligible | |

**All adopted together:** debug 519 MB, release 487 MB working set (950 MB private); Task Manager ~325 MB. That's roughly a 25% saving in Task Manager terms.

**Why not adopted:** the owner judged the saving not worth the extra upkeep (an explicit feature list has to be maintained as features are needed). Most remaining memory is the GPU driver and Bevy's render pipelines rather than game code.

**Easy wins if memory matters later:** the thread count and single shadow cascade are a few lines each with no feature-list upkeep (together ~45 MB).

## 2026-11-02: Keeping item pictures on disk

Planned so that pictures need not be drawn again each time the game starts. Measured on the owner's laptop, in the native debug build, with Item Pictures asking for 147 pictures at once: 1.56 s until the graphics card's programs had compiled, then 131 pictures in 3.2 s, about 40 a second, with the slowest frame 37.5 ms and most about 35 ms. The frame time with nothing drawing wasn't measured.

Not adopted. A full inventory is a few dozen pictures, so under a second once compiled, with each item's edge showing until then. The compile would mostly be saved only for the first look at an inventory, while a cache would need reading pictures back from the graphics card, a PNG library, file handling on another thread, clearing out old files and a version to raise when drawing changes, for native only. Worth another look if models get much more expensive, or if the wait on first opening an inventory is noticed.

Since then (2026-11-03) pictures are drawn a page at a time with a 2D edge (see `decisions.md`): 300 pictures take about 1.75 s rather than 6.8 s, and 40 about 0.57 s rather than 1.29 s, of which building the models is under a tenth. A cache would save even less now.
