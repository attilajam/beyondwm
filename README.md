# BeyondWM

An infinite canvas window manager built using smithay. Big thanks to [smithay](https://github.com/Smithay/smithay/tree/master), [smallvile](https://github.com/Smithay/smithay/tree/master/smallvil), and [niri](https://github.com/niri-wm/niri/tree/main) for making this possible.
  
## Building

```bash
git clone https://github.com/attilajam/beyondwm.git
cd beyondwm
cargo build
```

## How it works

Store a camera position, and shift all windows according to camera position at render time. This effectively adds a couple additions every time the window surface is rendered, and the performance drop-off is minimal.

To move around the canvas, hold Super+Drag, allowing you to look across the canvas. 

## State

- [x] Basic canvas panning
- [x] Wayland Layer Protocol implemented (allows launchers to work)
- [ ] Zoom in/out of canvas
- [ ] Keyboard shortcuts for navigating around canvas
- [ ] Trackpad gestures for navigating around canvas
... and more!

## Contributing

Human written contributions are always welcome!
