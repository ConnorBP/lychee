### TODO

# design improvements
    - efficiency improvements such as:
        - only occaisonally updating local player address
        - not reading out local player both in the entity list and by itself (get by index)
        - don't have to get the array of entity base addresses every frame probably
    - checking for valid or paged out values in the batcher
    - possibility to build our own viewmatrix if need be?
    - figure out why viewoffset isn't working on other player entities
    - use get player index to get player from entity list instead of using the localplayer address func

# feature improvements
    - aimbot
    - better triggerbot (calculate positions)
        - vischeck using bspotted and bspottedbymask?
            - if bspottedmask doesn't work well i gotta port the bsp parser
    - minimap
    - bloop when entity in front of crosshair?
    - change over to bluetooth serial + teensy (needs a fancy voltage level circuit)
    - reoil recorder for more legit looking recoil
    - weapon detection
    - config system
    - flashed check
    - bsp parser
    - weapon based config