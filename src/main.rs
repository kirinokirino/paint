#![warn(clippy::nursery, clippy::pedantic)]
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::missing_const_for_fn,
    clippy::cast_possible_wrap,
    clippy::missing_panics_doc
)]
use simple_pixels::{rgb::RGBA8, start, Config, Context, KeyCode, MouseButton, State};

use std::thread;
use std::time::{Duration, Instant};

const RESOLUTION: Size = Size::new(100, 200);

fn main() {
    let config = Config {
        window_title: "game".to_string(),
        window_width: RESOLUTION.width,
        window_height: RESOLUTION.height,
        fullscreen: false,
        icon: None,
    };

    let game = Game::new();
    start(config, game);
}

struct Game {
    last_mouse_state: Option<(bool, Vec2)>,
    mouse: Vec2,
    clock: Clock,
    canvas: Sprite,
    sprite: Sprite,
}

impl Game {
    pub fn new() -> Self {
        let mut pixels = Vec::with_capacity(20 * 20);
        for y in 0..20 {
            let red = if y % 5 == 0 { 255 } else { 0 };
            for x in 0..20 {
                let green = if x % 5 == 0 { 255 } else { 0 };
                pixels.push(RGBA8::new(red, green, 5 * x + y, 255));
            }
        }
        let sprite = Sprite::new(Vec2::new(10.0, 10.0), Size::new(20, 20), pixels);
        let canvas_buffer =
            vec![RGBA8::default(); RESOLUTION.width as usize * RESOLUTION.height as usize];

        let canvas = Sprite::new(
            Vec2::new(0.0, 0.0),
            Size::new(RESOLUTION.width, RESOLUTION.height),
            canvas_buffer,
        );
        let clock = Clock::new();
        Self {
            last_mouse_state: None,
            mouse: Vec2::new(0., 0.),
            clock,
            sprite,
            canvas,
        }
    }
}

impl State for Game {
    fn update(&mut self, ctx: &mut Context) {
        if ctx.is_key_down(KeyCode::Escape) {
            ctx.quit();
        }

        let mut last_mouse_pressed = false;
        let mut last_mouse_position = Vec2::new(0.0, 0.0);
        if let Some(last_mouse_state) = self.last_mouse_state {
            last_mouse_pressed = last_mouse_state.0;
            last_mouse_position = last_mouse_state.1;
        }
        let (new_mouse_x, new_mouse_y) = ctx.get_mouse_pos();
        self.mouse = Vec2::new(
            constrain(new_mouse_x, 0.0, RESOLUTION.width as f32 - 1.0),
            constrain(new_mouse_y, 0.0, RESOLUTION.height as f32 - 1.0),
        );
        self.sprite.origin = Vec2::new(self.mouse.x, self.mouse.y);

        let mouse_pressed = ctx.is_mouse_button_down(MouseButton::Left);
        if mouse_pressed && !last_mouse_pressed {
            self.canvas.pixels
                [(self.mouse.y as u32 * RESOLUTION.width + self.mouse.x as u32) as usize] =
                RGBA8::new(100, 200, 100, 255);
        } else if mouse_pressed && last_mouse_pressed {
            // draw a line todo
            let line_pixels = line(&self.mouse, &last_mouse_position);
            for (x, y) in line_pixels {
                self.canvas.pixels[(y as u32 * RESOLUTION.width + x as u32) as usize] =
                    RGBA8::new(255, 200, 100, 255);
            }
        }
        self.last_mouse_state = Some((mouse_pressed, self.mouse));
        self.clock.sleep();
    }

    // @
    fn draw(&mut self, ctx: &mut Context) {
        ctx.clear();
        self.canvas.draw(ctx);

        self.sprite.draw(ctx);
    }
}

#[must_use]
pub fn line(from: &Vec2, to: &Vec2) -> Vec<(i32, i32)> {
    let (dest_x, dest_y) = (to.x as i32, to.y as i32);
    let (start_x, start_y) = (from.x as i32, from.y as i32);
    let (delta_x, delta_y) = ((dest_x - start_x), (dest_y - start_y));
    if delta_x == 0 && delta_y == 0 {
        return Vec::new();
    }

    let spacing = if delta_x == 0 || delta_y == 0 {
        1.0
    } else {
        let dx = f64::from(delta_x.abs());
        let dy = f64::from(delta_y.abs());
        (dx / dy).max(dy / dx)
    };

    let len = delta_x.abs().max(delta_y.abs());
    let mut result: Vec<(i32, i32)> = Vec::with_capacity(len as usize);

    let x_step = delta_x.signum();
    let x_iter = num_iter::range_step_inclusive(start_x, dest_x, x_step);
    let y_step = delta_y.signum();
    let y_iter = num_iter::range_step_inclusive(start_y, dest_y, y_step);

    let mut counter = 0.0;
    if delta_x.abs() > delta_y.abs() {
        let mut y = start_y;
        let last = vec![dest_y];
        let mut short_range = y_iter.chain(last.iter().cycle().copied());
        result.extend(x_iter.map(|x| {
            counter += 1.0;
            while counter > spacing {
                y = short_range.next().expect("iterator should be endless");
                counter -= spacing;
            }
            (x, y)
        }));
    } else {
        let mut x = start_x;
        let last = vec![dest_x];
        let mut short_range = x_iter.chain(last.iter().cycle().copied());
        result.extend(y_iter.map(|y| {
            counter += 1.0;
            while counter > spacing {
                x = short_range.next().expect("iterator should be endless");
                counter -= spacing;
            }
            (x, y)
        }));
    };
    result
}

struct Sprite {
    pub origin: Vec2,
    size: Size,
    pixels: Vec<RGBA8>,
}

impl Sprite {
    pub fn new(pos: Vec2, size: Size, pixels: Vec<RGBA8>) -> Self {
        Self {
            origin: pos,
            size,
            pixels,
        }
    }

    pub fn draw(&self, ctx: &mut Context) {
        let screen_width = ctx.width();
        let screen_height = ctx.height();

        let screen_size = Size::new(screen_width, screen_height);
        let screen_origin = Vec2::new(0.0, 0.0);

        let visible_from_x = max(screen_origin.x as i32, self.origin.x as i32);
        let visible_to_x = min(
            self.size.width as i32 + self.origin.x as i32,
            screen_size.width as i32 + screen_origin.x as i32,
        );
        let visible_width = visible_to_x - visible_from_x;
        let sprite_offset_x = if self.origin.x < screen_origin.x {
            -(self.origin.x - screen_origin.x) as i32
        } else {
            0
        };

        let visible_from_y = max(screen_origin.y as i32, self.origin.y as i32);
        let visible_to_y = min(
            self.size.height as i32 + self.origin.y as i32,
            screen_size.height as i32 + screen_origin.y as i32,
        );
        let visible_height = visible_to_y - visible_from_y;
        let sprite_offset_y = if self.origin.y < screen_origin.y {
            -(self.origin.y - screen_origin.y) as i32
        } else {
            0
        };

        let mut visible_pixels: Vec<RGBA8> =
            Vec::with_capacity((visible_width * visible_height).try_into().unwrap());

        for y in sprite_offset_y..visible_height + sprite_offset_y {
            for x in sprite_offset_x..visible_width + sprite_offset_x {
                visible_pixels.push(self.pixels[((y * self.size.width as i32) + x) as usize]);
            }
        }

        ctx.draw_pixels(
            max(screen_origin.x as i32, self.origin.x as i32),
            max(screen_origin.y as i32, self.origin.y as i32),
            visible_width.try_into().unwrap(),
            visible_height.try_into().unwrap(),
            &visible_pixels,
        );
    }
}

#[must_use]
pub fn min(of: i32, or: i32) -> i32 {
    of.min(or)
}

#[must_use]
pub fn max(of: i32, or: i32) -> i32 {
    of.max(or)
}

pub fn constrain<T: PartialOrd>(this: T, min: T, max: T) -> T {
    assert!(min < max);
    if this < min {
        return min;
    } else if this > max {
        return max;
    }
    this
}

#[derive(Debug, Copy, Clone)]
pub struct Size {
    width: u32,
    height: u32,
}

impl Size {
    #[must_use]
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    #[must_use]
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

struct Clock {
    creation_time: Instant,
    past: [Duration; 2],
    lifetime: Duration,
    cycles: u64,
}

impl Clock {
    pub fn new() -> Self {
        let creation_time = Instant::now();
        let lifetime = Duration::new(0, 0);
        let past = [Duration::default(), Duration::default()];
        Self {
            creation_time,
            past,
            lifetime,
            cycles: 0,
        }
    }

    pub fn update(&mut self) {
        self.cycles += 1;
        let now = Instant::now();
        let time_delta = now.duration_since(self.creation_time + self.lifetime);
        self.lifetime = now.duration_since(self.creation_time);
        self.past[1] = self.past[0];
        self.past[0] = time_delta;
        if self.cycles % 15 == 0 {
            println!(
                "playtime: {:.2}, delta: {}ms",
                self.lifetime.as_secs_f32(),
                time_delta.as_millis()
            );
        }
    }

    // Should slowdown updates to 30fps, but, it looks like, actually slows down to 60fps
    pub fn sleep(&mut self) {
        self.update();
        let average_delta = (self.past[0] + self.past[1]).as_secs_f32() / 2.0;
        // approximate frame time for 30fps;
        let frame_time = Duration::new(0, 30_000_000).as_secs_f32();
        let frame_slowdown = if average_delta < frame_time {
            Duration::from_secs_f32(frame_time - average_delta)
        } else {
            Duration::new(0, 0)
        };
        thread::sleep(frame_slowdown);
    }
}
