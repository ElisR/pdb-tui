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

