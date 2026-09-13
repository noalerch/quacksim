#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Grass {
    Bare { ticks_until_grown: u32 },
    Grown,
}

impl Default for Grass {
    fn default() -> Self {
        Self::Bare {
            ticks_until_grown: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Cell {
    pub grass: Grass,
}

impl Cell {
    #[must_use]
    pub const fn bare(ticks_until_grown: u32) -> Self {
        Self {
            grass: Grass::Bare { ticks_until_grown },
        }
    }

    #[must_use]
    pub const fn grown() -> Self {
        Self {
            grass: Grass::Grown,
        }
    }
}
