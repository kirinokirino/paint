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

mod clock;
mod common;
mod sprite;

use clock::Clock;
use common::{constrain, Size, Vec2};
use sprite::Sprite;

const RESOLUTION: Size = Size::new(400, 200);

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
    cursor: Sprite,
}

impl Game {
    pub fn new() -> Self {
        let cursor_width = 10;
        let mut pixels = vec![RGBA8::new(0,0,0,0); cursor_width * cursor_width];
        for y in 0..cursor_width {
            for x in 0..cursor_width {
                let i = y * cursor_width + x;
                let pixel = match (x, y) {
                    (0..=5, 0) => RGBA8::new(255, 255, 255, 255),
                    (0, 0..=5) => RGBA8::new(255, 255, 255, 255),
                    _ => {
                        if x == y {
                            RGBA8::new(255, 255, 255, 255)
                        } else {
                            RGBA8::new(0, 0, 0, 0)
                        }
                    }
                };
                pixels[i] = pixel;
            }
        }
        let cursor = Sprite::new(Vec2::new(10.0, 10.0), Size::new(cursor_width as u32, cursor_width as u32), pixels);
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
            cursor,
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
        self.cursor.origin = Vec2::new(self.mouse.x, self.mouse.y);

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

    fn draw(&mut self, ctx: &mut Context) {
        ctx.clear();
        self.canvas.draw(ctx);

        self.cursor.draw(ctx);
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
