//! A standard part can be a gear, worm or timing-belt pulley: spur, helical, herringbone,
//! internal ring and straight bevel gears and worm wheels (`family` `spur_gear_m0_5` …
//! `bevel_gear_m3`, ISO 53 profile; a bevel gear's `size` its teeth and its mate's, "20:40"),
//! worms (`worm_m…`, `size` the number of starts) and pulleys (`pulley_gt2`, `pulley_htd3m`,
//! `pulley_htd5m`), `size` otherwise the number of teeth and `length` the width (a worm's length,
//! a bevel gear's face width). Bevel pairs, worm drives and planetary sets go in as one standard
//! part per gear. More screws and set screws join the fasteners (`low_head_cap_screw`,
//! `button_head_screw`, `flanged_button_head_screw`, `countersunk_socket_screw`,
//! `countersunk_screw`, `pan_head_screw`, `pan_head_slotted_screw`, `cheese_head_screw`,
//! `set_screw_cone`, `set_screw_dog`, `set_screw_cup`). The values chosen for the part are saved
//! with it: `bore` (mm), a helical gear's `helix` angle (radians, negative: left hand), a worm
//! wheel's worm `starts`, a gear's `pressure_angle` (radians), `profile_shift` (coefficient),
//! `backlash`, `root_fillet` and `tip_fillet` (mm) and `offset_hole` ([distance, diameter], mm),
//! a screw's `drive` (`hex`, `hexalobular`, `cross`, `slot`) and `modeled_thread` (the thread cut
//! into the part rather than cosmetic); none: the size's or the family's default. Older designs
//! need no transformation: they have no such parts, and no other part takes these fields. The
//! last release refuses a design saved with this (the newer format number).
