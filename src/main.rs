use bracket_lib::prelude::*;

enum GameMode {
    Menu,
    Playing,
    End,
}

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const FRAME_DURATION: f32 = 30.0;
const BACKGROUND_COLOR: (u8, u8, u8) = SKY_BLUE;
const PLAYER_SCREEN_X: i32 = 5;

const OBSTACLE_WIDTH: i32 = 5;
const NUM_PARTICLES: usize = 5;

#[derive(Copy, Clone)]
enum ParticleType {
    Rock,
    Paper,
    Scissors,
}

#[derive(Copy, Clone)]
struct Particle {
    x: i32,
    y: i32,
    v_x: i32,
    v_y: i32,
    kind: ParticleType,
}

impl Particle {
    fn new() -> Self {
        let mut random = RandomNumberGenerator::new();
        Particle {
            x: random.range(0, SCREEN_WIDTH),
            y: random.range(0, SCREEN_HEIGHT),
            v_x: random.range(-2, 3),
            v_y: random.range(-2, 3),
            kind: match random.range(0, 3) {
                0 => ParticleType::Rock,
                1 => ParticleType::Paper,
                _ => ParticleType::Scissors,
            },
        }
    }

    fn render(&self, ctx: &mut BTerm) {
        ctx.set(
            self.x,
            self.y,
            WHITE,
            BLACK,
            to_cp437(match self.kind {
                ParticleType::Rock => '☻',
                ParticleType::Paper => '■',
                ParticleType::Scissors => '▲',
            }),
        );
    }

    fn update_position(&mut self) {
        self.x += self.v_x;
        if self.x < 0 {
            self.x = -self.x
        } else if self.x > SCREEN_WIDTH {
            self.x = SCREEN_WIDTH - (self.x - SCREEN_WIDTH)
        }

        self.y += self.v_y;
        if self.y < 0 {
            self.y = -self.y
        } else if self.y > SCREEN_HEIGHT {
            self.y = SCREEN_HEIGHT - (self.y - SCREEN_HEIGHT)
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

    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(BLACK);

        for mut particle in self.particles {
            particle.render(ctx);
            particle.update_position()
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
    let context = BTermBuilder::simple80x50()
        .with_title("Rock Paper Scissors")
        .build()?;
    main_loop(context, State::new())
}
