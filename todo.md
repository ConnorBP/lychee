### TODO

# design improvements

    - modularize render code
    - modularize main init code
    - efficiency improvements such as:
        - not reading out local player both in the entity list and by itself (get by index)
        - don't have to get the array of entity base addresses every frame probably
    - checking for valid or paged out values in the batcher (idk if this is necessary)
    - use get player index to get player from entity list instead of using the localplayer address func

    - make aimbot continue spray control for a bit after enemies die when there is no new enemy
        - tried this and broke aimbot. TODO: FIX AIMBOT

# feature improvements
    - pattern scanning CHECK
    - other internal features / future proofing
        - convar manager
        - netvar manager CHECK
        - interface manager
    - BSP parsing vischeck tracing
    - aimbot
    - reoil recorder for more legit looking recoil
    - humanized smoothing (possibly on the teensy with a LUT)
    - bloop when entity in front of crosshair?
    - config system
        - weapon based config
        - add stuff like is max movespeed for trigger to config per weapon
    - flashed check // bspotted mask may already account for this