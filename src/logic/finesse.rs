use super::piece::Piece;

type FinTable = [Option<usize>; 10];

/*
 * Finesse lookup tables.
 *
 * The value is the minimum number of discrete player inputs required
 * to reach the given (active_pos.x, rotation) state from the standard
 * spawn state:
 *
 *     active_pos.x = 3
 *     rotation     = 0
 *
 * Input semantics match GameState:
 *
 *   - Left/right key press = 1 input
 *   - CW/CCW rotation     = 1 input
 *   - 180 rotation         = 1 input
 *   - DAS movement         = 1 input
 *   - ARR repeats          = 0 additional finesse inputs
 *
 * SRS kicks are taken into account.
 *
 * Index = active_pos.1
 *
 * NOTE:
 * active_pos.1 is actually capable of becoming negative for some
 * wall placements. Since lookup_finesse() currently accepts usize,
 * those states cannot be represented here.
 */

// -----------------------------------------------------------------------------
// O
// -----------------------------------------------------------------------------

static O_ROT0: FinTable = [
    Some(2),
    Some(2),
    Some(1),
    Some(0),
    Some(1),
    Some(2),
    Some(2),
    Some(1),
    None,
    None,
];

// O does not rotate in this engine.

// -----------------------------------------------------------------------------
// I
// -----------------------------------------------------------------------------

static I_ROT0: FinTable = [
    Some(1),
    Some(2),
    Some(1),
    Some(0),
    Some(1),
    Some(2),
    Some(1),
    None,
    None,
    None,
];

static I_ROT1: FinTable = [
    Some(2),
    Some(3),
    Some(2),
    Some(1),
    Some(2),
    Some(3),
    Some(2),
    Some(2),
    None,
    None,
];

static I_ROT2: FinTable = [
    Some(2),
    Some(3),
    Some(2),
    Some(1),
    Some(2),
    Some(3),
    Some(2),
    None,
    None,
    None,
];

static I_ROT3: FinTable = [
    Some(2),
    Some(3),
    Some(2),
    Some(1),
    Some(2),
    Some(3),
    Some(2),
    Some(3),
    Some(2),
    None,
];

// -----------------------------------------------------------------------------
// T / J / L
// -----------------------------------------------------------------------------

static TJL_ROT0: FinTable = [
    Some(1),
    Some(2),
    Some(1),
    Some(0),
    Some(1),
    Some(2),
    Some(2),
    Some(1),
    None,
    None,
];

static TJL_ROT1: FinTable = [
    Some(2),
    Some(3),
    Some(2),
    Some(1),
    Some(2),
    Some(3),
    Some(3),
    Some(2),
    None,
    None,
];

static TJL_ROT2: FinTable = [
    Some(2),
    Some(3),
    Some(2),
    Some(1),
    Some(2),
    Some(3),
    Some(3),
    Some(2),
    None,
    None,
];

static TJL_ROT3: FinTable = [
    Some(2),
    Some(3),
    Some(2),
    Some(1),
    Some(2),
    Some(3),
    Some(3),
    Some(2),
    Some(2),
    None,
];

// -----------------------------------------------------------------------------
// S
// -----------------------------------------------------------------------------

static S_ROT0: FinTable = [
    Some(1),
    Some(2),
    Some(1),
    Some(0),
    Some(1),
    Some(2),
    Some(2),
    Some(1),
    None,
    None,
];

static S_ROT1: FinTable = [
    Some(2),
    Some(3),
    Some(2),
    Some(1),
    Some(2),
    Some(3),
    Some(3),
    Some(2),
    None,
    None,
];

static S_ROT2: FinTable = [
    Some(2),
    Some(3),
    Some(2),
    Some(1),
    Some(2),
    Some(3),
    Some(3),
    Some(2),
    None,
    None,
];

static S_ROT3: FinTable = [
    Some(2),
    Some(3),
    Some(2),
    Some(1),
    Some(2),
    Some(3),
    Some(3),
    Some(2),
    Some(2),
    None,
];

// -----------------------------------------------------------------------------
// Z
// -----------------------------------------------------------------------------

static Z_ROT0: FinTable = [
    Some(1),
    Some(2),
    Some(1),
    Some(0),
    Some(1),
    Some(2),
    Some(2),
    Some(1),
    None,
    None,
];

static Z_ROT1: FinTable = [
    Some(2),
    Some(3),
    Some(2),
    Some(1),
    Some(2),
    Some(3),
    Some(3),
    Some(2),
    None,
    None,
];

static Z_ROT2: FinTable = [
    Some(2),
    Some(3),
    Some(2),
    Some(1),
    Some(2),
    Some(3),
    Some(3),
    Some(2),
    None,
    None,
];

static Z_ROT3: FinTable = [
    Some(2),
    Some(3),
    Some(2),
    Some(1),
    Some(2),
    Some(3),
    Some(3),
    Some(2),
    Some(2),
    None,
];

// -----------------------------------------------------------------------------
// Lookup
// -----------------------------------------------------------------------------

fn get_table(piece: Piece, rotation: u8) -> &'static FinTable {
    match piece {
        Piece::O => &O_ROT0,

        Piece::I => match rotation & 3 {
            0 => &I_ROT0,
            1 => &I_ROT1,
            2 => &I_ROT2,
            3 => &I_ROT3,
            _ => unreachable!(),
        },

        Piece::T | Piece::J | Piece::L => match rotation & 3 {
            0 => &TJL_ROT0,
            1 => &TJL_ROT1,
            2 => &TJL_ROT2,
            3 => &TJL_ROT3,
            _ => unreachable!(),
        },

        Piece::S => match rotation & 3 {
            0 => &S_ROT0,
            1 => &S_ROT1,
            2 => &S_ROT2,
            3 => &S_ROT3,
            _ => unreachable!(),
        },

        Piece::Z => match rotation & 3 {
            0 => &Z_ROT0,
            1 => &Z_ROT1,
            2 => &Z_ROT2,
            3 => &Z_ROT3,
            _ => unreachable!(),
        },
    }
}

pub fn lookup_finesse(
    piece: Piece,
    target_col: usize,
    target_rotation: u8,
) -> Option<usize> {
    if target_col >= 10 {
        return None;
    }

    get_table(piece, target_rotation)[target_col]
}
