multi-rymd
--------------
rough list of things to work on in the game currently, usually ordered as items highest up = higher priority

# unit ordering
[x] implement ordering by drawing a line with the mouse
[x] make ordering by drawing a line with the mouse direct units to the closest point on the line they have, instead of the current arbitrary order
[x] implement orders that arrange the units as the group is currently aligned relative to eachother
[x] implement adding units to an existing selection (when holding shift)
[x] implement adding/removing units to an existing selection (when holding ctrl)
[] implement shift+double click on a unit to select all other units of that type

# movement
[] data-drive parameters for stuff like turn-rate?

# rts camera
[x] implement a simple camera that you can pan around with
[x] fix grouped movement, currently ends up crazily offset when units have moved away from the origin and a grouped move order is issued

# construction
[] when constructing buildings, show an eta in seconds/minutes/etc
[] when constructing buildings, make sure multiple constructors racing to build a single building end up helping to construct the same building, rather than building individual buildings.

# units
[x] represent health of units

# buildings
[x] implement construction
[x] represent health of buildings
[x] implement "ghosts" of buildings that are about to be built/queued for construction
[x] transition buildings from ghost state to constructed state when their health reaches 100 % after construction
[] implement buildings constructing buildings
[] implement buildings constructing units

# multiplayer
[] implement game state hashing for checking if in sync