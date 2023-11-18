use std::collections::{HashMap};
use nanoserde::{SerJson, DeJson};

pub use i64 as PeerID;
pub use i32 as TurnID;

use crate::IS_DEBUGGING;

const TURN_DELAY: i32 = 1;
const TURN_LENGTH: i32 = 4;

trait HasTurnId {
    fn turn_id(&self) -> TurnID;
}

impl HasTurnId for TurnCommand {
    fn turn_id(&self) -> i32 {
        match self {
            TurnCommand::Command(turn_id, _) => *turn_id,
            TurnCommand::Pass(turn_id) => *turn_id
        }
    }
}

#[derive(Debug, Clone, SerJson, DeJson)]
pub enum TurnCommand {

    /// Sent to the game layer, representing a specific queued command that should be deserialized and executed.
    Command(TurnID, String),

    /// Sent only when the specific peer has nothing to do on a given turn.
    Pass(TurnID),

}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TurnState {
    Running,
    Waiting
}

#[derive(Debug)]
pub struct LockstepCommandQueue {
    commands_to_process: HashMap<TurnID, HashMap<PeerID, Vec<TurnCommand>>>,
    commands_to_send: HashMap<TurnID, Vec<TurnCommand>>
}

impl LockstepCommandQueue {
    
    pub fn new() -> LockstepCommandQueue {
        LockstepCommandQueue {
            commands_to_process: HashMap::new(),
            commands_to_send: HashMap::new()
        }
    }

    pub fn has_command_for_turn(&self, turn_id: TurnID, peer_id: PeerID) -> bool {
        if let Some(commands) = self.commands_to_process.get(&turn_id) && commands.contains_key(&peer_id) {
            true
        } else {
            false
        }
    }

    pub fn has_queued_command_for_turn(&self, turn_id: TurnID) -> bool {
        self.commands_to_send.contains_key(&turn_id)
    }
    
    pub fn commands_for_turn(&self, turn_id: TurnID) -> Option<&Vec<TurnCommand>> {
        if let Some(commands) = self.commands_to_send.get(&turn_id) {
            Some(commands)
        } else {
            None
        }
    }

    pub fn commands_for_turn_mut(&mut self, turn_id: TurnID) -> Option<&mut Vec<TurnCommand>> {
        if let Some(commands) = self.commands_to_send.get_mut(&turn_id) {
            Some(commands)
        } else {
            None
        }
    }

    pub fn commands_to_process_for_turn(&self, turn_id: TurnID) -> Option<&HashMap<PeerID, Vec<TurnCommand>>> {
        if let Some(commands) = self.commands_to_process.get(&turn_id) {
            Some(commands)
        } else {
            None
        }
    }

    pub fn remove_queued_commands_for_turn(&mut self, turn_id: TurnID) {
        self.commands_to_send.remove(&turn_id);
    }

    pub fn remove_commands_for_turn(&mut self, turn_id: TurnID) {
        self.commands_to_process.remove(&turn_id);
    }

    pub fn receive(&mut self, peer_id: PeerID, turn_id: TurnID, command: TurnCommand) {
        if let Some(commands) = self.commands_to_process.get_mut(&turn_id) {
            if let Some(peer_commands) = commands.get_mut(&peer_id) {
                peer_commands.push(command);
            } else {
                commands.insert(peer_id, vec![command]);
            }
        } else {
            let mut new_peer_map = HashMap::new();
            new_peer_map.insert(peer_id, vec![command]);
            self.commands_to_process.insert(turn_id, new_peer_map);
        }
    }

    pub fn send(&mut self, turn_id: TurnID, cmd: TurnCommand) {
        if let Some(commands) = self.commands_to_send.get_mut(&turn_id) {
            commands.push(cmd);
        } else {
            self.commands_to_send.insert(turn_id, vec![cmd]);
        }
    }

}

#[derive(Debug)]
pub struct LockstepPeer {
    pub id: PeerID
}

#[derive(Debug)]
pub struct LockstepClient {

    is_singleplayer: bool,

    peer_id: PeerID,
    peers: Vec<LockstepPeer>,
    command_queue: LockstepCommandQueue,
    turn_state: TurnState,

    turn_part: i32,
    turn_number: i32,
    turn_length: i32, // adjust this to make turn times longer!
    turn_delay: i32, // how many turns out should the message be sent by?

}

impl LockstepClient {

    pub fn new(peer_id: PeerID, is_singleplayer: bool) -> LockstepClient {
        LockstepClient {

            is_singleplayer: is_singleplayer,

            peer_id: peer_id,
            peers: Vec::new(),

            command_queue: LockstepCommandQueue::new(),
            turn_state: TurnState::Waiting,

            turn_part: 0,
            turn_number: -1,
            turn_length: TURN_LENGTH,
            turn_delay: TURN_DELAY, // in turns

        }
    }

    pub fn is_singleplayer(&self) -> bool {
        self.is_singleplayer
    }

    pub fn reset(&mut self) {
        self.turn_part = 0;
        self.turn_number = -1;
        self.turn_length = TURN_LENGTH;
        self.turn_delay = TURN_DELAY;
        self.turn_state = TurnState::Waiting;
    }

    pub fn peer_id(&self) -> PeerID {
        self.peer_id
    }

    pub fn peers(&self) -> &Vec<LockstepPeer> {
        &self.peers
    }

    pub fn turn_state(&self) -> TurnState {
        self.turn_state
    }

    pub fn turn_part(&self) -> i32 {
        self.turn_part
    }

    pub fn turn_number(&self) -> i32 {
        self.turn_number
    }

    pub fn turn_length(&self) -> i32 {
        self.turn_length
    }

    pub fn turn_delay(&self) -> i32 {
        self.turn_delay
    }
    
    pub fn current_send_turn_id(&self) -> TurnID {
        self.turn_number + self.turn_delay
    }

    pub fn has_peer_with_id(&self, id: PeerID) -> bool {
        for p in &self.peers {
            if p.id == id {
                return true;
            }
        }
        false
    }

    pub fn update_peers(&mut self, peers: &[PeerID]) {
        self.peers.clear();
        for &peer_id in peers {
            self.add_peer(peer_id);
        }
    }

    pub fn add_peer(&mut self, id: PeerID) {

        if self.has_peer_with_id(id) {
            panic!("[LockstepClient] should never be adding a peer when one already exists, tried adding peer: {} twice!", id);
        }

        let new_peer = LockstepPeer { id }; 
        self.peers.push(new_peer);
    }

    pub fn remove_peer(&mut self, id: PeerID) {
        // #FIXME: maybe clean up all the turn commands this peer would have been broadcasting?
        self.peers.retain(|p| p.id != id);
    }

    fn all_turns_received(&self, turn_id: i32) -> bool {

        let mut confirmed_peers = 0;
        for peer in &self.peers {

            let peer_has_command_for_turn = self.command_queue.has_command_for_turn(turn_id, peer.id);
            if peer.id != self.peer_id && peer_has_command_for_turn {
                confirmed_peers += 1;
            }

        }
        
        confirmed_peers == (self.peers.len().saturating_sub(1))

    }

    fn check_pass_turn(&mut self) {
        self.check_pass_turn_with_offset(0);
    }

    fn check_pass_turn_with_offset(&mut self, offset: i32) {
        let current_send_turn_id = self.current_send_turn_id() + offset;
        if self.command_queue.commands_to_process.contains_key(&current_send_turn_id) == false {
            self.command_queue.send(current_send_turn_id, TurnCommand::Pass(current_send_turn_id));
            if IS_DEBUGGING {
                println!("[LockstepClient] queued pass turn message for turn: {}", current_send_turn_id);
            }
        } else if let Some(commands) = self.command_queue.commands_to_process.get(&current_send_turn_id) {
            if commands.contains_key(&self.peer_id) == false {
                self.command_queue.send(current_send_turn_id, TurnCommand::Pass(current_send_turn_id));
            }
        } 
    }

    fn send_queued_commands_with_offset<F>(&mut self, mut send_command_fn: F, offset: i32) 
        where F: FnMut(PeerID, String) -> ()
    {
        let Some(commands_queued) = self.command_queue.commands_for_turn_mut(self.current_send_turn_id() + offset) else { return; };

        // #HACK: ugly clone, but it works!
        for command in &commands_queued.clone() {

            // send command to all our other peer friends
            send_command_fn(self.peer_id, command.serialize_json());

            // always enqueue our own commands locally, as they do not get sent back to us, or should not be at least
            self.command_queue.receive(self.peer_id, command.turn_id(), command.clone());

        }

        self.command_queue.remove_queued_commands_for_turn(self.current_send_turn_id());
    }

    fn send_turn_command(&mut self, command: TurnCommand) {
        self.send_turn_command_with_delay(command, 0);
    }

    fn send_turn_command_with_delay(&mut self, command: TurnCommand, delay: i32) {
        self.command_queue.send(self.current_send_turn_id() + delay, command);
    }

    pub fn send_command(&mut self, command: String) {
        let turn_command = TurnCommand::Command(self.current_send_turn_id() + self.turn_delay, command);
        self.send_turn_command_with_delay(turn_command, self.turn_delay);
    }

    pub fn handle_message(&mut self, peer_id: PeerID, command: &str) {

        let turn_command = match TurnCommand::deserialize_json(&command) {
            Ok(cmd) => cmd,
            Err(err) => {
                println!("[LockstepClient] got error: {} when processing command!", err);
                return;
            },
        };

        match turn_command {
            TurnCommand::Command(turn_id, _) => self.command_queue.receive(peer_id, turn_id, turn_command),
            TurnCommand::Pass(turn_id) => self.command_queue.receive(peer_id, turn_id, turn_command),
        };

    }

    fn execute_with<F>(&mut self, mut handle_command_fn: F)
        where F: FnMut(PeerID, &str) -> ()
    {

        if self.turn_number == -1 {
            return;
        }

        // execute all commands for the current turn
        if let Some(peer_commands) = self.command_queue.commands_to_process_for_turn(self.turn_number) {

            for (peer_id, commands) in peer_commands {
                for turn_command in commands {
                    if let TurnCommand::Command(_, command) = turn_command {
                        handle_command_fn(*peer_id, &command);
                    }
                }
            }

        } else {
            panic!("[LockstepClient] had no turns to process, this should never happen, turn was: {}", self.turn_number);
        }

        self.command_queue.remove_commands_for_turn(self.turn_number);
        
    }

    pub fn tick_with<F1, F2>(&mut self, handle_command_fn: F1, send_command_fn: F2) -> bool 
        where
            F1: FnMut(PeerID, &str) -> (),
            F2: FnMut(PeerID, String) -> ()
    {

        let mut state_changed = false;

        match self.turn_state {
            TurnState::Running => {

                // if we've reached the end of the turn, it's now time send our queued comands
                if self.turn_part == self.turn_length - 1 {

                    self.check_pass_turn_with_offset(TURN_DELAY);
                    self.send_queued_commands_with_offset(send_command_fn, TURN_DELAY);

                    if self.all_turns_received(self.current_send_turn_id()) {
                        // execute all the commands! ... or at least queue them to be executed
                        self.execute_with(handle_command_fn);
                        self.turn_number += 1;
                        self.turn_part = 0;
                    } else {
                        self.turn_state = TurnState::Waiting;
                        state_changed = true;
                    }

                } else {
                    self.turn_part += 1;
                }

            },
            TurnState::Waiting => {

                if self.turn_number == -1 {
                    self.send_turn_command(TurnCommand::Pass(self.current_send_turn_id()));
                    self.send_turn_command_with_delay(TurnCommand::Pass(self.current_send_turn_id() + self.turn_delay), self.turn_delay);
                    self.send_queued_commands_with_offset(send_command_fn, 0);
                }

                if self.all_turns_received(self.current_send_turn_id()) {
                    self.turn_state = TurnState::Running;
                    state_changed = true;
                }

            },
        };

        state_changed

    }

}