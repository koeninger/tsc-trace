use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::TextureCreator;
use sdl2::render::WindowCanvas;
use sdl2::ttf::Font;
use sdl2::video::WindowContext;
use std::env;
use std::path::Path;
use std::thread;
use std::time::Duration;
//use std::num::Saturating;

#[derive(Clone, Copy, Debug)]
pub struct Span {
    tag: u64,
    start: u64,
    stop: u64,
}

pub struct Area {
    y_start: u64,
    x_start: i32,
    y_stop: u64,
    x_stop: i32,
    tag_num: u64,
    //tag_name
}

pub struct App {
    window_width: u32,
    window_height: u32,
    background_color: Color,
    draw_zones: Vec<Area>,
    colors: Vec<Color>,
    muted_colors: Vec<Color>,
    sdl_context: sdl2::Sdl,
    canvas: WindowCanvas,
    /// span cycles per horizontal pixel
    scale: u64,
    /// vertical pixels per span
    span_height: i32,
    /// vertical pixels between spans
    span_spacing: i32,
    min_start: u64,
    max_stop: u64,
    scroll: i32,
    texture_creator: TextureCreator<WindowContext>,
}

impl App {
    pub fn new() -> Result<(Vec<Span>, App), String> {
        let mut spans = vec![
            Span {
                tag: 0,
                start: 100,
                stop: 10_000,
            },
            Span {
                tag: 1,
                start: 20_000,
                stop: 23_000,
            },
            Span {
                tag: 2,
                start: 10_000,
                stop: 13_000,
            },
            Span {
                tag: 0,
                start: 10_500,
                stop: 10_900,
            },
            Span {
                tag: 3,
                start: 12_000,
                stop: 13_000,
            },
            Span {
                tag: 8,
                start: 1_000,
                stop: 10_000,
            },
            Span {
                tag: 13,
                start: 100,
                stop: 110,
            },
            Span {
                tag: 13,
                start: 200,
                stop: 210,
            },
            Span {
                tag: 13,
                start: 500,
                stop: 507,
            },
            Span {
                tag: 13,
                start: 15_000,
                stop: 30_000,
            },
            Span {
                tag: 100,
                start: 1_000,
                stop: 20_000,
            },
        ];
        let draw_zones: Vec<Area> = vec![];
        assert!(
            !spans.is_empty(),
            "expected a non-empty array of trace spans"
        );
        spans.sort_unstable_by_key(|s| s.start);
        let min_start = spans[0].start;
        // TODO check and correct this on first iteration
        let max_stop = spans[spans.len() - 1].stop;
        let window_width = 800u32;
        let window_height = 600u32;
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        let scale = (max_stop - min_start) / window_width as u64;
        let window = video_subsystem
            .window("tsc-trace viewer", window_width, window_height)
            .position_centered()
            .opengl()
            .resizable()
            .build()
            .map_err(|e| e.to_string())?;
        let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        let texture_creator = canvas.texture_creator();

        Ok((
            spans,
            App {
                texture_creator,
                draw_zones,
                window_width,
                window_height,
                background_color: Color::RGB(192, 192, 192),
                colors: vec![
                    (255, 0, 0),
                    (0, 255, 0),
                    (0, 0, 255),
                    (255, 255, 0),
                    (0, 255, 255),
                    (255, 0, 255),
                ]
                .into_iter()
                .map(|c| Color::RGB(c.0, c.1, c.2))
                .collect(),
                muted_colors: vec![
                    (192, 0, 0),
                    (0, 192, 0),
                    (0, 0, 192),
                    (192, 192, 0),
                    (0, 192, 192),
                    (192, 0, 192),
                ]
                .into_iter()
                .map(|c| Color::RGB(c.0, c.1, c.2))
                .collect(),
                sdl_context,
                canvas,
                scale,
                span_height: 10,
                span_spacing: 1,
                min_start,
                max_stop,
                scroll: 0,
            },
        ))
    }

    fn draw_span(&mut self, span: &Span) {
        let x_sz = self.x_size(span);
        let scrolled_x = self.x_pos(span).saturating_sub(self.scroll);
        self.canvas.set_draw_color(if x_sz < 1 {
            self.colors[span.tag as usize % self.colors.len()]
        } else {
            self.muted_colors[span.tag as usize % self.muted_colors.len()]
        });
        self.canvas
            .fill_rect(Rect::new(
                scrolled_x,
                self.y_pos(span),
                x_sz,
                (self.span_height) as u32,
            ))
            .unwrap_or_else(|e| panic!("draw failure {e} for span {span:?}"));
        self.draw_zones.push(Area {
            y_start: self.y_pos(span) as u64,
            x_start: scrolled_x,
            y_stop: (self.y_pos(span) + self.span_height) as u64,
            x_stop: (scrolled_x + x_sz as i32),
            tag_num: span.tag,
        });
    }

    fn x_size(&self, span: &Span) -> u32 {
        ((span.stop - span.start) / (self.scale + 1))
            .try_into()
            .unwrap_or_else(|e| {
                panic!(
                    "bad x_size for scale {} span {:?} err {e}",
                    self.scale, span
                )
            })
    }

    fn x_pos(&self, span: &Span) -> i32 {
        // TODO scrolling viewport
        (span.start / (self.scale + 1))
            .try_into()
            .unwrap_or_else(|e| {
                panic!("bad x_pos for scale {} span {:?} err {e}", self.scale, span)
            })
    }

    fn y_pos(&self, span: &Span) -> i32 {
        let tag: i32 = span.tag.try_into()
            .expect("not intended to handle very high span tag cardinality, try filtering / renumbering first: {span}");
        ((self.span_spacing + self.span_height) * tag) + self.span_spacing
    }

    fn draw_text(
        canvas: &mut WindowCanvas,
        texture_creator: &TextureCreator<WindowContext>,
        font: &sdl2::ttf::Font,
        x: i32,
        y: i32,
        tag: i32,
        len: i32,
    ) -> Result<(), String> {
        let tag_text: String = format!("{0},{1}", tag.to_string(), len.to_string());
        let surface = font
            .render(&tag_text)
            .blended(Color::RGBA(0, 0, 0, 128))
            .map_err(|e| e.to_string())?;

        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())?;

        canvas.set_draw_color(Color::RGB(255,255,255));
        canvas.fill_rect(Rect::new(x, y, (tag_text.len()*20) as u32, 50 as u32))?;

        let target = Rect::new(x, y, (tag_text.len()*20) as u32, 50 as u32);
        canvas.copy(&texture, None, Some(target))?;

        Ok(())
    }

    pub fn run<'a>(&mut self, spans: Vec<Span>, font: Font<'a, 'static>) -> Result<(), String> {
        let mut event_pump = self.sdl_context.event_pump()?;
        let mut prev_keycount: i32 = 0;
        let mut keycount: i32 = 0;
        let mut keyspeed;
        let mut draw_x = 0;
        let mut draw_y = 0;
        let mut draw_tag = 0;
        let mut draw_len = 0;

        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'running,
                    Event::KeyDown {
                        keycode: Some(keycode),
                        ..
                    } => {
                        keycount += 1;
                        match keycount {
                            1..=45 => keyspeed = 2,
                            46..=90 => keyspeed = 6,
                            91..=i32::MAX => keyspeed = 10,
                            i32::MIN..=0_i32 => todo!(),
                        }
                        match keycode {
                            Keycode::Q => {
                                self.scale = self
                                    .scale
                                    .saturating_add((keyspeed / 2).try_into().unwrap())
                            } //plus
                            Keycode::W => {
                                self.scale = self
                                    .scale
                                    .saturating_sub((keyspeed / 2).try_into().unwrap())
                            } //minus
                            Keycode::E => {
                                self.scale =
                                    (self.max_stop - self.min_start) / (self.window_width as u64)
                            } //reset
                            Keycode::A => self.scroll = self.scroll.saturating_add(keyspeed),
                            Keycode::S => self.scroll = self.scroll.saturating_sub(keyspeed),
                            Keycode::D => self.scroll = 0,
                            _ => {}
                        }
                    }
                    Event::MouseButtonDown { x, y, .. } => {
                        for zone in &self.draw_zones {
                            if zone.x_start < x.try_into().unwrap()
                                && zone.x_stop > x.try_into().unwrap()
                                && zone.y_start < y.try_into().unwrap()
                                && zone.y_stop > y.try_into().unwrap()
                            {
                                draw_x = x;
                                draw_y = y;
                                draw_tag = zone.tag_num;
                                draw_len = (zone.x_stop - zone.x_start)*(<u64 as TryInto<i32>>::try_into(self.scale).unwrap()+1); // slightly inaccurate
                            }
                        }
                    }
                    Event::MouseButtonUp { .. } => {
                        draw_x = 0;
                        draw_y = 0;
                    }
                    _ => {}
                }
            }
            if prev_keycount == keycount {
                keycount = 0;
            }
            prev_keycount = keycount;

            self.canvas.set_draw_color(self.background_color);
            self.canvas.clear();
            self.draw_zones.clear();
            for span in &spans {
                self.draw_span(span);
            }
            if (draw_x > 0) && (draw_y > 0){
                Self::draw_text(
                &mut self.canvas,
                &self.texture_creator,
                &font,
                draw_x,
                draw_y,
                draw_tag.try_into().unwrap(),
                draw_len.try_into().unwrap(),
                )?;
            }
            self.canvas.present();
            thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        }

        Ok(())
    }
}

pub fn main() -> Result<(), String> {
    env::set_var("RUST_BACKTRACE", "1");
    let (spans, mut app) = App::new()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
    let font_path: &Path = Path::new(&"fonts/Opensans-Regular.ttf");
    let font = ttf_context.load_font(font_path, 128)?;
    println!("{}", app.scale);
    app.run(spans, font)
}
