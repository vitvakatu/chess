use std::ops::Add;
use std::ops::Sub;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum File {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            File::A => write!(f, "a"),
            File::B => write!(f, "b"),
            File::C => write!(f, "c"),
            File::D => write!(f, "d"),
            File::E => write!(f, "e"),
            File::F => write!(f, "f"),
            File::G => write!(f, "g"),
            File::H => write!(f, "h"),
        }
    }
}

impl File {
    pub fn as_u8(self) -> u8 {
        use File::*;
        match self {
            A => 1,
            B => 2,
            C => 3,
            D => 4,
            E => 5,
            F => 6,
            G => 7,
            H => 8,
        }
    }

    pub fn try_from_u8(value: u8) -> Option<Self> {
        use File::*;
        match value {
            1 => Some(A),
            2 => Some(B),
            3 => Some(C),
            4 => Some(D),
            5 => Some(E),
            6 => Some(F),
            7 => Some(G),
            8 => Some(H),
            _ => None,
        }
    }

    pub fn from_u8(value: u8) -> Self {
        Self::try_from_u8(value).unwrap()
    }
}

impl PartialEq<u8> for File {
    fn eq(&self, other: &u8) -> bool {
        self.as_u8().eq(other)
    }
}

impl PartialOrd<u8> for File {
    fn partial_cmp(&self, other: &u8) -> Option<std::cmp::Ordering> {
        self.as_u8().partial_cmp(other)
    }
}

impl Add<i8> for File {
    type Output = Self;
    fn add(self, rhs: i8) -> Self::Output {
        assert!(rhs <= 7 && rhs >= -7, "Attempt to add {rhs} to {self:?}");
        let n = self.as_u8();
        let n = n as i8 + rhs;
        assert!(n >= 1 && n <= 8, "Overflow while adding {rhs} to {self:?}");
        Self::from_u8(n as u8)
    }
}

impl Sub<i8> for File {
    type Output = Self;
    fn sub(self, rhs: i8) -> Self::Output {
        assert!(rhs <= 7 && rhs >= -7, "Attempt to sub {rhs} from {self:?}");
        let n = self.as_u8();
        let n = n as i8 - rhs;
        assert!(
            n >= 1 && n <= 8,
            "Overflow while subbing {rhs} from {self:?}"
        );
        Self::from_u8(n as u8)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Rank(u8);

impl Rank {
    pub fn new(rank: u8) -> Self {
        Self::try_new(rank).unwrap()
    }

    pub fn try_new(rank: u8) -> Option<Self> {
        if rank >= 1 && rank <= 8 {
            Some(Self(rank))
        } else {
            None
        }
    }

    pub fn get(&self) -> u8 {
        self.0
    }

    pub fn map<F: Fn(u8) -> u8>(&self, f: F) -> Self {
        let new = f(self.0);
        Self::new(new)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnboundedPos {
    pub file: i8,
    pub rank: i8,
}

impl UnboundedPos {
    pub fn from_pos(pos: Pos) -> Self {
        Self {
            file: pos.file.as_u8() as i8,
            rank: pos.rank.get() as i8,
        }
    }

    pub fn up(&self, n: u8) -> Self {
        Self {
            rank: self.rank + n as i8,
            ..*self
        }
    }

    pub fn down(&self, n: u8) -> Self {
        Self {
            rank: self.rank - n as i8,
            ..*self
        }
    }

    pub fn left(&self, n: u8) -> Self {
        Self {
            file: self.file - n as i8,
            ..*self
        }
    }

    pub fn right(&self, n: u8) -> Self {
        Self {
            file: self.file + n as i8,
            ..*self
        }
    }

    pub fn to_pos(&self) -> Option<Pos> {
        if self.file < 0 || self.rank < 0 {
            return None;
        }
        let file = File::try_from_u8(self.file as u8)?;
        let rank = Rank::try_new(self.rank as u8)?;
        Some(Pos::new(file, rank))
    }

    pub fn horizontal(
        pos: Pos,
        n: u8,
        direction: HorizontalDirection,
    ) -> impl Iterator<Item = Self> {
        let origin = Self::from_pos(pos);
        let mut i = 0;
        std::iter::repeat_with(move || {
            i += 1;
            match direction {
                HorizontalDirection::Left => origin.left(i),
                HorizontalDirection::Right => origin.right(i),
            }
        })
        .take(n as usize)
    }

    pub fn vertical(pos: Pos, n: u8, direction: VerticalDirection) -> impl Iterator<Item = Self> {
        let origin = Self::from_pos(pos);
        let mut i = 0;
        std::iter::repeat_with(move || {
            i += 1;
            match direction {
                VerticalDirection::Up => origin.up(i),
                VerticalDirection::Down => origin.down(i),
            }
        })
        .take(n as usize)
    }

    pub fn diagonal(pos: Pos, n: u8, direction: DiagonalDirection) -> impl Iterator<Item = Self> {
        let origin = Self::from_pos(pos);
        let mut i = 0;
        std::iter::repeat_with(move || {
            i += 1;
            match direction {
                DiagonalDirection::UpLeft => origin.up(i).left(i),
                DiagonalDirection::UpRight => origin.up(i).right(i),
                DiagonalDirection::DownLeft => origin.down(i).left(i),
                DiagonalDirection::DownRight => origin.down(i).right(i),
            }
        })
        .take(n as usize)
    }
}

pub enum VerticalDirection {
    Up,
    Down,
}

pub enum HorizontalDirection {
    Left,
    Right,
}

pub enum DiagonalDirection {
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pos {
    pub file: File,
    pub rank: Rank,
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{file}{rank}", file=self.file, rank=self.rank.get())
    }
}

impl Pos {
    pub fn new(file: File, rank: Rank) -> Self {
        Self { file, rank }
    }

    pub fn up(&self, n: u8) -> Self {
        self.with_rank(|r| r.map(|r| r + n))
    }

    pub fn down(&self, n: u8) -> Self {
        self.with_rank(|r| r.map(|r| r - n))
    }

    pub fn left(&self, n: u8) -> Self {
        self.with_file(|f| f - 1)
    }

    pub fn right(&self, n: u8) -> Self {
        self.with_file(|f| f + 1)
    }

    pub fn with_rank<F>(&self, f: F) -> Self
    where
        F: FnOnce(Rank) -> Rank,
    {
        Self {
            rank: f(self.rank),
            file: self.file,
        }
    }

    pub fn with_file<F>(&self, f: F) -> Self
    where
        F: FnOnce(File) -> File,
    {
        Self {
            file: f(self.file),
            rank: self.rank,
        }
    }
}

