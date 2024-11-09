
use nanoserde::{DeJson, SerJson};

pub use i32 as LobbyID;
pub use i64 as LobbyClientID;
pub const DEFAULT_LOBBY_PORT: u16 = 4302;

#[derive(Debug, Clone, SerJson, DeJson)]
pub enum RelayMessage {

    /// Registers with the server with a specific nickname.
    Register(String),

    /// Represents a response from the server telling the client what their id is.
    ClientID(LobbyClientID),

    /// Represents a desire for creation of a new lobby in the relay server.
    CreateLobby(String),

    /// Represents a desire for starting the lobby, and thus closing it to new connections (while leaving it open).
    StartLobby,

    /// Represents a desire to stop the running lobby, and thus transitioning its state back to open.
    StopLobby,

    /// Represents a desire to close the lobby, disconnecting all clients currently in it.
    CloseLobby,

    /// Represents a desire to join a specfic lobby in the relay server.
    JoinLobby(LobbyID),

    /// Represents a server response telling the client they've joined a lobby.
    SuccessfullyJoinedLobby(LobbyID),

    /// Represents a response telling clients in a specific lobby that the specific client has joined.
    JoinedLobby(LobbyClientID),

    /// Represents a response telling clients a specific lobby has been updated (or possibly created).
    UpdatedLobby(Lobby),

    /// Represents a response telling clients the lobby they are in has been started.
    StartedLobby,

    /// Represents a response telling clients the lobby they are in has been stopped.
    StoppedLobby,

    /// Represents a request from a client to another client, for them to send them a pong message back, can be a server ping if no target client is specified.
    Ping(LobbyClientID, Option<LobbyClientID>),

    /// Represents a response from a client to another client's ping message, can be a server ping if no source client is specified.
    Pong(Option<LobbyClientID>, LobbyClientID),

    /// Represents a request from a client to update the current lobby data, will be distributed to all other clients and also to any clients who join the lobby.
    PushLobbyData(LobbyClientID, String),

    /// Represents a response telling clients in a specific lobby that the specific client has left.
    LeftLobby(LobbyClientID),

    /// Represents a response telling the client they've failed to join a lobby for some reason.
    FailedToJoinLobby(LobbyID, String),

    /// Represents a desire to leave the current lobby the sending client is connected to.
    LeaveLobby,

    /// Represents a payload that should be sent through the current active lobby to all other players in the lobby.
    Message(LobbyClientID, String),

    /// Represents a request to get all the active lobbies on the relay server.
    QueryActiveLobbies,

    /// Represents a response with all the active lobbies on the relay server.
    ActiveLobbies(Vec<Lobby>),

    /// Represents a request to get all the active players on the relay server.
    QueryActivePlayers,

    /// Represents a response with all the active players on the relay server.
    ActivePlayers(Vec<LobbyClient>),

}

#[derive(Debug, Clone, Copy, SerJson, DeJson, PartialEq)]
pub enum LobbyState {

    /// If the lobby is currently accepting clients, it is in the open state.
    Open,

    /// If the lobby is currently running its session, it is in the running state.
    Running,

}

#[derive(Debug, Clone, SerJson, DeJson)]
pub struct LobbyClient {
    pub id: LobbyClientID,
    pub name: String
}

#[derive(Debug, Clone, SerJson, DeJson)]
pub struct Lobby {
    pub id: LobbyID,
    pub name: String,
    pub clients: Vec<LobbyClientID>,
    pub state: LobbyState,
    pub data: String
}

impl Lobby {
    pub fn new(id: LobbyID, name: String) -> Lobby {
        Lobby {
            id: id,
            name: name,
            clients: Vec::new(),
            state: LobbyState::Open,
            data: String::new()
        }
    }
}