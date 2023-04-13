use macroquad::prelude::*;

pub enum TextPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight
}

#[derive(Default)]
pub struct DebugText {
    current_font_size: f32,
    current_pos_top_left: Vec2,
    current_pos_top_right: Vec2,
    current_pos_bottom_left: Vec2,
    current_pos_bottom_right: Vec2,
    current_padding: f32
}

impl DebugText {

    fn next_text_position(&mut self, text: &str, pos: TextPosition) -> Vec2 {

        let text_size = measure_text(text, None, self.current_font_size as u16, 1.0);

        match pos {
            TextPosition::TopLeft => {
                let last_text_position = self.current_pos_top_left;
                self.current_pos_top_left += vec2(0.0, self.current_font_size);
                last_text_position
            },
            TextPosition::TopRight => {
                let last_text_position = self.current_pos_top_right;
                self.current_pos_top_right += vec2(0.0, self.current_font_size);
                last_text_position - vec2(text_size.width, 0.0)
            },
            TextPosition::BottomLeft => {
                let last_text_position = self.current_pos_bottom_left;
                self.current_pos_bottom_left -= vec2(0.0, self.current_font_size);
                last_text_position
            },
            TextPosition::BottomRight => {
                let last_text_position = self.current_pos_bottom_right;
                self.current_pos_bottom_right -= vec2(0.0, self.current_font_size);
                last_text_position - vec2(text_size.width, 0.0)
            }
        }
        
    }

    pub fn new() -> DebugText {
        DebugText {
            current_font_size: 16.0,
            current_padding: 32.0,
            ..Default::default()
        }
    }

    pub fn draw_text<S>(&mut self, text: S, pos: TextPosition, color: Color)
        where S: Into<String>
    {

        push_camera_state();
        set_default_camera();

        let text = &text.into();
        let next_position = self.next_text_position(text, pos);
        draw_text(text, next_position.x, next_position.y, self.current_font_size, color);

        pop_camera_state();

    }
    
    pub fn new_frame(&mut self) {

        self.current_pos_top_left = vec2(self.current_padding, self.current_padding);
        self.current_pos_top_right = vec2(screen_width() - self.current_padding, self.current_padding);
        self.current_pos_bottom_left = vec2(self.current_padding, screen_height() - self.current_padding);
        self.current_pos_bottom_right = vec2(screen_width() - self.current_padding, screen_height() - self.current_padding);

    }

}

pub trait BenchmarkWithDebugText {
    fn benchmark_execution<F>(&mut self, f: F, text: &str, pos: TextPosition, color: Color)
        where F: FnMut();
}

impl BenchmarkWithDebugText for DebugText {
    fn benchmark_execution<F>(&mut self, mut f: F, text: &str, pos: TextPosition, color: Color) 
        where F: FnMut()
    {

        let start_time = get_time();
        f();
        let end_time = get_time();

        let elapsed_time_ms = (end_time - start_time) * 1000.0;
        self.draw_text(format!("{} - {:.2} ms", text, elapsed_time_ms), pos, color);
        
    }
}

#[macro_export]
macro_rules! benchmark_execution {
    ($debug_text:expr, $x:expr) => {
        $debug_text.benchmark_execution(|| $x, stringify!($x).split_once('(').unwrap().0, TextPosition::TopRight, BLACK);
    };
    ($debug_text:expr, $x:expr, $n:expr) => {
        $debug_text.benchmark_execution(|| $x, $n, TextPosition::TopRight, BLACK);
    };
}