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
    - restructure aimbot to use struct instead of statics for persistant data
    - make aimbot continue spray control for a bit after enemies die when there is no new enemy

# feature improvements
    - aimbot
    - replace render system
    - minimap
    - bloop when entity in front of crosshair?
    - change over to bluetooth serial + teensy (needs a fancy voltage level circuit)
    - reoil recorder for more legit looking recoil
    - config system
    - flashed check // bspotted mask may already account for this
    - bsp parser
    - weapon based config