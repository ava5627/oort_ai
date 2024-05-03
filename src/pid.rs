use oort_api::prelude::TICK_LENGTH;
pub struct PID {
    pub p: f64,
    pub i: f64,
    pub d: f64,
    pub last_error: Option<f64>,
    pub integral: f64,
    pub integral_limit: f64,
    pub output_limit: f64,
    pub delta_time: f64,
}
impl PID {
    pub fn new(p: f64, i: f64, d: f64, integral_limit: f64, output_limit: f64) -> PID {
        PID {
            p,
            i,
            d,
            last_error: None,
            integral: 0.0,
            integral_limit,
            output_limit,
            delta_time: TICK_LENGTH,
        }
    }
    pub fn update(&mut self, error: f64) -> f64 {
        let p = self.p * error;
        self.integral += error * self.delta_time;
        self.integral = self
            .integral
            .clamp(-self.integral_limit, self.integral_limit);
        let i = self.i * self.integral;
        let derivative = match self.last_error {
            Some(last_error) => (error - last_error) / self.delta_time,
            None => 0.0,
        };
        self.last_error = Some(error);
        let d = self.d * derivative;
        let output = p + i + d;
        output.clamp(-self.output_limit, self.output_limit)
    }
    pub fn reset(&mut self) {
        self.last_error = None;
        self.integral = 0.0;
    }
}
