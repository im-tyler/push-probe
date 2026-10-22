//! The PC port's online play: a dedicated server in the role of EA's "RB" game servers, and
//! the game's client of it. Players exchange replay frames instead of the original's
//! lockstep pad input; the lobby, matchmaking and challenge flow follow `Sk8::Net`'s
//! messages. See `docs/online/`.

pub mod client;
pub mod codec;
pub mod crypto;
pub mod frames;
pub mod protocol;
pub mod range;
pub mod server;
pub mod transport;
pub mod words;

#[cfg(test)]
mod tests;
