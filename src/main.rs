extern crate sdl2;
extern crate sdl2_sys;

extern crate byteorder;

use byteorder::ByteOrder;
use byteorder::LittleEndian;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::process;

use std::mem;
use std::thread;
use std::time::Duration;

pub mod chip8;

fn main() {
    let mut chip = chip8::Chip8::new();
    chip.load("./roms/TETRIS");

    // sets up window and draws rectangle right now
    let ctx = sdl2::init().unwrap();
    let video_ctx = ctx.video().unwrap();

    let window = match video_ctx
        .window("CHIP8 EMU", 1024, 512)
        .position_centered()
        .opengl()
        .build()
    {
        Ok(window) => window,
        Err(err) => panic!("failed to create window: {}", err),
    };

    let mut renderer = match window.into_canvas().build() {
        Ok(renderer) => renderer,
        Err(err) => panic!("failed to create renderer: {}", err),
    };

    let texture_creator = renderer.texture_creator();
    let mut sdl_texture = match texture_creator.create_texture(
        sdl2::pixels::PixelFormatEnum::ARGB8888,
        sdl2::render::TextureAccess::Streaming,
        64,
        32,
    ) {
        Ok(texture) => texture,
        Err(err) => panic!("failed to create renderer: {}", err),
    };

    #[allow(unused_mut)]
    let mut pixels = &mut [0 as u32; 2048];

    let mut events = ctx.event_pump().unwrap();

    loop {
        chip.emulate_cycle();

        // gfx debug
        // println!("---gfx---");
        // for element in chip.gfx.into_iter() {
        //     print!("{}, ", element);
        // }
        // println!("---end gfx---");

        for event in events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    process::exit(1);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    // do things
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    // do things
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    // do things
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    // do things
                }
                _ => {}
            }
        }

        if chip.draw_flag {
            chip.draw_flag = false;

            // let gfx_pixels = chip.gfx.to_owned();
            // pixels = &mut gfx_pixels.to_owned();

            for index in 0..pixels.len() {
                let gfx_pixel = chip.gfx[index].to_owned() as u32;
                let new_value = (0x00FFFFFF * gfx_pixel) | 0xFF000000;
                pixels[index] = new_value;
            }

            // hack to get pixels correct
            let mut buff = [0; (2048 * 32) / 8];

            for pixel in pixels.iter() {
                LittleEndian::write_u32(&mut buff, *pixel);
            }

            // these return errors that should really be handled
            let _ = sdl_texture.update(None, &buff, 64 * mem::size_of::<u32>());
            let _ = renderer.clear();
            let _ = renderer.copy(&sdl_texture, None, None);
            let _ = renderer.present();
        }

        thread::sleep(Duration::from_millis(1000));
        // thread::sleep(Duration::from_micros(1200));
    }
}
