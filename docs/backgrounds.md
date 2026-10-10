# Background sources

The playground loads four environment images from the publishers' servers.
No game raster images are distributed in this repository. The images remain
copyright their respective publishers and are outside this crate's MIT/Apache-2.0
license. Selecting a scene requests its image from the publisher's CDN.

- CS2 Dust II and Overpass come from Valve's [Counter-Strike 2 page](https://www.counter-strike.net/cs2).
  The selected images are `dust2_ct_ramp_s2.jpg` and `overpass_lower_tunnel_s2.jpg`.
- VALORANT Ascent and Icebox come from Riot Games' [maps page](https://playvalorant.com/en-us/maps/).
  The selected image identifiers begin `93f5b5f274df64d938051c4ca8577995190e711a`
  and `91422206c56df7177495b96a26021d6379de8079`.

The exact image and source URLs are recorded on each option in
[web/index.html](../web/index.html). Images were visually inspected and loaded
successfully in the browser on 2026-10-10 UTC. Availability depends on the publisher.

Images cover the preview while preserving aspect ratio; the viewport can crop
edges. The Bevy canvas remains transparent outside the crosshair and hit marker.
A failed image leaves the solid background and shows recovery text. A stale image
completion cannot replace the latest selection.
