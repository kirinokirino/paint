use simple_pixels::{rgb::RGBA8, start, Config, Context, KeyCode, State};

use std::time::{Duration, Instant};
use std::{ops::Div, thread};

fn main() {
    let config = Config {
        window_title: "game".to_string(),
        window_width: 200,
        window_height: 200,
        fullscreen: false,
        icon: None,
    };

    let game = Game::new();
    start(config, game);
}

struct Game {
    clock: Clock,
    sprite: Sprite,
}

impl Game {
    pub fn new() -> Self {
        let mut pixels = Vec::with_capacity(20 * 20);
        for col in 0..20 {
            let red = if col % 5 == 0 { 255 } else { 0 };
            for row in 0..20 {
                let green = if row % 5 == 0 { 255 } else { 0 };
                pixels.push(RGBA8::new(red, green, 5 * row + col, 255))
            }
        }
        let sprite = Sprite::new(Vec2::new(10.0, 10.0), Size::new(20, 20), pixels);
        let clock = Clock::new();
        Self { clock, sprite }
    }
}

impl State for Game {
    fn update(&mut self, ctx: &mut Context) {
        if ctx.is_key_down(KeyCode::Escape) {
            ctx.quit();
        }

        let mouse = ctx.get_mouse_pos();
        self.sprite.origin = Vec2::new(mouse.0, mouse.1);

        self.clock.sleep();
    }

    fn draw(&mut self, ctx: &mut Context) {
        ctx.clear();
        self.sprite.draw(ctx);
    }
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

        for col in sprite_offset_y..visible_height + sprite_offset_y {
            for row in sprite_offset_x..visible_width + sprite_offset_x {
                visible_pixels.push(self.pixels[((row * self.size.width as i32) + col) as usize]);
            }
        }
        //dbg!(&visible_width, &visible_height);
        ctx.draw_pixels(
            max(screen_origin.x as i32, self.origin.x as i32),
            max(screen_origin.y as i32, self.origin.y as i32),
            visible_width.try_into().unwrap(),
            visible_height.try_into().unwrap(),
            &visible_pixels,
        );
    }
}

pub fn min(of: i32, or: i32) -> i32 {
    of.min(or)
}

pub fn max(of: i32, or: i32) -> i32 {
    of.max(or)
}

struct Size {
    width: u32,
    height: u32,
}

impl Size {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
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
