use hoxx_shared::{ClientColor, ClientID, GameState};

pub struct HoxxGameState {
    state: GameState
}

impl HoxxGameState {
    pub fn new() -> HoxxGameState {
        HoxxGameState {
            state: GameState::new()
        }
    }

    pub fn get_game_state(&self) -> &GameState {
        &self.state
    }

    pub fn get_client_colour(&self, client_id: ClientID) -> Option<ClientColor> {
        self.state.get_client_colour(client_id)
    }

    pub fn set_client_colour(&mut self, client_id: ClientID, client_colour: ClientColor) {
        self.state.set_client_colour(client_id, client_colour);
    }

    pub fn get_claim_hex(&mut self, x: i32, y: i32) -> Option<ClientID> {
        self.state.get_hex(x, y).and_then(|id| Some(ClientID { id }))
    }

    pub fn put_claim_hex(&mut self, x: i32, y: i32, v: ClientID) {
        self.state.set_hex(x, y, v.id);
    }

    pub fn put_claim_world(&mut self, x: i32, y: i32, v: ClientID) {
        self.state.set_world(x, y, v.id);
    }

    pub fn get_claim_world(&mut self, x: i32, y: i32) -> Option<ClientID> {
        if let Some(id) = self.state.get_world(x, y) {
            Some(ClientID { id })
        } else {
            None
        }
    }
}