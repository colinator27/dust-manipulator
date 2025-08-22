use crate::rng::LinearRNG;

#[derive(Clone, Copy)]
pub struct BoundingBox {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32
}
impl BoundingBox {
    pub fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self { left, top, right, bottom }
    }
}

#[derive(Clone, Copy)]
pub struct Snowball {
    pub x: f32,
    pub y: f32,
    pub move_amount: i32
}
impl Snowball {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y, move_amount: 1 }
    }
    fn point_in_ellipse(left: i32, top: i32, right: i32, bottom: i32, x: i32, y: i32) -> bool {
        let dx: f32 = ((x as f32) - (((right + left) / 2)) as f32) / (((right - left) / 2) as f32);
        let dy: f32 = ((y as f32) - (((bottom + top) / 2)) as f32) / (((bottom - top) / 2) as f32);
        (dx * dx) + (dy * dy) <= 1.0
    }
    pub fn colliding_with(&self, bbox: &BoundingBox) -> bool {
        let radius: f32 = 2.0;
        let x1 = self.x - radius;
        let y1 = self.y - radius;
        let x2 = self.x + radius;
        let y2 = self.y + radius;
        if y2 < bbox.top as f32 {
            return false;
        }
        if y1 >= (bbox.bottom + 1) as f32 {
            return false;
        }
        if x1 >= (bbox.right + 1) as f32 {
            return false;
        }
        if x2 < bbox.left as f32 {
            return false;
        }
        let center_x = (x1 + x2) / 2.0;
        let center_y = (y1 + y2) / 2.0;
        if ((center_x < (bbox.left as f32) || (bbox.right as f32) < center_x)) &&
	        (center_y < (bbox.top as f32) || ((bbox.bottom as f32) < center_y)) {
            let ellipse_left: i32 = f32::round(x1) as i32;
            let ellipse_top: i32 = f32::round(y1) as i32;
            let ellipse_right: i32 = f32::round(x2) as i32;
            let ellipse_bottom: i32 = f32::round(y2) as i32;
            let check_x = if bbox.right as f32 <= center_x { bbox.right } else { bbox.left };
            let check_y = if bbox.bottom as f32 <= center_y { bbox.bottom } else { bbox.top };
            Snowball::point_in_ellipse(ellipse_left, ellipse_top, ellipse_right, ellipse_bottom, check_x, check_y)
        } else {
            true
        }
    }
    pub fn update(&mut self, mainchara_bbox: &BoundingBox, rng: &mut impl LinearRNG) {
        if self.colliding_with(mainchara_bbox) {
            self.move_amount = f32::floor(rng.next_f64(4.0) as f32) as i32 + 2;
        }
        if self.move_amount > 1 {
            if (mainchara_bbox.left as f32) > self.x {
                self.x -= self.move_amount as f32;
            }
            if (mainchara_bbox.right as f32) < self.x {
                self.x += self.move_amount as f32;
            }
            if (mainchara_bbox.top as f32) > self.y {
                self.y -= self.move_amount as f32;
            }
            if (mainchara_bbox.bottom as f32) < self.y {
                self.y += self.move_amount as f32;
            }
            
            self.x += ((rng.next_f64(self.move_amount as f64) as f32 - (self.move_amount as f32 / 2.0)) / 2.0) as f32;
            self.y += ((rng.next_f64(self.move_amount as f64) as f32 - (self.move_amount as f32 / 2.0)) / 2.0) as f32;
        
            self.move_amount -= 1;
        }
    }
}

#[derive(Clone)]
pub struct SnowArea {
    pub snowballs: Vec<Snowball>
}
impl SnowArea {
    pub fn new(x: f32, y: f32) -> Self {
        let x = x + 2.2;
        let y = y + 1.0;
        let mut snowballs = Vec::with_capacity(if x < 136.0 { 5 } else { 25 });
        let mut yy = 0;
        let mut xx = 0;
        while yy < 5 {
            let snowball_x: f32 = match xx {
                4 => (x + (xx as f32 * 4.0)) - 0.2,
                _ => x + (xx as f32 * 4.0)
            };
            let snowball_y: f32 = y + (yy as f32 * 4.0);
            if snowball_x >= 136.0 {
                snowballs.push(Snowball::new(snowball_x, snowball_y));
            }
            
            if xx == 4 {
                xx = -1;
                yy += 1;
            }
            xx += 1;
        }
        Self { snowballs }
    }
    pub fn update(&mut self, mainchara_bbox: &BoundingBox, rng: &mut impl LinearRNG) {
        for snowball in self.snowballs.iter_mut() {
            snowball.update(mainchara_bbox, rng);
        }
    }
    pub fn new_array() -> [SnowArea; 8] { 
        [
            SnowArea::new(120.0, 400.0),
            SnowArea::new(140.0, 400.0),
            SnowArea::new(120.0, 420.0),
            SnowArea::new(140.0, 420.0),
            SnowArea::new(120.0, 380.0),
            SnowArea::new(140.0, 380.0),
            SnowArea::new(120.0, 440.0),
            SnowArea::new(140.0, 440.0)
        ]
    }
    pub fn simulate_array(arr: &mut [SnowArea; 8], rng: &mut impl LinearRNG) {
        let mut mainchara_bbox = BoundingBox::new( 140, 367, 159, 377);
        for _ in 0..40 {
            mainchara_bbox.top += 3;
            mainchara_bbox.bottom += 3;

            for snow_area in arr.iter_mut().rev() {
                snow_area.update(&mainchara_bbox, rng);
            }
        }
    }
}

#[derive(Clone, Copy)]
struct SimulatedSnowball {
    pub x: u8,
    pub y: u8
}

pub const SNOWBALLS_ORIGIN_X: i32 = 150;
pub const SNOWBALLS_ORIGIN_Y: i32 = 360;

#[derive(Clone)]
pub struct SnowballSimulator {
    pub snowballs: [Snowball; 120]
}
impl SnowballSimulator {
    pub fn new() -> Self {
        Self {
            snowballs: [
                Snowball::new(142.2, 441.0), Snowball::new(146.2, 441.0), Snowball::new(150.2, 441.0), Snowball::new(154.2, 441.0), Snowball::new(158.0, 441.0), 
                Snowball::new(142.2, 445.0), Snowball::new(146.2, 445.0), Snowball::new(150.2, 445.0), Snowball::new(154.2, 445.0), Snowball::new(158.0, 445.0),
                Snowball::new(142.2, 449.0), Snowball::new(146.2, 449.0), Snowball::new(150.2, 449.0), Snowball::new(154.2, 449.0), Snowball::new(158.0, 449.0), 
                Snowball::new(142.2, 453.0), Snowball::new(146.2, 453.0), Snowball::new(150.2, 453.0), Snowball::new(154.2, 453.0), Snowball::new(158.0, 453.0), 
                Snowball::new(142.2, 457.0), Snowball::new(146.2, 457.0), Snowball::new(150.2, 457.0), Snowball::new(154.2, 457.0), Snowball::new(158.0, 457.0), 
                Snowball::new(138.0, 441.0), Snowball::new(138.0, 445.0), Snowball::new(138.0, 449.0), Snowball::new(138.0, 453.0), Snowball::new(138.0, 457.0), 
                Snowball::new(142.2, 381.0), Snowball::new(146.2, 381.0), Snowball::new(150.2, 381.0), Snowball::new(154.2, 381.0), Snowball::new(158.0, 381.0), 
                Snowball::new(142.2, 385.0), Snowball::new(146.2, 385.0), Snowball::new(150.2, 385.0), Snowball::new(154.2, 385.0), Snowball::new(158.0, 385.0), 
                Snowball::new(142.2, 389.0), Snowball::new(146.2, 389.0), Snowball::new(150.2, 389.0), Snowball::new(154.2, 389.0), Snowball::new(158.0, 389.0), 
                Snowball::new(142.2, 393.0), Snowball::new(146.2, 393.0), Snowball::new(150.2, 393.0), Snowball::new(154.2, 393.0), Snowball::new(158.0, 393.0), 
                Snowball::new(142.2, 397.0), Snowball::new(146.2, 397.0), Snowball::new(150.2, 397.0), Snowball::new(154.2, 397.0), Snowball::new(158.0, 397.0), 
                Snowball::new(138.0, 381.0), Snowball::new(138.0, 385.0), Snowball::new(138.0, 389.0), Snowball::new(138.0, 393.0), Snowball::new(138.0, 397.0), 
                Snowball::new(142.2, 421.0), Snowball::new(146.2, 421.0), Snowball::new(150.2, 421.0), Snowball::new(154.2, 421.0), Snowball::new(158.0, 421.0), 
                Snowball::new(142.2, 425.0), Snowball::new(146.2, 425.0), Snowball::new(150.2, 425.0), Snowball::new(154.2, 425.0), Snowball::new(158.0, 425.0), 
                Snowball::new(142.2, 429.0), Snowball::new(146.2, 429.0), Snowball::new(150.2, 429.0), Snowball::new(154.2, 429.0), Snowball::new(158.0, 429.0), 
                Snowball::new(142.2, 433.0), Snowball::new(146.2, 433.0), Snowball::new(150.2, 433.0), Snowball::new(154.2, 433.0), Snowball::new(158.0, 433.0), 
                Snowball::new(142.2, 437.0), Snowball::new(146.2, 437.0), Snowball::new(150.2, 437.0), Snowball::new(154.2, 437.0), Snowball::new(158.0, 437.0), 
                Snowball::new(138.0, 421.0), Snowball::new(138.0, 425.0), Snowball::new(138.0, 429.0), Snowball::new(138.0, 433.0), Snowball::new(138.0, 437.0), 
                Snowball::new(142.2, 401.0), Snowball::new(146.2, 401.0), Snowball::new(150.2, 401.0), Snowball::new(154.2, 401.0), Snowball::new(158.0, 401.0), 
                Snowball::new(142.2, 405.0), Snowball::new(146.2, 405.0), Snowball::new(150.2, 405.0), Snowball::new(154.2, 405.0), Snowball::new(158.0, 405.0), 
                Snowball::new(142.2, 409.0), Snowball::new(146.2, 409.0), Snowball::new(150.2, 409.0), Snowball::new(154.2, 409.0), Snowball::new(158.0, 409.0), 
                Snowball::new(142.2, 413.0), Snowball::new(146.2, 413.0), Snowball::new(150.2, 413.0), Snowball::new(154.2, 413.0), Snowball::new(158.0, 413.0), 
                Snowball::new(142.2, 417.0), Snowball::new(146.2, 417.0), Snowball::new(150.2, 417.0), Snowball::new(154.2, 417.0), Snowball::new(158.0, 417.0), 
                Snowball::new(138.0, 401.0), Snowball::new(138.0, 405.0), Snowball::new(138.0, 409.0), Snowball::new(138.0, 413.0), Snowball::new(138.0, 417.0)
            ]
        }
    }
    pub fn simulate(&self, rng: &impl LinearRNG, output_position_data: &mut Vec<u8>) {
        let mut rng = rng.clone();
        let mut simulation_snowballs = self.snowballs.clone();
        let mut mainchara_bbox = BoundingBox::new( 140, 367, 159, 377);
        for _ in 0..40 {
            mainchara_bbox.top += 3;
            mainchara_bbox.bottom += 3;

            for snowball in simulation_snowballs.iter_mut() {
                snowball.update(&mainchara_bbox, &mut rng);
            }
        }
        let mut simulated_snowballs: Vec<SimulatedSnowball> = Vec::with_capacity(120);
        for snowball in simulation_snowballs.iter() {
            let snowball_x = f32::round(snowball.x) as i32;
            let snowball_y = f32::round(snowball.y) as i32;
            if snowball_x < SNOWBALLS_ORIGIN_X || snowball_y < SNOWBALLS_ORIGIN_Y || snowball_x >= SNOWBALLS_ORIGIN_X + 250 || snowball_y >= SNOWBALLS_ORIGIN_Y + 250 {
                continue;
            }
            let snowball_x = (snowball_x - SNOWBALLS_ORIGIN_X) as u8;
            let snowball_y = (snowball_y - SNOWBALLS_ORIGIN_Y) as u8;
            simulated_snowballs.push(SimulatedSnowball { x: snowball_x, y: snowball_y });
        }
        simulated_snowballs.sort_unstable_by(|a, b| b.x.cmp(&a.x));

        const SNOWBALL_CAPACITY: usize = 64;
        let mut num_snowballs_output = 0;
        for simulated_snowball in simulated_snowballs.iter() {
            output_position_data.push(simulated_snowball.x);
            output_position_data.push(simulated_snowball.y);
            num_snowballs_output += 1;
            if num_snowballs_output >= SNOWBALL_CAPACITY {
                break;
            }
        }
        for _ in num_snowballs_output..SNOWBALL_CAPACITY {
            output_position_data.push(0);
            output_position_data.push(0);
        }
    }
    pub fn simulate_range<F>(&self, rng: &impl LinearRNG, range: usize, output_position_data: &mut Vec<u8>, should_abort: F) where F: Fn() -> bool  {
        let mut rng = rng.clone();
        for _ in 0..range {
            self.simulate(&rng, output_position_data);
            _ = rng.next_u32();

            if should_abort() {
                return;
            }
        }
    }
}