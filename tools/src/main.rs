// modification of `freetype-rs/examples/single_glyph.rs`
// to generate binary bitmap from otf font file

#![allow(dead_code)]

extern crate freetype as ft;

const WIDTH: usize = 8;
const HEIGHT: usize = 16;

fn draw_bitmap(bitmap: ft::Bitmap, x: usize, y: usize) -> [[u8; WIDTH]; HEIGHT] {
    let mut figure = [[0; WIDTH]; HEIGHT];
    let mut p = 0;
    let mut q = 0;
    let w = bitmap.width() as usize;
    let x_max = x + w;
    let y_max = y + bitmap.rows() as usize;

    for i in x..x_max {
        for j in y..y_max {
            if i < WIDTH && j < HEIGHT {
                figure[j][i] |= bitmap.buffer()[q * w + p];
                q += 1;
            }
        }
        q = 0;
        p += 1;
    }
    figure
}

fn render() {
    // usage :
    // exec noto_size jet_size baseline threshold

    let args = std::env::args().collect::<Vec<_>>();
    let noto_size = args[1].parse::<f64>().unwrap_or(14.0);
    let jet_size = args[2].parse::<f64>().unwrap_or(14.0);
    let baseline = args[3].parse::<usize>().unwrap_or(12);
    // let threshold = args[4].parse::<u8>().unwrap_or(32);
    
    let library = ft::Library::init().unwrap();

    let noto = library.new_face("./assets/NotoSansMonoCJKkr-Regular.otf", 0).unwrap();
    noto.set_char_size(0, (noto_size*64.0).round() as isize, 0, 0).unwrap();

    let jet = library.new_face("./assets/JetBrainsMono-ExtraLight.ttf", 0).unwrap();
    jet.set_char_size(0, (jet_size*64.0).round() as isize, 0, 0).unwrap();

    for c in 0..=0x7f {
        println!("{:#04x}", c );
        let (face, threshold, visible_code) = if c < 0x20 {
            (&jet, 16, c + 0x2400)
        } else if c == 0x7f {
            (&jet, 16, 0x2421)
        } else { (&noto, 48, c) };
        face.load_char(
            visible_code as usize,
            ft::face::LoadFlag::RENDER
        ).unwrap();

        let glyph = face.glyph();

        if (baseline as i32) < glyph.bitmap_top() {
            eprintln!("ERROR {:#04x}({}): baseline too high", c, char::from_u32(visible_code).unwrap());
            continue;
        }
        if (baseline as i32 - glyph.bitmap_top() + glyph.bitmap().rows()) > HEIGHT as i32 {
            eprintln!("ERROR {:#04x}({}): baseline too low", c, char::from_u32(visible_code).unwrap());
            continue;
        }
        if glyph.bitmap_left() + glyph.bitmap().width() > WIDTH as i32 {
            eprintln!("ERROR {:#04x}({}): character too wide", c, char::from_u32(visible_code).unwrap());
            // continue;
        }

        let x = glyph.bitmap_left() as usize;
        let y = (baseline as i32 - glyph.bitmap_top()) as usize;
        let figure = draw_bitmap(glyph.bitmap(), x, y);

        for i in 0..HEIGHT {
            for j in 0..WIDTH {
                print!(
                    "{}",
                    match figure[i][j] {
                        // p if p == 0 => ".",
                        p if p < threshold => ".",
                        // p if p < 128 => "*",
                        _ => "@",
                    }
                );
            }
            println!("");
        }

    }
}

fn main() -> std::io::Result<()> {
    // render();

    use std::io::prelude::*;

    println!("static SYSFONT: [[u8; 16]; 128] = [");

    let file = std::fs::File::open("sysfont.txt")?;
    let reader = std::io::BufReader::new(file);

    let mut init = false;
    for line in reader.lines() {
        let str = line?;
        if str.starts_with("0x") {
            if !init {
                init = true;
            } else {
                println!("    ],");
            }
            println!("    [");
            continue;
        }
        let mut buf = String::new();
        for b in str.bytes() {
            buf.push(if b == b'@' { '1' } else { '0' });
        }
        let buf_rev = buf.chars().rev().collect::<String>();

        println!("        0b{}, ", buf_rev);
    }
    println!("    ],");
    println!("];");

    Ok(())
}