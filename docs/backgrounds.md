# Background sources

The playground loads ten environment images from external hosts. No game raster
images are distributed in this repository. The images remain copyright their
respective owners and are outside this crate's MIT/Apache-2.0 license. Selecting
a scene requests its image from the credited source's image host.

- CS2 Dust II and Overpass come from Valve's [Counter-Strike 2 page](https://www.counter-strike.net/cs2).
  The images are `dust2_ct_ramp_s2.jpg` and `overpass_lower_tunnel_s2.jpg`.
- VALORANT Ascent and Icebox come from Riot Games' [maps page](https://playvalorant.com/en-us/maps/).
  The image identifiers begin `93f5b5f274df64d938051c4ca8577995190e711a`
  and `91422206c56df7177495b96a26021d6379de8079`.
- Wolfenstein: Enemy Territory Gold Rush, Fuel Dump, and Rail Gun come from level
  designer Mark Fry's [project portfolio](https://www.markfry.tv/level-design/wolfenstein-enemy-territory/).
  The respective images are `LD_wet_shot0026.jpg`, `LD_wet_shot0015.jpg`, and
  `LD_wet_shot0042.jpg`.
- Dirty Bomb Vault comes from MMO Culture's [Vault update coverage](https://mmoculture.com/2017/07/dirty-bomb-first-new-mercenary-added-after-separating-from-nexon/).
- Dirty Bomb Dockyard comes from Geek Blender's [Dockyard coverage](https://geekstuffpod.wixsite.com/geek-blender/single-post/2017/05/26/dirty-bomb-ya-tiene-nuevo-mapa-dockyard).
- Dirty Bomb Chapel comes from PC Gamer's [beta review](https://www.pcgamer.com/dirty-bomb-review-beta/).

The exact image and source URLs are recorded on each option in
[web/index.html](../web/index.html). All ten images were visually inspected and
loaded successfully in the browser on 2026-10-10 UTC. Availability depends on the
external host. The Dockyard and Chapel screenshots retain their original game HUD.

Images cover the preview while preserving aspect ratio; the viewport can crop
edges. The Bevy canvas remains transparent outside its rendered geometry.
A failed image leaves the solid background and shows recovery text. A stale image
completion cannot replace the latest selection.
