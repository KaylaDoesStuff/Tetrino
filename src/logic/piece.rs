use egui::Color32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Piece {
    I,
    L,
    J,
    S,
    Z,
    T,
    O,
}

impl Piece {
    pub fn from_id(id: i32) -> Option<Self> {
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

    pub fn color(self) -> Color32 {
        match self {
            Piece::I => Color32::from_rgb(0, 240, 240),
            Piece::L => Color32::from_rgb(240, 160, 0),
            Piece::J => Color32::from_rgb(0, 0, 240),
            Piece::S => Color32::from_rgb(0, 240, 0),
            Piece::Z => Color32::from_rgb(240, 0, 0),
            Piece::T => Color32::from_rgb(160, 0, 240),
            Piece::O => Color32::from_rgb(240, 240, 0),
        }
    }

    pub fn spawn_cells(self) -> [(i32, i32); 4] {
        match self {
            Piece::I => [(1, 0), (1, 1), (1, 2), (1, 3)],
            Piece::T => [(0, 1), (1, 0), (1, 1), (1, 2)],
            Piece::J => [(0, 0), (1, 0), (1, 1), (1, 2)],
            Piece::L => [(0, 2), (1, 0), (1, 1), (1, 2)],
            Piece::S => [(0, 1), (0, 2), (1, 0), (1, 1)],
            Piece::Z => [(0, 0), (0, 1), (1, 1), (1, 2)],
            Piece::O => [(0, 1), (0, 2), (1, 1), (1, 2)],
        }
    }

    pub fn rotate_cw(cells: &[(i32, i32); 4], piece: Piece) -> [(i32, i32); 4] {
        match piece {
            Piece::O => *cells,
            Piece::I => {
                let mut out = [(0i32, 0i32); 4];
                for (i, &(r, c)) in cells.iter().enumerate() {
                    out[i] = (c, 3 - r);
                }
                out
            }
            _ => {
                let mut out = [(0i32, 0i32); 4];
                for (i, &(r, c)) in cells.iter().enumerate() {
                    out[i] = (c, 2 - r);
                }
                out
            }
        }
    }

    pub fn rotate_ccw(cells: &[(i32, i32); 4], piece: Piece) -> [(i32, i32); 4] {
        match piece {
            Piece::O => *cells,
            Piece::I => {
                let mut out = [(0i32, 0i32); 4];
                for (i, &(r, c)) in cells.iter().enumerate() {
                    out[i] = (3 - c, r);
                }
                out
            }
            _ => {
                let mut out = [(0i32, 0i32); 4];
                for (i, &(r, c)) in cells.iter().enumerate() {
                    out[i] = (2 - c, r);
                }
                out
            }
        }
    }

    pub fn flip_180(cells: &[(i32, i32); 4], piece: Piece) -> [(i32, i32); 4] {
        match piece {
            Piece::O => *cells,
            Piece::I => {
                let mut out = [(0i32, 0i32); 4];
                for (i, &(r, c)) in cells.iter().enumerate() {
                    out[i] = (3 - r, 3 - c);
                }
                out
            }
            _ => {
                let mut out = [(0i32, 0i32); 4];
                for (i, &(r, c)) in cells.iter().enumerate() {
                    out[i] = (2 - r, 2 - c);
                }
                out
            }
        }
    }

    pub fn get_kick_offsets(piece: Piece, from: u8, to: u8) -> [(i32, i32); 5] {
        match piece {
            Piece::O => [(0, 0); 5],
            Piece::I => match (from, to) {
                (0, 1) => [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
                (1, 0) => [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
                (1, 2) => [(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
                (2, 1) => [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
                (2, 3) => [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
                (3, 2) => [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
                (3, 0) => [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
                (0, 3) => [(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
                _ => [(0, 0); 5],
            },
            _ => match (from, to) {
                (0, 1) => [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
                (1, 0) => [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
                (1, 2) => [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
                (2, 1) => [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
                (2, 3) => [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
                (3, 2) => [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
                (3, 0) => [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
                (0, 3) => [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
                _ => [(0, 0); 5],
            },
        }
    }
}
