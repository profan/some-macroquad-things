hoxx
-----
rough list of things that need to be done on hoxx, probably in order of priority?

# todos
[x] pick pleasing (contrasting) colours where possible
[x] flood-fill enclosed spaces when fully enclosed by a player
[x] show pending orders to place tiles on the client in the same colour as hover
[] render the game to texture (except the UI) to make zooming work more reasonably
[] implement a restriction so that the only place you can place your next hex is next to one of your existing hexes (unless you've not placed any hex yet in the world)
[] fix the on-the-wire format to be less deranged, sending all clients the entire game-state on every change is probably nuts (... unless it's not? can we make it not nuts?)