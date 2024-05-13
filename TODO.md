## TODO Priorities

- [x] Swap to `wgpu` for rendering

- [ ] Choose a simpler enum representation for colours
- [ ] Load obj or PDB depending on filetype
- [ ] Add hierarchy of shapes to allow for sensible colouring
- [ ] Move to async polling of keys
- [ ] Refactor UI updates into the state structs
- [ ] Load to CoM of each PDB file, rather than CoM of entire scene
- [ ] Make scene `znear` and `zfar` sensitive to size of object.
- [ ] Use a macro to define the help screen from the function which decides the next action.
- [ ] Deprecate old CPU-based rendering code and move it to less visible location.

## Specific GPU Priorities

- [x] Add a trivial compute shader to the pipeline right at the end. Start off with just subsampling.
- [x] Allow for grid-sizes of bigger than 1x1.
- [x] Write a compute shader for traditional ASCII rasterisation.

- [ ] Fix buffer sizes for images in `save_screenshot`.
- [ ] Add colour back in.
- [ ] Refactor structure.
- [ ] Bring back convenience of loading `.obj` files from the command line.

- [ ] Write a benchmarking script.
- [ ] Look for performance improvements in `ratatui` components.
- [ ] Swap to different mechanism for event handling - hopefully fixing window resize events not being registered until next input.

### Internal Notes

Apparently, a `Surface` can be generated safely now. (Requires window lifetime.)
