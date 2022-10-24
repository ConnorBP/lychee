### TODO

# always on improvements
    - check periodically for if process is open
    - if process is not open then go in a loop waiting for process
    - efficiency improvements such as:
        - only occaisonally updating local player address
        - not reading out local player both in the entity list and by itself (get by index)
        - don't have to get the array of entity base addresses every frame probably

# feature improvements
    - Get Player origins
        - aimbot
        - better triggerbot (calculate positions)
            - vischeck using bspotted and bspottedbymask?
                - if bspottedmask doesn't work well i gotta port the bsp parser
    - bloop when entity in front of crosshair?

    - change over to bluetooth serial + teensy (needs a fancy voltage level circuit)