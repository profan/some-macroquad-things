use macroquad::math::Vec2;

const SHOULD_INTERPOLATE: bool = false;

pub struct AverageLine2D {
    points: Vec<Vec2>,
    min_distance: f32
}

impl AverageLine2D {

    pub fn new(min_distance: f32) -> AverageLine2D {
        AverageLine2D {
            points: Vec::new(),
            min_distance
        }
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    pub fn points(&self) -> &Vec<Vec2> {
        &self.points
    }

    pub fn length(&self) -> usize {
        self.points.len()
    }

    pub fn euclidean_length(&self) -> f32 {

        if self.points.is_empty() {
            return 0.0
        }

        let mut d_sum = 0.0;
        let mut last_point = self.points[0];
        for &current_point in self.points.iter().skip(1) {
            d_sum += current_point.distance(last_point);
            last_point = current_point;
        }
        d_sum

    }

    pub fn add_point(&mut self, point: Vec2) {

        if self.points.len() > 0 {
            
            let last_point = self.points[self.points.len() - 1];
            let current_point_distance = point.distance(last_point);
            
            if current_point_distance >= self.min_distance {

                if SHOULD_INTERPOLATE {

                    let n = (current_point_distance / self.min_distance).max(1.0);

                    for i in 0..n as i32 {
                        let p = last_point.lerp(point, (n * i as f32) / current_point_distance);
                        self.points.push(p);
                    }

                } else {

                    self.points.push(point);

                }


            }

        } else {
            self.points.push(point);
        }

    }

    pub fn get_point(&self, fraction: f32) -> Vec2 {

        if self.is_empty() {
            panic!("should never be empty when trying to get a point on the line!");
        } else if self.length() == 1 {
            return self.points[0]
        }

        // let current_length = self.euclidean_length();

        // let mut last_point = self.points[0];
        // let mut current_point = self.points[1];
        // let mut current_distance = 0.0;

        // while current_distance < fraction {

        // }

        let current_index = (self.points.len() as f32 * fraction) as usize;
        self.points[current_index]

    }

    pub fn clear_points(&mut self) {
        self.points.clear()
    }

}