pub const SYSCURSOR_WIDTH: usize = 9;
pub const SYSCURSOR_HEIGHT: usize = 15;

pub static SYSCURSOR_SHAPE: [&[u8; SYSCURSOR_WIDTH]; SYSCURSOR_HEIGHT] = [
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
    b"@.@ @.@  ",
    b"@@  @.@  ",
    b"@    @.@ ",
    b"     @.@ ",
    b"      @@ ",
];