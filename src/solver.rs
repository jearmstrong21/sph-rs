use cgmath::{Zero, InnerSpace};
use rand::Rng;

pub struct Parameters {
    pub gravity: cgmath::Vector2<f32>,
    pub rest_density: f32,
    pub gas_constant: f32,
    pub kernel_radius: f32,
    pub mass: f32,
    pub viscosity: f32,
    pub dt: f32,
    pub width: f32,
    pub height: f32,
}

pub struct Precomputed {
    poly6: f32,
    spiky_grad: f32,
    visc_lap: f32,
    kernel_radius_squared: f32,
}

impl Parameters {
    pub fn new() -> Parameters {
        Parameters {
            gravity: cgmath::vec2(0f32, -9.8 * 12000f32),
            rest_density: 1000f32,
            gas_constant: 2000f32,
            kernel_radius: 16f32,
            mass: 65f32,
            viscosity: 250f32,
            dt: 0.0008f32,
            width: 500f32,
            height: 500f32,
        }
    }
    pub fn precompute(&self) -> Precomputed {
        Precomputed {
            poly6: 315f32 / (65f32 * std::f32::consts::PI * self.kernel_radius.powf(9f32)),
            spiky_grad: -45f32 / (std::f32::consts::PI * self.kernel_radius.powf(6f32)),
            visc_lap: 45f32 / (std::f32::consts::PI * self.kernel_radius.powf(6f32)),
            kernel_radius_squared: self.kernel_radius * self.kernel_radius,
        }
    }
}

#[derive(Debug)]
pub struct Particle {
    pub x: cgmath::Vector2<f32>,
    pub v: cgmath::Vector2<f32>,
    pub f: cgmath::Vector2<f32>,
    pub rho: f32,
    pub p: f32,
}

impl Particle {
    pub fn new(x: f32, y: f32) -> Particle {
        Particle {
            x: cgmath::vec2(x, y),
            v: cgmath::Vector2::zero(),
            f: cgmath::Vector2::zero(),
            rho: 0f32,
            p: 0f32,
        }
    }
}

pub struct Simulation {
    pub particles: Vec<Particle>,
    pub parameters: Parameters,
    precomputed: Precomputed,
}

impl Simulation {
    pub fn new() -> Simulation {
        let parameters = Parameters::new();
        Simulation {
            particles: {
                let mut v = vec![];
                for i in 0..20 {
                    for j in 0..20 {
                        let x = (i as f32) * parameters.kernel_radius + 150f32 + rand::thread_rng().gen::<f32>();
                        let y = (j as f32) * parameters.kernel_radius + 150f32;
                        v.push(Particle::new(x, y))
                    }
                }
                v
            },
            precomputed: parameters.precompute(),
            parameters,
        }
    }

    fn density_pressure(&mut self) {
        for i in 0..self.particles.len() {
            self.particles[i].rho = 0f32;
            for j in 0..self.particles.len() {
                let rij: cgmath::Vector2<f32> = self.particles[j].x - self.particles[i].x;
                let r2 = rij.magnitude2();
                if r2 < self.precomputed.kernel_radius_squared {
                    self.particles[i].rho += self.parameters.mass * self.precomputed.poly6 * (self.precomputed.kernel_radius_squared - r2).powf(3f32)
                }
            }
            self.particles[i].p = self.parameters.gas_constant * (self.particles[i].rho - self.parameters.rest_density)
        }
    }

    fn forces(&mut self) {
        for i in 0..self.particles.len() {
            let mut press = cgmath::vec2(0f32, 0f32);
            let mut visc = cgmath::vec2(0f32, 0f32);
            for j in 0..self.particles.len() {
                if i == j {
                    continue;
                }
                let rij: cgmath::Vector2<f32> = self.particles[j].x - self.particles[i].x;
                let r = rij.magnitude();
                if r < self.parameters.kernel_radius {
                    press += -rij.normalize() * self.parameters.mass * (self.particles[i].p + self.particles[j].p) / (2f32 * self.particles[j].rho) * self.precomputed.spiky_grad * (self.parameters.kernel_radius - r).powf(2f32);
                    visc += self.parameters.viscosity * self.parameters.mass * (self.particles[j].v - self.particles[i].v) / self.particles[j].rho * self.precomputed.visc_lap * (self.parameters.kernel_radius - r)
                }
            }
            let grav = self.parameters.gravity * self.particles[i].rho;
            self.particles[i].f = press + visc + grav;
        }
    }

    fn integrate(&mut self) {
        for p in &mut self.particles {
            p.v += self.parameters.dt * p.f / p.rho;
            p.x += self.parameters.dt * p.v;
            if p.x.x < self.parameters.kernel_radius * 0.5f32 {
                p.v.x *= -0.5f32;
                p.x.x = self.parameters.kernel_radius * 0.5f32
            }
            if p.x.y < self.parameters.kernel_radius * 0.5f32 {
                p.v.y *= -0.5f32;
                p.x.y = self.parameters.kernel_radius * 0.5f32
            }
            if p.x.x > self.parameters.width - self.parameters.kernel_radius * 0.5f32 {
                p.v.x *= -0.5f32;
                p.x.x = self.parameters.width - self.parameters.kernel_radius * 0.5f32
            }
            if p.x.y > self.parameters.height - self.parameters.kernel_radius * 0.5f32 {
                p.v.y *= -0.5f32;
                p.x.y = self.parameters.height - self.parameters.kernel_radius * 0.5f32
            }
        }
    }

    pub fn update(&mut self) {
        self.density_pressure();
        self.forces();
        self.integrate()
    }
}