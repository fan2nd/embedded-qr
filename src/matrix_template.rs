#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionMatrixTemplates<M> {
    pub(crate) modules: M,
    pub(crate) reserved: M,
}

pub(crate) const fn build_version_matrix_templates<
    const VERSION: usize,
    const WIDTH: usize,
    const BYTES: usize,
>() -> VersionMatrixTemplates<[u8; BYTES]> {
    let mut modules = [0u8; BYTES];
    let mut reserved = [0u8; BYTES];
    draw_function_patterns::<VERSION, WIDTH, BYTES>(&mut modules, &mut reserved);
    VersionMatrixTemplates { modules, reserved }
}

const fn draw_function_patterns<const VERSION: usize, const WIDTH: usize, const BYTES: usize>(
    modules: &mut [u8; BYTES],
    reserved: &mut [u8; BYTES],
) {
    draw_finder_pattern::<WIDTH, BYTES>(modules, reserved, 0, 0);
    draw_finder_pattern::<WIDTH, BYTES>(modules, reserved, WIDTH - 7, 0);
    draw_finder_pattern::<WIDTH, BYTES>(modules, reserved, 0, WIDTH - 7);
    draw_alignment_patterns::<VERSION, WIDTH, BYTES>(modules, reserved);
    draw_timing_patterns::<WIDTH, BYTES>(modules, reserved);
    reserve_format_areas::<WIDTH, BYTES>(modules, reserved);
    draw_version_info::<VERSION, WIDTH, BYTES>(modules, reserved);
    set_function_module::<WIDTH, BYTES>(modules, reserved, 8, WIDTH - 8, true);
}

const fn draw_finder_pattern<const WIDTH: usize, const BYTES: usize>(
    modules: &mut [u8; BYTES],
    reserved: &mut [u8; BYTES],
    origin_x: usize,
    origin_y: usize,
) {
    let mut dy = -1isize;
    while dy <= 7 {
        let mut dx = -1isize;
        while dx <= 7 {
            let xx = origin_x as isize + dx;
            let yy = origin_y as isize + dy;
            if xx >= 0 && yy >= 0 && xx < WIDTH as isize && yy < WIDTH as isize {
                let on_pattern = dx >= 0 && dx <= 6 && dy >= 0 && dy <= 6;
                let inner_square = dx >= 2 && dx <= 4 && dy >= 2 && dy <= 4;
                let is_black =
                    on_pattern && (dx == 0 || dx == 6 || dy == 0 || dy == 6 || inner_square);
                set_function_module::<WIDTH, BYTES>(
                    modules,
                    reserved,
                    xx as usize,
                    yy as usize,
                    is_black,
                );
            }
            dx += 1;
        }
        dy += 1;
    }
}

const fn draw_timing_patterns<const WIDTH: usize, const BYTES: usize>(
    modules: &mut [u8; BYTES],
    reserved: &mut [u8; BYTES],
) {
    let mut index = 8usize;
    while index < WIDTH - 8 {
        let is_black = index % 2 == 0;
        set_function_module::<WIDTH, BYTES>(modules, reserved, index, 6, is_black);
        set_function_module::<WIDTH, BYTES>(modules, reserved, 6, index, is_black);
        index += 1;
    }
}

const fn draw_alignment_patterns<const VERSION: usize, const WIDTH: usize, const BYTES: usize>(
    modules: &mut [u8; BYTES],
    reserved: &mut [u8; BYTES],
) {
    let (positions, count) = alignment_pattern_positions::<VERSION, WIDTH>();
    if count == 0 {
        return;
    }

    let mut y_index = 0usize;
    while y_index < count {
        let mut x_index = 0usize;
        while x_index < count {
            if !((x_index == 0 && y_index == 0)
                || (x_index == 0 && y_index == count - 1)
                || (x_index == count - 1 && y_index == 0))
            {
                draw_alignment_pattern::<WIDTH, BYTES>(
                    modules,
                    reserved,
                    positions[x_index],
                    positions[y_index],
                );
            }
            x_index += 1;
        }
        y_index += 1;
    }
}

const fn draw_alignment_pattern<const WIDTH: usize, const BYTES: usize>(
    modules: &mut [u8; BYTES],
    reserved: &mut [u8; BYTES],
    center_x: usize,
    center_y: usize,
) {
    let mut dy = -2isize;
    while dy <= 2 {
        let mut dx = -2isize;
        while dx <= 2 {
            let x = (center_x as isize + dx) as usize;
            let y = (center_y as isize + dy) as usize;
            let is_black = isize_abs(dx) == 2 || isize_abs(dy) == 2 || (dx == 0 && dy == 0);
            set_function_module::<WIDTH, BYTES>(modules, reserved, x, y, is_black);
            dx += 1;
        }
        dy += 1;
    }
}

const fn reserve_format_areas<const WIDTH: usize, const BYTES: usize>(
    modules: &mut [u8; BYTES],
    reserved: &mut [u8; BYTES],
) {
    let mut index = 0usize;
    while index <= 5 {
        set_function_module::<WIDTH, BYTES>(modules, reserved, 8, index, false);
        set_function_module::<WIDTH, BYTES>(modules, reserved, index, 8, false);
        index += 1;
    }

    set_function_module::<WIDTH, BYTES>(modules, reserved, 8, 7, false);
    set_function_module::<WIDTH, BYTES>(modules, reserved, 8, 8, false);
    set_function_module::<WIDTH, BYTES>(modules, reserved, 7, 8, false);

    index = 0;
    while index <= 7 {
        set_function_module::<WIDTH, BYTES>(modules, reserved, WIDTH - 1 - index, 8, false);
        index += 1;
    }

    index = 0;
    while index <= 6 {
        set_function_module::<WIDTH, BYTES>(modules, reserved, 8, WIDTH - 1 - index, false);
        index += 1;
    }
}

const fn draw_version_info<const VERSION: usize, const WIDTH: usize, const BYTES: usize>(
    modules: &mut [u8; BYTES],
    reserved: &mut [u8; BYTES],
) {
    if VERSION < 7 {
        return;
    }

    let bits = version_bits::<VERSION>();
    let mut index = 0usize;
    while index < 18 {
        let bit = ((bits >> index) & 1) != 0;
        let x = WIDTH - 11 + (index % 3);
        let y = index / 3;
        set_function_module::<WIDTH, BYTES>(modules, reserved, x, y, bit);
        set_function_module::<WIDTH, BYTES>(modules, reserved, y, x, bit);
        index += 1;
    }
}

pub(crate) const fn alignment_pattern_positions<const VERSION: usize, const WIDTH: usize>()
-> ([usize; 7], usize) {
    if VERSION == 1 {
        return ([0; 7], 0);
    }

    let count = (VERSION / 7) + 2;
    let step = if VERSION == 32 {
        26
    } else {
        (((VERSION * 4) + (count * 2) + 1) / ((count * 2) - 2)) * 2
    };
    let mut positions = [0usize; 7];
    positions[0] = 6;
    let mut pos = WIDTH - 7;
    let mut index = count - 1;

    while index > 0 {
        positions[index] = pos;
        if index == 1 {
            break;
        }
        pos -= step;
        index -= 1;
    }

    (positions, count)
}

pub(crate) const fn version_bits<const VERSION: usize>() -> u32 {
    let mut remainder = VERSION as u32;
    let mut index = 0usize;
    while index < 12 {
        remainder = (remainder << 1) ^ (((remainder >> 11) & 1) * 0x1F25);
        index += 1;
    }
    ((VERSION as u32) << 12) | remainder
}

const fn set_function_module<const WIDTH: usize, const BYTES: usize>(
    modules: &mut [u8; BYTES],
    reserved: &mut [u8; BYTES],
    x: usize,
    y: usize,
    is_black: bool,
) {
    set_module::<WIDTH, BYTES>(modules, x, y, is_black);
    set_bit::<BYTES>(reserved, y * WIDTH + x, true);
}

const fn set_module<const WIDTH: usize, const BYTES: usize>(
    modules: &mut [u8; BYTES],
    x: usize,
    y: usize,
    is_black: bool,
) {
    set_bit::<BYTES>(modules, y * WIDTH + x, is_black);
}

const fn set_bit<const BYTES: usize>(buffer: &mut [u8; BYTES], index: usize, value: bool) {
    let byte_index = index >> 3;
    let offset = index & 0x07;
    let mask = 1u8 << offset;
    if value {
        buffer[byte_index] |= mask;
    } else {
        buffer[byte_index] &= !mask;
    }
}

const fn isize_abs(value: isize) -> usize {
    if value < 0 {
        (-value) as usize
    } else {
        value as usize
    }
}
