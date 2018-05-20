extern crate sdl2;
extern crate sdl2_sys;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use std::mem;
use std::mem::transmute;
use std::process;
use std::thread;
use std::time::Duration;

#[allow(unused_imports)]
use std::env;

pub mod chip8;

const KEYMAP: [sdl2::keyboard::Keycode; 16] = [
    sdl2::keyboard::Keycode::X,
    sdl2::keyboard::Keycode::Num1,
    sdl2::keyboard::Keycode::Num2,
    sdl2::keyboard::Keycode::Num3,
    sdl2::keyboard::Keycode::Q,
    sdl2::keyboard::Keycode::W,
    sdl2::keyboard::Keycode::E,
    sdl2::keyboard::Keycode::A,
    sdl2::keyboard::Keycode::S,
    sdl2::keyboard::Keycode::D,
    sdl2::keyboard::Keycode::Z,
    sdl2::keyboard::Keycode::C,
    sdl2::keyboard::Keycode::Num4,
    sdl2::keyboard::Keycode::R,
    sdl2::keyboard::Keycode::F,
    sdl2::keyboard::Keycode::V,
];

fn main() {
    let mut chip = chip8::Chip8::new();
    // let args: String = env::args().collect();
    // chip.load(&args);

    chip.load("roms/PONG2");

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

    let pixels = &mut [0 as u32; 2048];

    let mut events = ctx.event_pump().unwrap();

    loop {
        chip.emulate_cycle();

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
                    repeat,
                    keycode: Some(keycode),
                    ..
                } => {
                    if !repeat {
                        for i in 0..16 {
                            if keycode == KEYMAP[i] {
                                chip.key[i] = 1;
                            }
                        }
                    }
                }
                Event::KeyUp {
                    repeat,
                    keycode: Some(keycode),
                    ..
                } => {
                    if !repeat {
                        for i in 0..16 {
                            if keycode == KEYMAP[i] {
                                chip.key[i] = 0;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if chip.draw_flag {
            chip.draw_flag = false;

            for index in 0..pixels.len() {
                let gfx_pixel = chip.gfx[index].to_owned() as u32;
                let new_value = (0x00FFFFFF * gfx_pixel) | 0xFF000000;
                pixels[index] = new_value;
            }

            let buff: [u8; 8192] = unsafe { transmute(*pixels) };

            // these return errors that should really be handled
            let _ = sdl_texture.update(None, &buff, 64 * mem::size_of::<u32>());
            let _ = renderer.clear();
            let _ = renderer.copy(&sdl_texture, None, None);
            let _ = renderer.present();
        }

        thread::sleep(Duration::from_millis(10));
    }
}
