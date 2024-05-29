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

const GRID_SIZE: i32 = 100;
const LERP_SPEED: f32 = 0.3;
const TILT_STRENGTH: f32 = 0.4;

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

    fn center(&self) -> Vec2 {
        Vec2::new(
            self.position.x + self.size as i32 / 2,
            self.position.y + self.size as i32 / 2,
        )
    }

    fn snap_to_grid(&mut self, grid_spacing: i32) {
        let center = self.center();
        let snapped_x = ((center.x / grid_spacing) * grid_spacing) + grid_spacing / 2;
        let snapped_y = ((center.y / grid_spacing) * grid_spacing) + grid_spacing / 2;
        self.position.x = snapped_x - self.size as i32 / 2;
        self.position.y = snapped_y - self.size as i32 / 2;
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

fn draw_grid(canvas: &mut Canvas<Window>, color: Color, spacing: i32) {
    canvas.set_draw_color(color);
    let (width, height) = canvas.output_size().unwrap();
    for x in (0..width).step_by(spacing as usize) {
        canvas
            .draw_line((x as i32, 0), (x as i32, height as i32))
            .unwrap();
    }
    for y in (0..height).step_by(spacing as usize) {
        canvas
            .draw_line((0, y as i32), (width as i32, y as i32))
            .unwrap();
    }
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
                        square1.snap_to_grid(GRID_SIZE);
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
        square2.lerp_to(square1.position, LERP_SPEED);

        // Update the tilt of the second square
        square2.tilt(square1.position, TILT_STRENGTH);

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        // Draw grid
        draw_grid(&mut canvas, Color::RGB(50, 50, 50), GRID_SIZE);

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
