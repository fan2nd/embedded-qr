use crate::version::Version;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Errors that can occur while building a QR code.
pub enum QrError {
    /// The input could not be represented in a supported mode.
    DataInvalid,
    /// The encoded payload does not fit in the selected QR version.
    Overflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodeMode {
    Numeric,
    Alphanumeric,
    Bytes,
}

impl EncodeMode {
    pub const fn mode_bits(self) -> u16 {
        match self {
            Self::Numeric => 0b0001,
            Self::Alphanumeric => 0b0010,
            Self::Bytes => 0b0100,
        }
    }

    pub const fn counter_bits<T: Version>(self) -> usize {
        let version = T::VERSION;
        match self {
            Self::Numeric => {
                if version <= 9 {
                    10
                } else if version <= 26 {
                    12
                } else {
                    14
                }
            }
            Self::Alphanumeric => {
                if version <= 9 {
                    9
                } else if version <= 26 {
                    11
                } else {
                    13
                }
            }
            Self::Bytes => {
                if version <= 9 {
                    8
                } else {
                    16
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// QR Code error-correction level.
///
/// Higher levels recover better from damage, but reduce usable payload
/// capacity for a fixed version.
pub enum EccLevel {
    L = 0,
    M = 1,
    Q = 2,
    H = 3,
}

impl EccLevel {
    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn format_bits(self) -> u16 {
        match self {
            Self::L => 0b01,
            Self::M => 0b00,
            Self::Q => 0b11,
            Self::H => 0b10,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// One of the eight QR Code mask patterns.
pub enum Mask {
    M0,
    M1,
    M2,
    M3,
    M4,
    M5,
    M6,
    M7,
}

impl Mask {
    pub const ALL: [Self; 8] = [
        Self::M0,
        Self::M1,
        Self::M2,
        Self::M3,
        Self::M4,
        Self::M5,
        Self::M6,
        Self::M7,
    ];

    pub const fn index(self) -> usize {
        match self {
            Self::M0 => 0,
            Self::M1 => 1,
            Self::M2 => 2,
            Self::M3 => 3,
            Self::M4 => 4,
            Self::M5 => 5,
            Self::M6 => 6,
            Self::M7 => 7,
        }
    }

    pub const fn applies(self, x: usize, y: usize) -> bool {
        match self {
            Self::M0 => (x + y) % 2 == 0,
            Self::M1 => y % 2 == 0,
            Self::M2 => x % 3 == 0,
            Self::M3 => (x + y) % 3 == 0,
            Self::M4 => ((y / 2) + (x / 3)) % 2 == 0,
            Self::M5 => ((x * y) % 2) + ((x * y) % 3) == 0,
            Self::M6 => ((((x * y) % 2) + ((x * y) % 3)) % 2) == 0,
            Self::M7 => ((((x + y) % 2) + ((x * y) % 3)) % 2) == 0,
        }
    }

    pub(crate) fn format_bits(self, level: EccLevel) -> u16 {
        let data = (level.format_bits() << 3) | self.index() as u16;
        let mut remainder = data;
        for _ in 0..10 {
            remainder = (remainder << 1) ^ (((remainder >> 9) & 1) * 0x537);
        }
        ((data << 10) | remainder) ^ 0x5412
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DatawordsCapacity([usize; 4]);

impl DatawordsCapacity {
    const DATA_CAPACITY: [[usize; 4]; 40] = [
        [19, 16, 13, 9],
        [34, 28, 22, 16],
        [55, 44, 34, 26],
        [80, 64, 48, 36],
        [108, 86, 62, 46],
        [136, 108, 76, 60],
        [156, 124, 88, 66],
        [194, 154, 110, 86],
        [232, 182, 132, 100],
        [274, 216, 154, 122],
        [324, 254, 180, 140],
        [370, 290, 206, 158],
        [428, 334, 244, 180],
        [461, 365, 261, 197],
        [523, 415, 295, 223],
        [589, 453, 325, 253],
        [647, 507, 367, 283],
        [721, 563, 397, 313],
        [795, 627, 445, 341],
        [861, 669, 485, 385],
        [932, 714, 512, 406],
        [1006, 782, 568, 442],
        [1094, 860, 614, 464],
        [1174, 914, 664, 514],
        [1276, 1000, 718, 538],
        [1370, 1062, 754, 596],
        [1468, 1128, 808, 628],
        [1531, 1193, 871, 661],
        [1631, 1267, 911, 701],
        [1735, 1373, 985, 745],
        [1843, 1455, 1033, 793],
        [1955, 1541, 1115, 845],
        [2071, 1631, 1171, 901],
        [2191, 1725, 1231, 961],
        [2306, 1812, 1286, 986],
        [2434, 1914, 1354, 1054],
        [2566, 1992, 1426, 1096],
        [2702, 2102, 1502, 1142],
        [2812, 2216, 1582, 1222],
        [2956, 2334, 1666, 1276],
    ];

    pub const fn new(l: usize, m: usize, q: usize, h: usize) -> Self {
        Self([l, m, q, h])
    }

    pub const fn get_from_version<T: Version>() -> Self {
        Self(Self::DATA_CAPACITY[T::VERSION - 1])
    }

    pub const fn for_level(self, level: EccLevel) -> usize {
        self.0[level.index()]
    }
}

#[allow(dead_code)]
pub type DataCapaticy = DatawordsCapacity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataBlocks([usize; 4]);

impl DataBlocks {
    const DATA_BLOCKS: [[usize; 4]; 40] = [
        [1, 1, 1, 1],
        [1, 1, 1, 1],
        [1, 1, 2, 2],
        [1, 2, 2, 4],
        [1, 2, 4, 4],
        [2, 4, 4, 4],
        [2, 4, 6, 5],
        [2, 4, 6, 6],
        [2, 5, 8, 8],
        [4, 5, 8, 8],
        [4, 5, 8, 11],
        [4, 8, 10, 11],
        [4, 9, 12, 16],
        [4, 9, 16, 16],
        [6, 10, 12, 18],
        [6, 10, 17, 16],
        [6, 11, 16, 19],
        [6, 13, 18, 21],
        [7, 14, 21, 25],
        [8, 16, 20, 25],
        [8, 17, 23, 25],
        [9, 17, 23, 34],
        [9, 18, 25, 30],
        [10, 20, 27, 32],
        [12, 21, 29, 35],
        [12, 23, 34, 37],
        [12, 25, 34, 40],
        [13, 26, 35, 42],
        [14, 28, 38, 45],
        [15, 29, 40, 48],
        [16, 31, 43, 51],
        [17, 33, 45, 54],
        [18, 35, 48, 57],
        [19, 37, 51, 60],
        [19, 38, 53, 63],
        [20, 40, 56, 66],
        [21, 43, 59, 70],
        [22, 45, 62, 74],
        [24, 47, 65, 77],
        [25, 49, 68, 81],
    ];

    pub const fn new(l: usize, m: usize, q: usize, h: usize) -> Self {
        Self([l, m, q, h])
    }

    pub const fn get_from_version<T: Version>() -> Self {
        Self(Self::DATA_BLOCKS[T::VERSION - 1])
    }

    pub const fn for_level(self, level: EccLevel) -> usize {
        self.0[level.index()]
    }
}
