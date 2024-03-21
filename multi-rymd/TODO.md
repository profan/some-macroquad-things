multi-rymd
--------------
rough list of things to work on in the game currently, usually ordered as items highest up = higher priority

# unit ordering
[x] implement ordering by drawing a line with the mouse
[x] make ordering by drawing a line with the mouse direct units to the closest point on the line they have, instead of the current arbitrary order
[x] implement orders that arrange the units as the group is currently aligned relative to eachother
[x] implement adding units to an existing selection (when holding shift)
[x] implement adding/removing units to an existing selection (when holding ctrl)
[x] make ordering by drawing a line distribute units evenly along the line (instead of per point as currently)
[x] have separate order queues for construction and movement/similar things for units that can both construct things and also move/forward movement orders
[x] implement a basic attack order (targets an entity)
[] implement a basic attack move order (targets a position, attacks all entities encountered along the way)
[] implement shift+double click on a unit to select all other units of that type
[] implement control groups (ctrl+1, etc) for units

# selection
[x] make selection aware of exact bounds when available, so that you can more precisely select units in construction

# attacking
[x] represent attacking and attackable entities
[x] have units automatically attack their nearest target if within target acquisition range

# movement
[] data-drive parameters for stuff like turn-rate?

# rts camera
[x] implement a simple camera that you can pan around with
[x] fix grouped movement, currently ends up crazily offset when units have moved away from the origin and a grouped move order is issued
[x] make the camera move towards where the mouse is when you zoom in
[] make the camera "center" when zooming towards where the mouse is, so that it ends up exactly where your mouse was when you started zooming in
[] make movement velocity based (ideally framerate independent)
[] make zooming velocity based (ideally framerate independent)
[x] make camera variables tweakable (currently hardcoded)
[x] make camera reset properly when restarting the game

# construction
[x] when constructing buildings, show an eta in seconds/minutes/etc
[] when constructing buildings, make sure multiple constructors racing to build a single building end up helping to construct the same building, rather than building individual buildings.
[x] when constructing entities, make the construction range ependent on the (circle) bounds of the item in construction, rather than just the centerpoint
[x] when constructing buildings with a unit, draw a construction beam
[x] when constructing units, move them out of the way automatically on construction (unless they already have an order)
[x] when constructing buildings, show the entire build queue visually (probably by walking over the build order queue)
[x] when constructing units, have the constructed unit inherit any (non-construction) orders the building may have
[x] when constructing entities, display the construction queue when that entity is selected
[] when entities are left not fully constructed, have them slowly decay (health wise)
[x] when entities are being constructed, show the time they have left until done
[] when entities are being constructed in a building, allow constructors to assist the building
[] fix issue where construction queue always displays the commander ship as the current item in construction, regardless of what the current item in construction actually is

# resources
[x] allow entities to generate resources
[x] allow entities to depend on resources to function
[x] allow entities to depend on resources to construct things
[x] allow entities to expand the size of the metal and energy storage pools
[x] cap the maximum amount of metal and energy by the current size of the energy/metal storage pools
[] fix issue where resource consumption for unit construction falls over when item being constructed has health lower than the build power of the unit involved

# units
[x] represent health of units
[] allow finding all idle units
[] allow finding all idle constructor units

# environment
[] add asteroids which are dynamic bodies

# collisions
[x] figure out why the bounds of some entities are completely wrong sometimes

# buildings
[x] implement construction
[x] represent health of buildings
[x] implement "ghosts" of buildings that are about to be built/queued for construction
[x] transition buildings from ghost state to constructed state when their health reaches 100 % after construction
[] implement buildings constructing buildings
[x] implement buildings constructing units

# multiplayer
[] implement game state hashing for checking if in sync

# view
[x] implement parallax background
[] look at implementing parallax background layers (different levels/depths of stars?)
[] implement sort orders for sprites, initially just hardcoded numbers per sprite
[] implement the ability to switch what side you're currently controlling