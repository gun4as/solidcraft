//! A standard part can be a gear, worm, cycloidal disc or timing-belt pulley: spur, helical,
//! herringbone and internal ring gears and worm wheels (`family` `spur_gear_m0_5` …
//! `worm_wheel_m3`, ISO 53 profile), worms (`worm_m…`, `size` the number of starts), cycloidal
//! discs (`cycloidal_disc_d40` … `_d100`, `size` the number of lobes) and pulleys (`pulley_gt2`,
//! `pulley_htd3m`, `pulley_htd5m`), `size` otherwise the number of teeth and `length` the width
//! (a worm's length). More screws and set screws join the fasteners (`low_head_cap_screw`,
//! `button_head_screw`, `flanged_button_head_screw`, `countersunk_socket_screw`,
//! `countersunk_screw`, `pan_head_screw`, `pan_head_slotted_screw`, `cheese_head_screw`,
//! `set_screw_cone`, `set_screw_dog`, `set_screw_cup`). The values chosen for the part are saved
//! with it: `bore` (mm), a helical gear's `helix` angle (radians, negative: left hand), a worm
//! wheel's worm `starts`, a screw's `drive` (`hex`, `hexalobular`, `cross`, `slot`) and
//! `modeled_thread` (the thread cut into the part rather than cosmetic); none: the size's or the
//! family's default. Older designs need no transformation: they have no such parts, and no other
//! part takes these fields. The last release refuses a design saved with this (the newer format
//! number).
