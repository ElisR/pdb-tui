# pdb-tui

Rendering proteins in the terminal with Rust

## TODO Priorities

- [ ] Swap to `wgpu` for rendering

- [ ] Choose a simpler enum representation for colours
- [ ] Load obj or PDB depending on filetype
- [ ] Add hierarchy of shapes to allow for sensible colouring
- [ ] Move to async polling of keys
- [ ] Refactor UI updates into the state structs
- [ ] Load to CoM of each PDB file, rather than CoM of entire scene
- [ ] Make scene `znear` and `zfar` sensitive to size of object.
- [ ] Use a macro to define the help screen from the function which decides the next action.



## Specific GPU Priorities

- [x] Swap back to `nalgebra`
- [x] Create common wrapper struct around next actions.
- [x] Integrate basic TUI into new GPU-accelerated code.
- [ ] Make buffer work when not aligned to 256.
- [ ] Add a trivial compute shader to the pipeline right at the end. Start off with just subsampling.
- [ ] Write a benchmarking script.
- [ ] Look for performance improvements in `ratatui` components.
- [ ] Add colour back in.
- [ ] Refactor structure.
- [ ] Bring back convenience of loading `.obj` files from the command line.


### Internal Notes

Apparently, a `Surface` can be generated safely now. (Requires window lifetime.)

Should maybe consider the `#[repr(C, align(16))]` macro for aligning structs, hopefully avoiding padding.
Maybe this just changes the alignment of the whole struct, however.
