use macroquad::time::get_time;

pub fn get_time_ms() -> f32 {
    (get_time() * 1000.0) as f32
}

pub fn measure_function<F>(measured_function: F) -> f32
    where F: FnOnce() -> ()
{
    let start = get_time_ms();
    measured_function();
    get_time_ms() - start
}

pub fn measure_scope<F>(callback_function: F) -> StopWatch<F>
    where F: FnMut(f32) -> ()
{
    StopWatch::start(callback_function)
}

pub struct StopWatch<F: FnMut(f32) -> ()> {
    start_ms: f32,
    callback: F
}

impl<F> StopWatch<F> where F: FnMut(f32) -> () {
    pub fn start(f: F) -> StopWatch<F> {
        StopWatch {
            start_ms: get_time_ms(),
            callback: f
        }
    }
}

impl<F> Drop for StopWatch<F> where F: FnMut(f32) -> () {
    fn drop(&mut self) {
        let total_time = get_time_ms() - self.start_ms;
        (self.callback)(total_time);
    }
}

#[macro_export]
macro_rules! measure_scope {
    ($e:expr) => {
        let time = crate::utils::measure::measure_scope(|ms| $e = ms);
    };
}