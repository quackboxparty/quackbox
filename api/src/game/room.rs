//! The room actor — one owning tokio task per live game.
//!
//! `spawn_room()` creates the channels and spawns the task; returns a
//! `RoomHandle { cmd_tx, state_tx }` for the registry. The task loop is a
//! `tokio::select!` racing: next `Command` (mpsc), the timer deadline
//! (`sleep_until`), and shutdown — mutate state, broadcast a snapshot, loop.
//! Sole owner of `GameState`, so mutation needs no lock; first-buzz-wins falls
//! out of mpsc ordering. On exit, removes its own entry from the registry.
//!
//! v1: grid_quiz buzz/lockout/timer/scoring policy lives directly in this loop.
//!
//! TODO: RoomHandle, spawn_room, the select! loop.

use tokio::sync::{broadcast, mpsc};

use crate::{
    protocol::{
        ClientView, Command,
        ServerMessage::{self, Snapshot},
    },
    state::{JoinCode, RoomHandle},
};

pub fn spawn_room(code: JoinCode) -> RoomHandle {
    let (command_tx, mut command_rx) = mpsc::channel::<Command>(64);
    let (server_tx, _) = broadcast::channel::<ServerMessage>(16);

    let server_tx_loop = server_tx.clone();
    tokio::spawn(async move {
        let mut state = ClientView {
            players: Vec::new(),
        };
        while let Some(command) = command_rx.recv().await {
            tracing::info!("command {:?} received in room {}", command, &code.0);
            match command {
                Command::Join { name } => {
                    state.players.push(name);
                }
                _ => {
                    todo!()
                }
            }
            let _ = server_tx_loop.send(Snapshot(state.clone()));
        }
    });

    RoomHandle {
        command_tx,
        server_tx,
    }
}
