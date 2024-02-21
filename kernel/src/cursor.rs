pub const SYSCURSOR_WIDTH_PX: usize = 9;
pub const SYSCURSOR_HEIGHT_PX: usize = 15;

pub const SYSCURSOR_SHAPE: [&[u8; SYSCURSOR_WIDTH_PX]; SYSCURSOR_HEIGHT_PX] = [
    b"@        ",
    b"@@       ",
    b"@.@      ",
    b"@..@     ",
    b"@...@    ",
    b"@....@   ",
    b"@.....@  ",
    b"@......@ ",
    b"@....@@@@",
    b"@..@.@   ",
    b"@.@@.@   ",
    b"@@  @.@  ",
    b"@   @.@  ",
    b"     @.@ ",
    b"      @@ ",
];