lockstep
-----------------------
this is an experimental platform for building lockstep networking based games ontop of, note it is very unfinished!


## structure
the project is structured into a server part and a client part, the server part is basically a relay which clients connect to and which handles forwarding messages through a central lobby (though clients can work in a 100% singleplayer mode as well, the multiplayer part is the whole reason this thing even exists).

the server has no idea what the game is, it just forwards data to the other clients in the lobby, in a sort of peer-to-peer fashion, which means the server can stay unchanged while an entirely new game is built ontop, ideally at least.

the client part is what contains the guts of the lockstep networking pieces and lobby handling, there's a very basic abstraction built which mostly makes it so that implementors of the "game" do not need to care that they are running in singleplayer or multiplayer, or that anything in the layers above even exist.

i wouldn't use this right now, or maybe ever, and expect it to change tons, it is an experiment for my own purposes!

as far as technical details go, the client and server communicate with eachother over a websocket connection.
this connection works regardless of if the client is running as a desktop application, or as a web application, the server does not particularly care either way.
