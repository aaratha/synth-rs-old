extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::video::{Window, WindowContext};
use std::time::Duration;

#[derive(Copy, Clone)]
struct Vec2 {
    x: i32,
    y: i32,
}

impl Vec2 {
    fn new(x: i32, y: i32) -> Self {
        Vec2 { x, y }
    }

    fn lerp(&self, target: Vec2, t: f32) -> Vec2 {
        Vec2 {
            x: (self.x as f32 + t * (target.x - self.x) as f32) as i32,
            y: (self.y as f32 + t * (target.y - self.y) as f32) as i32,
        }
    }
}

struct Square {
    position: Vec2,
    size: u32,
    color: Color,
    dragging: bool,
    offset: Vec2,
    angle: f64,
}

impl Square {
    fn new(x: i32, y: i32, size: u32, color: Color) -> Self {
        Square {
            position: Vec2::new(x, y),
            size,
            color,
            dragging: false,
            offset: Vec2::new(0, 0),
            angle: 0.0,
        }
    }

    fn rect(&self) -> Rect {
        Rect::new(self.position.x, self.position.y, self.size, self.size)
    }

    fn set_position(&mut self, x: i32, y: i32) {
        self.position.x = x - self.offset.x;
        self.position.y = y - self.offset.y;
    }

    fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.position.x
            && x <= self.position.x + self.size as i32
            && y >= self.position.y
            && y <= self.position.y + self.size as i32
    }

    fn lerp_to(&mut self, target: Vec2, t: f32) {
        self.position = self.position.lerp(target, t);
    }

    fn tilt(&mut self, target: Vec2, strength: f32) {
        let dx = target.x - self.position.x;
        let angle = dx as f64 * strength as f64;
        self.angle = angle;
    }
}

fn create_texture<'a>(
    canvas: &mut Canvas<Window>,
    texture_creator: &'a TextureCreator<WindowContext>,
    color: Color,
    size: u32,
) -> sdl2::render::Texture<'a> {
    let mut texture = texture_creator
        .create_texture_target(None, size, size)
        .unwrap();
    canvas
        .with_texture_canvas(&mut texture, |texture_canvas| {
            texture_canvas.set_draw_color(color);
            texture_canvas.clear();
        })
        .unwrap();
    texture
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("SDL2 Drag and Drop", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut square1 = Square::new(100, 100, 100, Color::RGB(255, 0, 0));
    let mut square2 = Square::new(120, 120, 100, Color::RGB(0, 0, 255));
    let square2_texture =
        create_texture(&mut canvas, &texture_creator, square2.color, square2.size);

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                Event::MouseButtonDown {
                    x, y, mouse_btn, ..
                } => {
                    if mouse_btn == MouseButton::Left && square1.contains_point(x, y) {
                        square1.dragging = true;
                        square1.offset = Vec2::new(x - square1.position.x, y - square1.position.y);
                    }
                }
                Event::MouseButtonUp { mouse_btn, .. } => {
                    if mouse_btn == MouseButton::Left {
                        square1.dragging = false;
                    }
                }
                Event::MouseMotion { x, y, .. } => {
                    if square1.dragging {
                        square1.set_position(x, y);
                    }
                }
                _ => {}
            }
        }

        // Update second square's position to follow the first square using LERP
        square2.lerp_to(square1.position, 0.3);

        // Update the tilt of the second square
        square2.tilt(square1.position, 0.1);

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        canvas.set_draw_color(square1.color);
        canvas.fill_rect(square1.rect()).unwrap();

        // Render the second square with rotation
        let rect = square2.rect();
        canvas
            .copy_ex(
                &square2_texture,
                None,
                rect,
                square2.angle,
                None,
                false,
                false,
            )
            .unwrap();

        canvas.present();
        std::thread::sleep(Duration::from_millis(16));
    }
}
