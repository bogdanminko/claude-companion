//! Clawd, the Claude Code mascot, block for block from the CLI banner (▐▛███▜▌ / ▝▜█████▛▘ / ▘▘ ▝▝).
//! One terminal quadrant is twice as tall as wide, so each banner row is two square pixels: 18×10.

use crate::canvas::Rgba;

pub const W: usize = 18;
pub const H: usize = 14;
pub const TOP: usize = 4; // rows above the head, for the fiesta sombrero

pub const BODY: Rgba = Rgba::hex(0xD97757);
pub const LIGHT: Rgba = Rgba::hex(0xEFA288); // blush
pub const EYE: Rgba = Rgba::hex(0x2B1B17);
pub const STRAW: Rgba = Rgba::hex(0xE8C15A); // sombrero
pub const STRAW_DARK: Rgba = Rgba::hex(0xB8902F);
pub const BAND: Rgba = Rgba::hex(0xC0392B);
pub const WOOD: Rgba = Rgba::hex(0x8B4A28); // guitar
pub const NECK: Rgba = Rgba::hex(0x4A2812);

const LEG_X: [i32; 4] = [4, 6, 11, 13];
/// Leg phase with all four legs down (icons). Any real phase always has one leg lifted.
pub const STANDING: f64 = f64::NAN;

#[derive(Clone, Copy, PartialEq)]
pub enum Eyes {
    Open { look_up: bool },
    Closed,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Screen {
    Prompt,
    Thinking { active: i32 },
    Listening { tick: i32 },
    Off,
}

pub type Grid = [[Option<Rgba>; W]; H];

/// `screen` drives the body language: thinking — eyes look around, listening — arms flap.
/// `fiesta`: sombrero + guitar; its value is the strum frame (odd = strumming hand down).
pub fn render(phase: f64, eyes: Eyes, blush: bool, screen: Screen, fiesta: Option<i32>) -> Grid {
    let mut g: Grid = [[None; W]; H];
    let mut put = |x: i32, y: i32, c: Option<Rgba>| {
        let row = y + TOP as i32;
        if x >= 0 && row >= 0 && (x as usize) < W && (row as usize) < H {
            g[row as usize][x as usize] = c;
        }
    };

    for y in 0..=7 {
        for x in 3..=14 {
            put(x, y, Some(BODY)); // body
        }
    }

    let mut arm_top = 4;
    if let Screen::Listening { tick } = screen {
        if (tick / 3) % 2 == 1 {
            arm_top = 3;
        }
    }
    for y in arm_top..=arm_top + 1 {
        for x in [1, 2, 15, 16] {
            put(x, y, Some(BODY));
        }
    }

    for (i, &lx) in LEG_X.iter().enumerate() {
        let lifted = (phase + i as f64 * 1.7).sin() > 0.6; // lift in turn
        put(lx, 8, Some(BODY));
        if !lifted {
            put(lx, 9, Some(BODY));
        }
    }

    let look = if let Screen::Thinking { active } = screen { active - 1 } else { 0 };
    for ex in [5, 12] {
        match eyes {
            Eyes::Open { look_up } => {
                put(ex + look, if look_up { 1 } else { 2 }, Some(EYE));
                put(ex + look, if look_up { 2 } else { 3 }, Some(EYE));
            }
            Eyes::Closed => put(ex, 3, Some(EYE)),
        }
    }

    if blush {
        put(4, 5, Some(LIGHT));
        put(13, 5, Some(LIGHT));
    }

    let Some(fiesta) = fiesta else { return g };

    // sombrero: crown, red band, wide brim with upturned tips
    for y in -4..=-3 {
        for x in 6..=11 {
            put(x, y, Some(STRAW));
        }
    }
    for x in 6..=11 {
        put(x, -2, Some(BAND));
    }
    for x in 1..=16 {
        put(x, -1, Some(STRAW));
    }
    put(0, -2, Some(STRAW));
    put(17, -2, Some(STRAW));
    for x in 3..=14 {
        put(x, 0, Some(STRAW_DARK)); // shadow on the forehead
    }

    // guitar across the belly, neck in the left hand
    for y in 4..=7 {
        for x in 9..=13 {
            if !((x == 9 || x == 13) && (y == 4 || y == 7)) {
                put(x, y, Some(WOOD));
            }
        }
    }
    put(11, 5, Some(NECK)); // sound hole
    put(11, 6, Some(NECK));
    for x in 2..=8 {
        put(x, 5, Some(NECK));
    }
    put(1, 4, Some(NECK)); // headstock
    if fiesta % 2 == 1 {
        // strumming hand drops onto the strings
        for x in 15..=16 {
            put(x, arm_top, None);
            put(x, arm_top + 2, Some(BODY));
        }
    }
    g
}

/// Clawd standing still, without the empty sombrero rows: for icons.
pub fn standing() -> Vec<[Option<Rgba>; W]> {
    render(STANDING, Eyes::Open { look_up: false }, false, Screen::Off, None)[TOP..].to_vec()
}
