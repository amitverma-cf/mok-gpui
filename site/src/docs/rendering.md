# Rendering

`mok_gpui::rendering::RenderSurface` shows pixels your app draws: a map's texture layer, a plot, a frame from a camera or another renderer. It runs wherever GPUI runs: macOS, Windows and Linux, on arm64 and x86_64, and in the browser.

## What GPUI can show

GPUI at Mok's revision puts pixels on screen three ways:

| Way | Where | What it takes |
| --- | --- | --- |
| `Window::paint_image` with a `RenderImage` | Every platform and the browser | Frames in BGRA. Each new frame goes up to GPUI's sprite atlas. |
| `Window::paint_surface` with a `CVPixelBuffer` | macOS alone | NV12 video frames. The wgpu and DirectX renderers drop it. |
| A native view laid over the window | Per platform | It draws above everything GPUI paints, and GPUI's clipping does not reach it. |

GPUI shares no GPU device and runs no outside shader. A shader, a 3D scene or a camera runs on your side, on your own device, and hands its frames to `RenderSurface` through the first way. Zero-copy textures and Metal, OpenGL, Vulkan or Direct3D surfaces inside GPUI's own drawing have no way in.

## The surface

`RenderSurface::new(id, ratio, draw)` takes the box's width over its height and a function that draws one frame. It fills the width it is given and keeps the ratio.

| Item | Does |
| --- | --- |
| `draw(size, cx)` | Returns a `RenderImage` of one frame, exactly `size` in device pixels: the box's size times the window's scale factor, so every pixel lands on one screen pixel. |
| `.revision(n)` | Your count of the content. The surface draws again when the size or the revision changes, and only then. |

A frame of the wrong size, or with more than one picture, panics with the surface's id. A box with no width or height draws nothing. The surface frees the old frame from the atlas each time it draws, and its last frame when it leaves the window. Each draw logs its size, revision and time at `debug`.

`draw` runs on the main thread in the surface's prepaint. Keep it to work one frame can afford, or draw elsewhere and copy the finished pixels in under a new revision.

## The revision

The revision covers everything a frame reads. If the frame takes a color from the theme, fold the color into it, or a change of mode leaves the old colors on screen. The gallery's story hashes the slider's place and both colors it paints with.

## A window with a surface

The example draws stripes across a surface and adds two each time the button is pressed.

```rust example=render
```
