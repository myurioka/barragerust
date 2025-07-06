use async_trait::async_trait;
use rand::prelude::*;
use std::cmp::{max, min};
use std::f64;
use std::f64::consts::PI;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, MouseEvent, TouchEvent, window};

const CANVAS_WIDTH: f32 = 800.0;
const CANVAS_HEIGHT: f32 = 1000.0;
const BOSS_WIDTH: f32 = 225.0; // ENEMY BOSS WIDTH for hit judgement
const BOSS_HEIGHT: f32 = 225.0; // ENEMY BOSS HEIGHT for hit judgement
const BOSS_MAX_HP: i32 = 999; // ENEMY BOSS MAX Helath Point
const MAX_BULLET_NUMBER: i32 = 500; // Number of BULLETS
const BULLET_WIDTH: f32 = 28.0;
const BULLET_HEIGHT: f32 = 28.0;
const BULLET_STRENGTH: i32 = 2;
const SHIP_WIDTH: f32 = 40.0;
const SHIP_HEIGHT: f32 = 50.0;
const SHIP_STEP: f32 = 1.0;
const SHOT_WIDTH: f32 = 5.0;
const SHOT_HEIGHT: f32 = 5.0;
const SHOT_SPEED: f32 = 10.0;
const SUPER_SHOT_WIDTH: f32 = 100.0;
const SUPER_SHOT_HEIGHT: f32 = 40.0;
const SUPER_WAIT_TIME: i32 = 200; // enery chage time
const SUPER_TIME: i32 = 200; // super mode time
const DEFAULT_COLOR: &str = "rgba(0,128, 0, 1.0)";
const LIGHT_GREEN_COLOR: &str = "rgba(226,238,197,1.0)";
const GREEN_DARK_LIGHT: &str = "rgba(17,31,17,1.0)";
const LIGHT_YELLOR_GREEN: &str = "rgba(168,230,207,1.0)";
const FPS: i32 = 16; // FPS 1000ms / 60frame

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into())
    }
}

// main : Wasm Access Point

#[wasm_bindgen]
pub fn main() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    wasm_bindgen_futures::spawn_local(async move {
        let document = window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas: HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>().unwrap();
        let game = Game::new(canvas);
        GameLoop::start(game).await.expect("Start Game");
    });
    Ok(())
}

trait Character {
    fn get_x(&self) -> f32;
    fn get_y(&self) -> f32;
    fn get_w(&self) -> f32;
    fn get_h(&self) -> f32;
    fn exit(&self) -> bool {
        let _x = self.get_x();
        let _y = self.get_y();
        if _x < 0.0 || _y < 0.0 || _x > CANVAS_WIDTH || _y > CANVAS_HEIGHT {
            return true;
        }
        false
    }
    fn hit(&self, obj: &dyn Character) -> bool {
        let _x = self.get_x() as i32;
        let _y = self.get_y() as i32;
        let _w = self.get_w() as i32;
        let _h = self.get_h() as i32;
        let _obj_x = obj.get_x() as i32;
        let _obj_y = obj.get_y() as i32;
        let _obj_w = obj.get_w() as i32;
        let _obj_h = obj.get_h() as i32;

        if max(_x, _obj_x) < min(_x + _w, _obj_x + _obj_w)
            && (max(_y, _obj_y) < min(_y + _h, _obj_y + _obj_h))
        {
            return true;
        } else {
            false
        }
    }
    fn update(&mut self);
    fn draw(&self, ctx: CanvasRenderingContext2d);
}

// Shot

#[derive(Clone, Debug)]
pub enum ShotType {
    Normal,
    Super,
}

#[derive(Clone, Debug)]
struct Shot {
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
    w: f32,
    h: f32,
    hp: i32,
    t: ShotType,
}

impl Character for Shot {
    fn get_x(&self) -> f32 {
        self.x
    }
    fn get_y(&self) -> f32 {
        self.y
    }
    fn get_w(&self) -> f32 {
        self.w
    }
    fn get_h(&self) -> f32 {
        self.h
    }
    fn update(&mut self) {
        self.y -= self.dy;
    }
    fn draw(&self, ctx: CanvasRenderingContext2d) {
        match self.t {
            ShotType::Normal => {
                ctx.begin_path();
                ctx.set_fill_style_str(DEFAULT_COLOR);
                ctx.set_line_width(2.0);
                ctx.move_to(self.x.into(), (self.y - SHOT_HEIGHT).into());
                ctx.line_to((SHOT_WIDTH + self.x).into(), (self.y - SHOT_HEIGHT).into());
                ctx.line_to((SHOT_WIDTH + self.x).into(), self.y.into());
                ctx.line_to(self.x.into(), self.y.into());
                ctx.close_path();
                ctx.fill();
            }
            ShotType::Super => {
                let _x: f32 = self.get_x();
                let _y: f32 = self.get_y();

                ctx.begin_path();
                ctx.set_fill_style_str(LIGHT_YELLOR_GREEN);
                ctx.move_to(_x.into(), (_y - SUPER_SHOT_HEIGHT).into());
                ctx.line_to(
                    (SUPER_SHOT_WIDTH + _x).into(),
                    (_y - SUPER_SHOT_HEIGHT).into(),
                );
                ctx.line_to((SUPER_SHOT_WIDTH + _x).into(), _y.into());
                ctx.line_to(_x.into(), _y.into());
                ctx.close_path();
                ctx.fill();
            }
        }
    }
}
#[derive(Clone, Debug)]
struct Bullet {
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
    w: f32,
    h: f32,
    hp: i32,
}

impl Character for Bullet {
    fn get_x(&self) -> f32 {
        self.x
    }
    fn get_y(&self) -> f32 {
        self.y
    }
    fn get_w(&self) -> f32 {
        self.w
    }
    fn get_h(&self) -> f32 {
        self.h
    }
    fn update(&mut self) {
        self.x += self.dx;
        self.y += self.dy;
    }
    fn draw(&self, ctx: CanvasRenderingContext2d) {
        let _x = &self.x;
        let _y = &self.y;

        ctx.begin_path();
        ctx.set_fill_style_str(DEFAULT_COLOR);
        let _ = ctx.arc(
            (*_x).into(),
            (*_y).into(),
            (BULLET_WIDTH / 2.0).into(),
            0.0,
            (PI * 2.0).into(),
        );
        ctx.close_path();
        ctx.fill();
        ctx.begin_path();
        ctx.set_fill_style_str(LIGHT_GREEN_COLOR);
        let _ = ctx.arc(
            (*_x).into(),
            (*_y).into(),
            (BULLET_WIDTH / 4.0).into(),
            0.0,
            (PI * 2.0).into(),
        );
        ctx.close_path();
        ctx.fill();
    }
}

#[derive(Clone, Debug)]
struct Boss {
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
    w: f32,
    h: f32,
    hp: i32,
    d: f32, // 0:left, 1:right
}

impl Character for Boss {
    fn get_x(&self) -> f32 {
        self.x
    }
    fn get_y(&self) -> f32 {
        self.y
    }
    fn get_w(&self) -> f32 {
        self.w
    }
    fn get_h(&self) -> f32 {
        self.h
    }
    fn update(&mut self) {
        // direction
        if self.x < 0.0 && self.d == -1.0 {
            self.d = 1.0;
        }
        if self.x > CANVAS_WIDTH - BOSS_WIDTH && self.d == 1.0 {
            self.d = -1.0;
        }
        self.x += self.d * self.dx;
    }
    fn draw(&self, ctx: CanvasRenderingContext2d) {
        let _txt = format!("{} / {}", &self.hp, BOSS_MAX_HP);
        ctx.set_font("12px myfont");
        ctx.set_fill_style_str(LIGHT_GREEN_COLOR);
        let _ = ctx.fill_text(&_txt, (&self.x + 70.0).into(), (&self.y - 10.0).into());
        draw_boss(&ctx, self.x, self.y);
    }
}

// Ship

#[derive(Clone, Debug)]
struct Ship {
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
    w: f32,
    h: f32,
    hp: i32,
    mouse_x: f32,
    t: ShotType,
}

trait CharacterShip {
    fn set_mouse_x(&mut self, offset_x: f32);
}

impl CharacterShip for Ship {
    fn set_mouse_x(&mut self, offset_x: f32) {
        self.mouse_x = offset_x;
    }
}

impl Character for Ship {
    fn get_x(&self) -> f32 {
        self.x
    }
    fn get_y(&self) -> f32 {
        self.y
    }
    fn get_w(&self) -> f32 {
        self.w
    }
    fn get_h(&self) -> f32 {
        self.h
    }
    fn update(&mut self) {
        // direction
        if self.mouse_x < self.x {
            self.x -= self.dx;
        } else if self.mouse_x > self.x + SHIP_WIDTH {
            self.x += self.dx;
        }
    }
    fn draw(&self, ctx: CanvasRenderingContext2d) {
        match self.t {
            ShotType::Normal => {
                draw_ship(ctx, self.x, self.y);
            }
            ShotType::Super => {
                draw_super_ship(ctx, self.x, self.y);
            }
        }
    }
}

// Static Game Trait

#[async_trait(?Send)]
pub trait StaticGame {
    fn new(canvas: HtmlCanvasElement) -> Self;
    fn get_canvas(&mut self) -> HtmlCanvasElement;
    fn on_animation_frame(&mut self);
    fn shot(&mut self, _x: i32, _y: i32);
    fn update(&mut self);
    fn draw(&mut self);
    fn reset(&mut self);
    fn mouse_move(&mut self, mouse_x: f32);
}

// Game Loop

struct GameLoop {
    last_frame: i32,
    accumulated_delta: i32,
}
impl GameLoop {
    pub async fn start(mut game: impl StaticGame + 'static) -> Result<(), String> {
        log!("START");
        let _canvas = game.get_canvas();
        let closure = Rc::new(RefCell::new(None));
        let closure_cloned = Rc::clone(&closure);

        let ref_game = Rc::new(RefCell::new(game));
        let ref_game_clone = ref_game.clone();
        let ref_game_update_clone = ref_game.clone();
        let ref_game_draw_clone = ref_game.clone();
        let ref_game_mousemove_clone = ref_game.clone();
        let ref_game_touchmove_clone = ref_game.clone();

        let mut game_loop = GameLoop {
            last_frame: get_now(),
            accumulated_delta: 0,
        };

        // on_animation_frame

        let _time = 0;
        closure_cloned.replace(Some(Closure::wrap(Box::new(move |_time: i32| {
            game_loop.accumulated_delta += _time - game_loop.last_frame;

            // FPS 1/60

            //while game_loop.accumulated_delta >= FPS {
            while game_loop.accumulated_delta >= 0 {
                // update start
                ref_game_update_clone.borrow_mut().update();
                // update end
                game_loop.accumulated_delta -= FPS;
            }
            game_loop.last_frame = _time;

            // draw start
            ref_game_draw_clone.borrow_mut().draw();
            // draw end

            ref_game.borrow_mut().on_animation_frame();
            request_animation_frame(closure.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut(i32)>)));

        request_animation_frame(closure_cloned.borrow().as_ref().unwrap());

        // mousedown event callback

        let _mouse_down = Closure::wrap(Box::new(move |e: MouseEvent| {
            ref_game_clone.borrow_mut().shot(e.client_x(), e.client_y());
        }) as Box<dyn FnMut(_)>);
        _canvas
            .add_event_listener_with_callback("mousedown", _mouse_down.as_ref().unchecked_ref())
            .unwrap();
        _mouse_down.forget();

        // mousemove event callback

        let _mouse_move = Closure::wrap(Box::new(move |e: MouseEvent| {
            ref_game_mousemove_clone
                .borrow_mut()
                .mouse_move(e.offset_x() as f32);
        }) as Box<dyn FnMut(_)>);
        _canvas
            .add_event_listener_with_callback("mousemove", _mouse_move.as_ref().unchecked_ref())
            .unwrap();
        _mouse_move.forget();

        // touchemove event callback

        let _touch_move = Closure::wrap(Box::new(move |e: TouchEvent| {
            let all_touches = e.touches();
            if all_touches.length() > 0 {
                let touch = all_touches.get(0).unwrap();
                let _x = touch.client_x();
                ref_game_touchmove_clone.borrow_mut().mouse_move(_x as f32);
            }
        }) as Box<dyn FnMut(_)>);
        _canvas
            .add_event_listener_with_callback("touchmove", _touch_move.as_ref().unchecked_ref())
            .unwrap();
        _touch_move.forget();

        Ok(())
    }
}

fn request_animation_frame(f: &Closure<dyn FnMut(i32)>) {
    window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

// Game Object

#[derive(Debug, Clone)]
enum Stage {
    Openning,
    Playing,
    Gameover,
    Gameclear,
}

#[derive(Debug, Clone)]
struct Game {
    canvas: HtmlCanvasElement,
    stage: Stage,
    shooting: bool,
    wait_time: i32,
    super_time: i32,
    ship: Ship,
    bosses: Vec<Boss>,
    bullets: Vec<Bullet>,
    shots: Vec<Shot>,
    max_passed_milli_secondtime: i32,
    max_passed_milli_secondtime_draw: i32,
    passed_milli_secondtime: i32,
    start_milli_secondtime: i32,
}

impl StaticGame for Game {
    // init

    fn new(canvas: HtmlCanvasElement) -> Self {
        Game {
            canvas: canvas,
            stage: Stage::Openning,
            shooting: false,
            wait_time: 0,
            super_time: 0,
            ship: Ship {
                x: CANVAS_WIDTH / 2.0 - SHIP_WIDTH / 2.0,
                y: CANVAS_HEIGHT - SHIP_HEIGHT - 10.0,
                dx: SHIP_STEP,
                dy: 0.0,
                w: SHIP_WIDTH,
                h: SHIP_HEIGHT,
                hp: 1,
                mouse_x: CANVAS_WIDTH / 2.0,
                t: ShotType::Normal,
            },
            bosses: vec![Boss {
                x: 180.0,
                y: 60.0,
                dx: 1.0,
                dy: 0.0,
                w: BOSS_WIDTH,
                h: BOSS_HEIGHT,
                hp: BOSS_MAX_HP,
                d: 1.0,
            }],
            bullets: vec![],
            shots: vec![],
            start_milli_secondtime: get_now(),
            passed_milli_secondtime: 0,
            max_passed_milli_secondtime: 0,
            max_passed_milli_secondtime_draw: 0,
        }
    }

    // get canvas

    fn get_canvas(&mut self) -> HtmlCanvasElement {
        self.canvas.clone()
    }

    // callback animation

    fn on_animation_frame(&mut self) {
        self.draw();
    }

    fn mouse_move(&mut self, _x: f32) {
        self.ship.set_mouse_x(_x);
    }
    // callback click
    fn shot(&mut self, _x: i32, _y: i32) {
        match &self.stage {
            Stage::Gameclear => {
                self.stage = Stage::Playing;
                self.reset();
            }
            Stage::Openning => {
                self.stage = Stage::Playing;
            }
            Stage::Gameover => {
                self.reset();
            }
            Stage::Playing => {
                self.wait_time = 0;
                self.shooting = !self.shooting;
            }
        }
    }

    // restart

    fn reset(&mut self) {
        self.stage = Stage::Openning;
        self.shooting = false;
        self.ship = Ship {
            x: CANVAS_WIDTH / 2.0 - SHIP_WIDTH / 2.0,
            y: CANVAS_HEIGHT - SHIP_HEIGHT - 10.0,
            dx: SHIP_STEP,
            dy: 0.0,
            w: SHIP_WIDTH,
            h: SHIP_HEIGHT,
            hp: 1,
            mouse_x: CANVAS_WIDTH / 2.0,
            t: ShotType::Normal,
        };
        self.bosses = vec![Boss {
            x: 180.0,
            y: 60.0,
            dx: 1.0,
            dy: 0.0,
            w: BOSS_WIDTH,
            h: BOSS_HEIGHT,
            hp: BOSS_MAX_HP,
            d: 1.0,
        }];
        self.wait_time = 0;
        self.super_time = 0;
        self.bullets = vec![];
        self.shots = vec![];
        self.start_milli_secondtime = get_now();
        self.max_passed_milli_secondtime = 0;
        self.max_passed_milli_secondtime_draw = 0;
        self.passed_milli_secondtime = 0;
    }

    // game controller
    fn update(&mut self) {
        match &self.stage {
            Stage::Openning => {}
            Stage::Gameclear => {}
            Stage::Gameover => {}
            Stage::Playing => {
                //passed time

                let _start_process_milli_secondtime = get_now();
                self.passed_milli_secondtime = get_now() - self.start_milli_secondtime;

                // bullet create

                if self.bullets.len() < MAX_BULLET_NUMBER as usize {
                    let mut rnd = rand::thread_rng();
                    let _x = self.bosses[0].x + BOSS_WIDTH / 2.0 - BULLET_WIDTH / 2.0;
                    let _y = self.bosses[0].y + BOSS_WIDTH / 2.0 - BULLET_WIDTH / 2.0;
                    let _dx: f32 = (rnd.gen_range(0..1000) as f32 / 1000.0 - 0.5) * 4.0;
                    let _dy: f32 = (rnd.gen_range(0..1000) as f32 / 1000.0 - 0.5) * 4.0;
                    let _bullet = Bullet {
                        x: _x,
                        y: _y,
                        dx: _dx,
                        dy: _dy,
                        w: BULLET_WIDTH,
                        h: BULLET_HEIGHT,
                        hp: BULLET_STRENGTH,
                    };
                    self.bullets.push(_bullet);
                }
                // bullet update
                self.bullets.iter_mut().for_each(|b| b.update());
                self.bullets.retain(|b| b.exit() == false && *&b.hp > 0);

                // shot create

                if self.shooting {
                    let _x = self.ship.x + SHIP_WIDTH / 2.0;
                    let _y = self.ship.y;
                    if self.super_time > 0 {
                        let _shot = Shot {
                            x: _x - SUPER_SHOT_WIDTH / 2.0,
                            y: _y,
                            dx: 0.0,
                            dy: SHOT_SPEED,
                            w: SUPER_SHOT_WIDTH,
                            h: SUPER_SHOT_HEIGHT,
                            hp: 3,
                            t: ShotType::Super,
                        };
                        self.shots.push(_shot);
                        self.super_time -= 1;
                    } else {
                        let _shot = Shot {
                            x: _x,
                            y: _y,
                            dx: 0.0,
                            dy: SHOT_SPEED,
                            w: SHOT_WIDTH,
                            h: SHOT_HEIGHT,
                            hp: 1,
                            t: ShotType::Normal,
                        };
                        self.shots.push(_shot);
                    }
                }

                // shots update

                self.shots.retain(|s| s.exit() == false && *&s.hp > 0);
                self.shots.iter_mut().for_each(|s| s.update());

                // hit check bullets x ship

                self.bullets.iter_mut().for_each(|b| {
                    if self.ship.hit(b) {
                        self.stage = Stage::Gameover;
                    }
                });

                // hit check shots x bullets

                self.shots.iter_mut().for_each(|s| {
                    self.bullets.iter_mut().for_each(|b| {
                        if s.hit(b) {
                            s.hp -= 1;
                            b.hp -= 1;
                        }
                    })
                });

                // hit check shots x boss

                self.shots.iter_mut().for_each(|s| {
                    self.bosses.iter_mut().for_each(|b| {
                        if s.hit(b) {
                            s.hp -= 1;
                            b.hp -= 1;
                        }
                    })
                });

                // ship update

                self.ship.update();

                // boss update

                self.bosses[0].update();

                if self.bosses[0].hp <= 0 {
                    self.bosses.retain(|s| *&s.hp > 0);
                    // Game Clear
                    if self.bosses.len() == 0 {
                        self.stage = Stage::Gameclear;
                    }
                }

                // not shooting

                if self.shooting == false && self.super_time == 0 {
                    self.wait_time += 1;
                }

                if self.wait_time > SUPER_WAIT_TIME {
                    self.super_time = SUPER_TIME;
                    self.ship.t = ShotType::Super;
                }

                if self.super_time <= 1 {
                    self.ship.t = ShotType::Normal;
                }

                if self.shooting == true {
                    self.wait_time = 0;
                }

                // mesure max passed time
                self.max_passed_milli_secondtime = max(
                    get_now() - _start_process_milli_secondtime,
                    self.max_passed_milli_secondtime,
                );
            }
        }
    }

    // draw

    fn draw(&mut self) {
        let _context = self
            .canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        let _ = _context.set_global_alpha(1.0);
        let _ = _context.clear_rect(0.0, 0.0, CANVAS_WIDTH as f64, CANVAS_HEIGHT as f64);

        match &self.stage {
            Stage::Openning => {
                // Draw Title
                draw_boss(&_context, 130.0, 20.0);
                let _ = _context.set_font("60px myfont");
                let _ = _context.fill_text("BARRAGE", 220.0, 360.0);
                let _ = _context.set_fill_style_str(LIGHT_GREEN_COLOR);
                let _ = _context.set_font("28px myfont");
                let _ = _context.fill_text("RUST & WASM", 270.0, 420.0);
                let _ = _context.set_fill_style_str(DEFAULT_COLOR);
                let _ = _context.set_font("28px myfont");
                let _ = _context.fill_text("Click Start", 300.0, 490.0);
                draw_ship(
                    _context.clone(),
                    self.canvas.client_width() as f32 / 2.0 - SHIP_WIDTH / 2.0,
                    self.canvas.client_height() as f32 - SHIP_HEIGHT - 10.0,
                );
            }
            Stage::Gameover => {
                // Draw Title
                let _ = _context.set_font("60px myfont");
                let _ = _context.fill_text("GAME OVER", 150.0, 360.0);
                let _ = _context.set_font("28px myfont");
                let _ = _context.set_fill_style_str(DEFAULT_COLOR);
                let _ = _context.fill_text("CLick Restart", 280.0, 420.0);
            }
            Stage::Gameclear => {
                let _ = _context.set_fill_style_str(DEFAULT_COLOR);
                let _ = _context.set_font("60px myfont");
                let _ = _context.fill_text("GAME CLEAR", 150.0, 360.0);
                let _ = _context.set_fill_style_str(DEFAULT_COLOR);
                let _ = _context.set_font("28px myfont");
                let _ = _context.fill_text("Congratiations!", 260.0, 420.0);
                let _cleartime = &format!(
                    "Your clear time: {} s.",
                    get_passed_time(&(self.passed_milli_secondtime))
                );
                let _ = _context.fill_text(_cleartime, 200.0, 500.0);
                let _ = _context.set_fill_style_str(LIGHT_GREEN_COLOR);
                let _max_update_time =
                    &format!("max update time: {} ms.", &self.max_passed_milli_secondtime);
                let _ = _context.fill_text(_max_update_time, 200.0, 600.0);
                let _max_draw_time = &format!(
                    "max draw time: {} ms.",
                    &self.max_passed_milli_secondtime_draw
                );
                let _ = _context.fill_text(_max_draw_time, 200.0, 650.0);
            }
            Stage::Playing => {
                //passed time

                let _start_process_milli_secondtime = get_now();
                self.passed_milli_secondtime = get_now() - self.start_milli_secondtime;

                //let _start_process_secondtime = get_now();

                // Draw boss

                self.bosses[0].draw(_context.clone());

                // Draw bullets

                self.bullets.iter().for_each(|b| b.draw(_context.clone()));

                // Draw shot

                for i in 0..self.shots.len() {
                    self.shots[i].draw(_context.clone());
                }
                // Draw Ship

                self.ship.draw(_context.clone());

                // Draw Time

                let _ = _context.set_font("28px myfont");
                let _ = _context.set_fill_style_str(LIGHT_GREEN_COLOR);
                let _str = get_passed_time(&(get_now() - self.start_milli_secondtime));
                let _ = _context.fill_text(&_str, 30.0, 50.0).unwrap();

                // Draw Number of Bullets

                let _ = _context.set_font("18px myfont");
                let _ = _context.set_fill_style_str(LIGHT_GREEN_COLOR);
                let _bullet_number = self.bullets.len();
                let _ = _context
                    .fill_text(
                        &format!("Bullets: {}", _bullet_number.to_string()),
                        30.0,
                        90.0,
                    )
                    .unwrap();

                // mesure max passed time
                self.max_passed_milli_secondtime_draw = max(
                    get_now() - _start_process_milli_secondtime,
                    self.max_passed_milli_secondtime_draw,
                );
            }
        }
    }
}

/**
 * get string from now()
 */
fn get_passed_time(passed_milli_secondtime: &i32) -> String {
    let _seconds = passed_milli_secondtime / 1000;
    let _mini_seconds = passed_milli_secondtime % 1000;
    format!("{:<02}.{:<02}", _seconds, _mini_seconds)
}

fn get_now() -> i32 {
    let _window = window().expect("no global `window` exists");
    let _performance = _window
        .performance()
        .expect("no global `performance` exists");
    _performance.now() as i32
}

/**
 * draw character
 */

fn draw_super_ship(ctx: CanvasRenderingContext2d, x: f32, y: f32) {
    {
        ctx.begin_path();
        ctx.set_fill_style_str(LIGHT_YELLOR_GREEN);
        ctx.move_to((17.0 + x).into(), y.into());
        ctx.line_to((25.0 + x).into(), y.into());
        ctx.line_to((25.0 + x).into(), (17.0 + y).into());
        ctx.line_to((30.0 + x).into(), (17.0 + y).into());
        ctx.line_to((30.0 + x).into(), (26.0 + y).into());
        ctx.line_to((42.0 + x).into(), (47.0 + y).into());
        ctx.line_to(x.into(), (47.0 + y).into());
        ctx.line_to((12.0 + x).into(), (26.0 + y).into());
        ctx.line_to((12.0 + x).into(), (17.0 + y).into());
        ctx.line_to((17.0 + x).into(), (17.0 + y).into());
        ctx.close_path();
        ctx.fill();
    }
    {
        ctx.begin_path();
        ctx.set_fill_style_str(DEFAULT_COLOR);
        ctx.move_to((17.0 + x).into(), (27.0 + y).into());
        ctx.line_to((24.0 + x).into(), (27.0 + y).into());
        ctx.line_to((24.0 + x).into(), (45.0 + y).into());
        ctx.line_to((17.0 + x).into(), (45.0 + y).into());
        ctx.close_path();
        ctx.fill();
    }
    {
        ctx.begin_path();
        ctx.set_fill_style_str(LIGHT_GREEN_COLOR);
        ctx.move_to((11.0 + x).into(), (47.0 + y).into());
        ctx.line_to((17.0 + x).into(), (47.0 + y).into());
        ctx.line_to((14.0 + x).into(), (56.0 + y).into());
        ctx.close_path();
        ctx.fill();
    }
    {
        ctx.begin_path();
        ctx.set_fill_style_str(LIGHT_GREEN_COLOR);
        ctx.move_to((26.0 + x).into(), (47.0 + y).into());
        ctx.line_to((32.0 + x).into(), (47.0 + y).into());
        ctx.line_to((29.0 + x).into(), (56.0 + y).into());
        ctx.close_path();
        ctx.fill();
    }
}

fn draw_ship(ctx: CanvasRenderingContext2d, x: f32, y: f32) {
    {
        ctx.begin_path();
        ctx.set_fill_style_str(DEFAULT_COLOR);
        ctx.move_to((17.0 + x).into(), y.into());
        ctx.line_to((25.0 + x).into(), y.into());
        ctx.line_to((25.0 + x).into(), (17.0 + y).into());
        ctx.line_to((30.0 + x).into(), (17.0 + y).into());
        ctx.line_to((30.0 + x).into(), (26.0 + y).into());
        ctx.line_to((42.0 + x).into(), (47.0 + y).into());
        ctx.line_to(x.into(), (47.0 + y).into());
        ctx.line_to((12.0 + x).into(), (26.0 + y).into());
        ctx.line_to((12.0 + x).into(), (17.0 + y).into());
        ctx.line_to((17.0 + x).into(), (17.0 + y).into());
        ctx.close_path();
        ctx.fill();
    }
    {
        ctx.begin_path();
        ctx.set_fill_style_str(LIGHT_GREEN_COLOR);
        ctx.move_to((17.0 + x).into(), (27.0 + y).into());
        ctx.line_to((24.0 + x).into(), (27.0 + y).into());
        ctx.line_to((24.0 + x).into(), (45.0 + y).into());
        ctx.line_to((17.0 + x).into(), (45.0 + y).into());
        ctx.close_path();
        ctx.fill();
    }
    {
        ctx.begin_path();
        ctx.set_fill_style_str(LIGHT_GREEN_COLOR);
        ctx.move_to((11.0 + x).into(), (47.0 + y).into());
        ctx.line_to((17.0 + x).into(), (47.0 + y).into());
        ctx.line_to((14.0 + x).into(), (56.0 + y).into());
        ctx.close_path();
        ctx.fill();
    }
    {
        ctx.begin_path();
        ctx.set_fill_style_str(LIGHT_GREEN_COLOR);
        ctx.move_to((26.0 + x).into(), (47.0 + y).into());
        ctx.line_to((32.0 + x).into(), (47.0 + y).into());
        ctx.line_to((29.0 + x).into(), (56.0 + y).into());
        ctx.close_path();
        ctx.fill();
    }
}

fn draw_boss(ctx: &CanvasRenderingContext2d, x: f32, y: f32) {
    {
        ctx.set_fill_style_str(GREEN_DARK_LIGHT);
        ctx.begin_path();
        ctx.move_to((55.0 + x).into(), y.into());
        ctx.line_to((165.0 + x).into(), y.into());
        ctx.line_to((225.0 + x).into(), (60.0 + y).into());
        ctx.line_to((225.0 + x).into(), (160.0 + y).into());
        ctx.line_to((160.0 + x).into(), (225.0 + y).into());
        ctx.line_to((55.0 + x).into(), (225.0 + y).into());
        ctx.line_to(x.into(), (160.0 + y).into());
        ctx.line_to(x.into(), (60.0 + y).into());
        ctx.close_path();
        ctx.fill();
    }
    {
        ctx.begin_path();
        ctx.set_fill_style_str(GREEN_DARK_LIGHT);
        ctx.move_to((55.0 + x).into(), y.into());
        ctx.line_to((165.0 + x).into(), y.into());
        ctx.line_to((225.0 + x).into(), (60.0 + y).into());
        ctx.line_to((225.0 + x).into(), (160.0 + y).into());
        ctx.line_to((160.0 + x).into(), (225.0 + y).into());
        ctx.line_to((55.0 + x).into(), (225.0 + y).into());
        ctx.line_to(x.into(), (160.0 + y).into());
        ctx.line_to(x.into(), (60.0 + y).into());
        ctx.close_path();
        ctx.fill();
    }
    {
        ctx.begin_path();
        ctx.set_stroke_style_str(DEFAULT_COLOR);
        ctx.move_to((55.0 + x).into(), (30.0 + y).into());
        ctx.line_to((165.0 + x).into(), (30.0 + y).into());
        ctx.line_to((195.0 + x).into(), (60.0 + y).into());
        ctx.line_to((195.0 + x).into(), (160.0 + y).into());
        ctx.line_to((160.0 + x).into(), (195.0 + y).into());
        ctx.line_to((55.0 + x).into(), (195.0 + y).into());
        ctx.line_to((26.0 + x).into(), (160.0 + y).into());
        ctx.line_to((26.0 + x).into(), (60.0 + y).into());
        ctx.close_path();
        ctx.stroke();
    }
    {
        ctx.begin_path();
        ctx.set_fill_style_str(DEFAULT_COLOR);
        ctx.set_line_width(2.0);
        ctx.move_to((85.0 + x).into(), (85.0 + y).into());
        ctx.line_to((140.0 + x).into(), (85.0 + y).into());
        ctx.line_to((140.0 + x).into(), (140.0 + y).into());
        ctx.line_to((85.0 + x).into(), (140.0 + y).into());
        ctx.close_path();
        ctx.stroke();
    }
    {
        ctx.begin_path();
        ctx.set_fill_style_str(DEFAULT_COLOR);
        let _ = ctx.arc(
            (112.0 + x).into(),
            (112.0 + y).into(),
            25.0,
            0.0,
            2.0 * PI as f64,
        );
        ctx.close_path();
        ctx.fill();
    }
    {
        ctx.begin_path();
        ctx.set_fill_style_str(LIGHT_GREEN_COLOR);
        let _ = ctx.arc(
            (112.0 + x).into(),
            (112.0 + y).into(),
            20.0,
            0.0,
            2.0 * PI as f64,
        );
        ctx.close_path();
        ctx.fill();
    }
    {
        ctx.begin_path();
        ctx.set_fill_style_str(DEFAULT_COLOR);
        ctx.set_line_width(2.0);
        ctx.move_to((70.0 + x).into(), (40.0 + y).into());
        ctx.line_to((152.0 + x).into(), (40.0 + y).into());
        ctx.line_to((140.0 + x).into(), (77.0 + y).into());
        ctx.line_to((80.0 + x).into(), (77.0 + y).into());
        ctx.close_path();
        ctx.fill();
    }
    {
        ctx.begin_path();
        ctx.set_fill_style_str(DEFAULT_COLOR);
        ctx.set_line_width(2.0);
        ctx.move_to((84.0 + x).into(), (145.0 + y).into());
        ctx.line_to((140.0 + x).into(), (145.0 + y).into());
        ctx.line_to((152.0 + x).into(), (180.0 + y).into());
        ctx.line_to((72.0 + x).into(), (180.0 + y).into());
        ctx.close_path();
        ctx.fill();
    }
    {
        ctx.begin_path();
        ctx.set_fill_style_str(DEFAULT_COLOR);
        ctx.set_line_width(2.0);
        ctx.move_to((147.0 + x).into(), (85.0 + y).into());
        ctx.line_to((183.0 + x).into(), (70.0 + y).into());
        ctx.line_to((183.0 + x).into(), (155.0 + y).into());
        ctx.line_to((147.0 + x).into(), (140.0 + y).into());
        ctx.close_path();
        ctx.fill();
    }
    {
        ctx.begin_path();
        ctx.set_fill_style_str(DEFAULT_COLOR);
        ctx.set_line_width(2.0);
        ctx.move_to((80.0 + x).into(), (85.0 + y).into());
        ctx.line_to((80.0 + x).into(), (140.0 + y).into());
        ctx.line_to((40.0 + x).into(), (155.0 + y).into());
        ctx.line_to((40.0 + x).into(), (70.0 + y).into());
        ctx.close_path();
        ctx.fill();
    }
}
