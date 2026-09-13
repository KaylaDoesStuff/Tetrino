use serde::{Deserialize, Serialize};

use super::piece::Piece;

// ---------------------------------------------------------------------------
// Delivery channel
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Reliability {
    Unreliable,
    Reliable,
}

pub fn delivery(packet_type: PacketType) -> Reliability {
    match packet_type {
        PacketType::Input | PacketType::StateSnapshot | PacketType::Ping => Reliability::Unreliable,
        PacketType::GarbageSend
        | PacketType::GarbageReceive
        | PacketType::GameStart
        | PacketType::GameOver
        | PacketType::PieceLock
        | PacketType::Chat
        | PacketType::SyncRequest => Reliability::Reliable,
    }
}

// ---------------------------------------------------------------------------
// Packet type discriminants
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum PacketType {
    Input = 0,
    StateSnapshot = 1,
    GarbageSend = 2,
    GarbageReceive = 3,
    GameStart = 4,
    GameOver = 5,
    PieceLock = 6,
    Chat = 7,
    Ping = 8,
    SyncRequest = 9,
}

// ---------------------------------------------------------------------------
// Top-level packet enum
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Packet {
    Input(InputPacket),
    StateSnapshot(StateSnapshot),
    GarbageSend(GarbagePacket),
    GarbageReceive(GarbagePacket),
    GameStart(GameStartPacket),
    GameOver(GameOverPacket),
    PieceLock(PieceLockPacket),
    Chat(ChatPacket),
    Ping(PingPacket),
    SyncRequest,
}

impl Packet {
    pub fn packet_type(&self) -> PacketType {
        match self {
            Packet::Input(_) => PacketType::Input,
            Packet::StateSnapshot(_) => PacketType::StateSnapshot,
            Packet::GarbageSend(_) => PacketType::GarbageSend,
            Packet::GarbageReceive(_) => PacketType::GarbageReceive,
            Packet::GameStart(_) => PacketType::GameStart,
            Packet::GameOver(_) => PacketType::GameOver,
            Packet::PieceLock(_) => PacketType::PieceLock,
            Packet::Chat(_) => PacketType::Chat,
            Packet::Ping(_) => PacketType::Ping,
            Packet::SyncRequest => PacketType::SyncRequest,
        }
    }
}

// ---------------------------------------------------------------------------
// Input — unreliable, sent every key event
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputAction {
    MoveLeft,
    MoveRight,
    SoftDropStart,
    SoftDropStop,
    HardDrop,
    RotateCW,
    RotateCCW,
    Rotate180,
    Hold,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputPacket {
    pub sequence: u16,
    pub action: InputAction,
    pub timestamp_ms: u64,
}

// ---------------------------------------------------------------------------
// State snapshot — unreliable, sent at a fixed tick rate for spectators/replay
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub tick: u64,
    pub grid: [[u8; 10]; 23],
    pub score: u64,
    pub lines: u32,
    pub level: i32,
    pub combo: i32,
    pub b2b: i32,
    pub active_piece: Option<u8>,
    pub active_rotation: u8,
    pub active_pos: (i32, i32),
    pub active_cells: [(i32, i32); 4],
    pub hold_piece: Option<u8>,
    pub hold_used: bool,
    pub queue: Vec<u8>,
    pub finesse_faults: u32,
    pub finesse_pct: f32,
    pub total_keys: u32,
}

// ---------------------------------------------------------------------------
// Garbage — reliable, must never be lost
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarbagePacket {
    pub lines: u32,
    pub source_piece: u8,
    pub sender_id: u8,
}

// ---------------------------------------------------------------------------
// Game start — reliable
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    Sprint40L,
    Blitz,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStartPacket {
    pub seed: u64,
    pub mode: GameMode,
    pub arr: f32,
    pub das: f32,
    pub dcd: f32,
    pub sdf: f32,
    pub infinite_sdf: bool,
}

// ---------------------------------------------------------------------------
// Game over — reliable
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameOverPacket {
    pub winner_id: u8,
    pub score: u64,
    pub lines: u32,
    pub pieces: u32,
    pub pps: f32,
    pub time_ms: u64,
    pub finesse_pct: f32,
}

// ---------------------------------------------------------------------------
// Piece lock event — reliable, for replay sync
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PieceLockPacket {
    pub piece: u8,
    pub rotation: u8,
    pub pos: (i32, i32),
    pub lines_cleared: u32,
    pub score_earned: u64,
    pub is_tspin: bool,
    pub is_mini: bool,
    pub combo: i32,
    pub b2b: i32,
}

// ---------------------------------------------------------------------------
// Chat — reliable
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatPacket {
    pub sender_id: u8,
    pub message: String,
}

// ---------------------------------------------------------------------------
// Ping — either channel depending on context
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingPacket {
    pub ping_id: u32,
    pub sent_ms: u64,
}

// ---------------------------------------------------------------------------
// Encoding / decoding — length-prefixed bincode frames
// ---------------------------------------------------------------------------

pub fn encode(packet: &Packet) -> Result<Vec<u8>, bincode::Error> {
    let payload = bincode::serialize(packet)?;
    let len = payload.len() as u16;
    let mut buf = Vec::with_capacity(2 + payload.len());
    buf.extend_from_slice(&len.to_le_bytes());
    buf.extend_from_slice(&payload);
    Ok(buf)
}

/// Try to decode one packet from the front of a byte buffer.
/// Returns `Ok(Some((packet, bytes_consumed)))` on success,
/// `Ok(None)` if the buffer doesn't contain a full frame yet,
/// `Err` on corruption.
pub fn decode(buf: &[u8]) -> Result<Option<(Packet, usize)>, bincode::Error> {
    if buf.len() < 2 {
        return Ok(None);
    }
    let len = u16::from_le_bytes([buf[0], buf[1]]) as usize;
    if buf.len() < 2 + len {
        return Ok(None);
    }
    let packet: Packet = bincode::deserialize(&buf[2..2 + len])?;
    Ok(Some((packet, 2 + len)))
}

/// Convenience: take as many complete packets as possible from a buffer,
/// returning the remaining unconsumed bytes.
pub fn decode_all(buf: &[u8]) -> (Vec<Packet>, Vec<u8>) {
    let mut packets = Vec::new();
    let mut cursor = buf;
    loop {
        match decode(cursor) {
            Ok(Some((packet, consumed))) => {
                packets.push(packet);
                cursor = &cursor[consumed..];
            }
            _ => break,
        }
    }
    (packets, cursor.to_vec())
}

// ---------------------------------------------------------------------------
// Helper: convert Piece to/from u8 for serialization
// ---------------------------------------------------------------------------

pub fn piece_to_u8(piece: Piece) -> u8 {
    match piece {
        Piece::I => 1,
        Piece::L => 2,
        Piece::J => 3,
        Piece::S => 4,
        Piece::Z => 5,
        Piece::T => 6,
        Piece::O => 7,
    }
}

pub fn piece_from_u8(id: u8) -> Option<Piece> {
    match id {
        1 => Some(Piece::I),
        2 => Some(Piece::L),
        3 => Some(Piece::J),
        4 => Some(Piece::S),
        5 => Some(Piece::Z),
        6 => Some(Piece::T),
        7 => Some(Piece::O),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Helpers to convert grid Color32 -> u8 and back
// ---------------------------------------------------------------------------

pub fn color_to_cell(color: egui::Color32) -> u8 {
    let [r, g, b, _a] = color.to_array();
    // 0 = empty, 1-7 map to piece colors by dominant channel
    // Simplified: just pack RGB into a single byte (high bits)
    if r == 0 && g == 0 && b == 0 {
        return 0;
    }
    // Map known piece colors to 1..=7
    let id = match (r, g, b) {
        (0, 240, 240) => 1,   // I
        (240, 160, 0) => 2,   // L
        (0, 0, 240) => 3,     // J
        (0, 240, 0) => 4,     // S
        (240, 0, 0) => 5,     // Z
        (160, 0, 240) => 6,   // T
        (240, 240, 0) => 7,   // O
        _ => 0,
    };
    id
}

pub fn cell_to_color(cell: u8) -> Option<egui::Color32> {
    match cell {
        0 => None,
        1 => Some(egui::Color32::from_rgb(0, 240, 240)),
        2 => Some(egui::Color32::from_rgb(240, 160, 0)),
        3 => Some(egui::Color32::from_rgb(0, 0, 240)),
        4 => Some(egui::Color32::from_rgb(0, 240, 0)),
        5 => Some(egui::Color32::from_rgb(240, 0, 0)),
        6 => Some(egui::Color32::from_rgb(160, 0, 240)),
        7 => Some(egui::Color32::from_rgb(240, 240, 0)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_input() {
        let pkt = Packet::Input(InputPacket {
            sequence: 42,
            action: InputAction::RotateCW,
            timestamp_ms: 12345,
        });
        let bytes = encode(&pkt).unwrap();
        let (decoded, rest) = decode_all(&bytes);
        assert_eq!(decoded.len(), 1);
        assert!(rest.is_empty());
        match &decoded[0] {
            Packet::Input(ip) => {
                assert_eq!(ip.sequence, 42);
                assert!(matches!(ip.action, InputAction::RotateCW));
                assert_eq!(ip.timestamp_ms, 12345);
            }
            _ => panic!("wrong packet type"),
        }
    }

    #[test]
    fn roundtrip_garbage() {
        let pkt = Packet::GarbageSend(GarbagePacket {
            lines: 4,
            source_piece: piece_to_u8(Piece::T),
            sender_id: 1,
        });
        let bytes = encode(&pkt).unwrap();
        let (decoded, _) = decode_all(&bytes);
        match &decoded[0] {
            Packet::GarbageSend(gp) => {
                assert_eq!(gp.lines, 4);
                assert_eq!(gp.source_piece, 6);
            }
            _ => panic!("wrong packet type"),
        }
    }

    #[test]
    fn multiple_packets_in_buffer() {
        let p1 = Packet::Input(InputPacket {
            sequence: 1,
            action: InputAction::MoveLeft,
            timestamp_ms: 100,
        });
        let p2 = Packet::Chat(ChatPacket {
            sender_id: 0,
            message: "hello".to_string(),
        });
        let mut buf = encode(&p1).unwrap();
        buf.extend_from_slice(&encode(&p2).unwrap());

        let (packets, rest) = decode_all(&buf);
        assert_eq!(packets.len(), 2);
        assert!(rest.is_empty());
    }

    #[test]
    fn partial_buffer_returns_none() {
        let pkt = Packet::Ping(PingPacket {
            ping_id: 1,
            sent_ms: 500,
        });
        let bytes = encode(&pkt).unwrap();
        // Only give first 3 bytes — not enough for a full frame
        let result = decode(&bytes[..3]).unwrap();
        assert!(result.is_none());
    }
}
