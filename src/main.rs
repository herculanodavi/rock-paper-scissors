use bracket_lib::prelude::*;

enum GameMode {
    Menu,
    Playing,
    End,
}

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 80;
const FRAME_DURATION: f32 = 60.0;

const NUM_PARTICLES: usize = 25;
const MIN_VELOCITY: f64 = -1.0;
const MAX_VELOCITY: f64 = 1.0;
const PARTICLE_RADIUS: f64 = 1.5;

#[derive(Copy, Clone, Debug)]
struct Vec2f {
    x: f64,
    y: f64,
}

impl Vec2f {
    fn scalar_product(&self, other: &Vec2f) -> f64 {
        (self.x * other.x) + (self.y * other.y)
    }

    fn product(&self, other: f64) -> Vec2f {
        Vec2f {
            x: self.x * other,
            y: self.y * other,
        }
    }

    fn minus(&self, other: Vec2f) -> Vec2f {
        Vec2f {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    fn plus(&self, other: Vec2f) -> Vec2f {
        Vec2f {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    fn distance(&self, other: &Vec2f) -> f64 {
        self.minus(*other).norm()
    }

    fn norm(&self) -> f64 {
        self.scalar_product(self).sqrt()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Hand {
    Rock,
    Paper,
    Scissors,
}

pub trait Beats {
    fn beats(&self) -> Self;
}

impl Beats for Hand {
    fn beats(&self) -> Self {
        // match is exhaustive, so every enum variant must be covered
        match *self {
            Hand::Rock => Hand::Scissors,
            Hand::Paper => Hand::Rock,
            Hand::Scissors => Hand::Paper,
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct Particle {
    position: Vec2f,
    velocity: Vec2f,
    hand: Hand,
}

impl Particle {
    fn new() -> Self {
        let mut random = RandomNumberGenerator::new();
        Particle {
            position: Vec2f {
                x: random.range(0.0, SCREEN_WIDTH as f64),
                y: random.range(0.0, SCREEN_HEIGHT as f64),
            },
            velocity: Vec2f {
                x: random.range(MIN_VELOCITY, MAX_VELOCITY),
                y: random.range(MIN_VELOCITY, MAX_VELOCITY),
            },
            hand: match random.range(0, 3) {
                0 => Hand::Rock,
                1 => Hand::Paper,
                _ => Hand::Scissors,
            },
        }
    }

    fn render(&self, ctx: &mut BTerm) {
        let glyph: FontCharType = match self.hand {
            Hand::Rock => 199,
            Hand::Paper => 193,
            Hand::Scissors => 196,
        };

        for dx in -1..2 {
            for dy in -1..2 {
                ctx.set(
                    self.position.x as i32 + dx,
                    self.position.y as i32 - dy,
                    WHITE,
                    BLACK,
                    (glyph as i32 + dx - 16 * dy) as u16,
                );
            }
        }
    }

    fn check_wall_collision(&mut self) {
        if self.position.x < 0.0 {
            self.position.x = -self.position.x;
            self.velocity.x = -self.velocity.x;
        } else if self.position.x > SCREEN_WIDTH as f64 {
            self.position.x = 2.0 * SCREEN_WIDTH as f64 - self.position.x;
            self.velocity.x = -self.velocity.x;
        }

        if self.position.y < 0.0 {
            self.position.y = -self.position.y;
            self.velocity.y = -self.velocity.y;
        } else if self.position.y > SCREEN_HEIGHT as f64 {
            self.position.y = 2.0 * SCREEN_HEIGHT as f64 - self.position.y;
            self.velocity.y = -self.velocity.y;
        }
    }

    fn collides_width(&self, other: &Particle) -> bool {
        self.position.distance(&other.position) < 2.0 * PARTICLE_RADIUS
    }

    fn velocity_projection(&self, other: &Particle) -> Vec2f {
        let line = other.position.minus(self.position);
        line.product(self.velocity.scalar_product(&line) / line.norm().powi(2))
    }

    fn update_position(&mut self) {
        self.position = self.position.plus(self.velocity);
    }

    fn handle_match(&mut self, other: Hand) {
        if other.beats() == self.hand {
            self.hand = other;
        }
    }
}

struct State {
    particles: [Particle; NUM_PARTICLES],
    frame_time: f32,
    mode: GameMode,
    score: i32,
}

impl State {
    fn new() -> Self {
        State {
            particles: [(); NUM_PARTICLES].map(|_| Particle::new()),
            frame_time: 0.0,
            mode: GameMode::Menu,
            score: 0,
        }
    }

    fn collide(&mut self, lhs: usize, rhs: usize) {
        // Changes in velocity
        let v_lr = self.particles[lhs].velocity_projection(&self.particles[rhs]);
        let v_rl = self.particles[rhs].velocity_projection(&self.particles[lhs]);

        self.particles[lhs].velocity = self.particles[lhs].velocity.minus(v_lr).plus(v_rl);
        self.particles[rhs].velocity = self.particles[rhs].velocity.minus(v_rl).plus(v_lr);

        // Displace particles to leave collision condition
        let distance = self.particles[rhs]
            .position
            .distance(&self.particles[lhs].position);
        let displacement = PARTICLE_RADIUS - distance / 2.0; // per particle

        let l_to_r = self.particles[rhs]
            .position
            .minus(self.particles[lhs].position);
        let displacement_vec = l_to_r.product(displacement / l_to_r.norm());

        self.particles[lhs].position = self.particles[lhs].position.minus(displacement_vec);
        self.particles[rhs].position = self.particles[rhs].position.plus(displacement_vec);

        // Change symbol type
        self.particles[lhs].handle_match(self.particles[rhs].hand);
        self.particles[rhs].handle_match(self.particles[lhs].hand);
    }

    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg((54, 126, 127));

        self.frame_time += ctx.frame_time_ms;
        if self.frame_time > FRAME_DURATION {
            self.frame_time = 0.0;

            for particle in &mut self.particles {
                particle.update_position();
                particle.check_wall_collision();
            }

            (0..NUM_PARTICLES).for_each(|lhs| {
                for rhs in lhs + 1..NUM_PARTICLES {
                    if self.particles[lhs].collides_width(&self.particles[rhs]) {
                        self.collide(lhs, rhs);
                        return;
                    }
                }
            });
        }

        for particle in &self.particles {
            particle.render(ctx);
        }
    }

    fn restart(&mut self) {
        self.frame_time = 0.0;
        self.mode = GameMode::Playing;
        self.score = 0;
    }

    fn main_menu(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "Welcome to Flappy Dragon!");
        ctx.print_centered(8, "(P) Play Game");
        ctx.print_centered(9, "(Q) Quit Game");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn dead(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "You are dead!");
        ctx.print_centered(6, &format!("You earned {} points", self.score));
        ctx.print_centered(8, "(P) Play Game");
        ctx.print_centered(9, "(Q) Quit Game");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::End => self.dead(ctx),
            GameMode::Playing => self.play(ctx),
        }
    }
}

fn main() -> BError {
    let context = BTermBuilder::new()
        .with_title("Rock Paper Scissors")
        .with_fps_cap(30.0)
        .with_dimensions(SCREEN_WIDTH, SCREEN_HEIGHT)
        .with_tile_dimensions(64, 64)
        .with_resource_path("resources/")
        .with_font("font.png", 64, 64)
        .with_simple_console(SCREEN_WIDTH, SCREEN_HEIGHT, "font.png")
        .with_simple_console_no_bg(SCREEN_WIDTH, SCREEN_HEIGHT, "font.png")
        .build()?;
    main_loop(context, State::new())
}
