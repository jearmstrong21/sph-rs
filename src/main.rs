mod solver;

extern crate rgl;

use cgmath::num_traits::FloatConst;
use cgmath::{One, InnerSpace};
use crate::solver::Particle;

struct RenderState {
    shader: rgl::Shader,
    vertex_array: rgl::VertexArray,
    simulation: solver::Simulation,
    last_mouse_state: Option<cgmath::Vector2<f32>>,
}

fn main() {
    let mut rgl = rgl::GL::new().unwrap();
    println!("{:#?}", rgl.get_information());
    println!("{:?}", rgl.get_extensions());
    rgl.render_loop(|renderer| {
        RenderState {
            shader: renderer.shader(include_str!("shaders/shader.vert"), include_str!("shaders/shader.frag")).unwrap(),
            vertex_array: {
                let mut vertex_array = renderer.vertex_array();
                vertex_array.add_buffer();
                let detail = 20;
                let point = |i: f32| {
                    cgmath::vec2((f32::TAU() * i / detail as f32).cos(), (f32::TAU() * i / detail as f32).sin())
                };
                let mut pos = vec![];
                let mut indices = vec![];
                for i in 0..detail {
                    let p1 = point(i as f32);
                    let p2 = point(1f32 + i as f32);
                    pos.push(p1);
                    pos.push(p2);
                    pos.push(cgmath::vec2(0f32, 0f32));
                    indices.push(pos.len() as i32 - 3);
                    indices.push(pos.len() as i32 - 2);
                    indices.push(pos.len() as i32 - 1);
                }
                vertex_array.set_buffer(0, pos);
                vertex_array.set_indices(indices);
                vertex_array
            },
            simulation: solver::Simulation::new(),
            last_mouse_state: None,
        }
    }, |s: &mut RenderState, events: Vec<glfw::WindowEvent>, renderer: &mut rgl::Renderer| {
        let height = 500;
        let mut mouse = None;
        for e in events {
            match e {
                glfw::WindowEvent::CursorPos(x, y) => mouse = Some(cgmath::vec2(x as f32 * s.simulation.parameters.height / height as f32, (height as f64 - y) as f32 * s.simulation.parameters.height / height as f32)),
                _ => {}
            }
        }
        if let Some(mouse) = mouse {
            if let Some(old) = s.last_mouse_state {
                if renderer.get_mouse_button(glfw::MouseButtonLeft) == glfw::Action::Press {
                    let diff = (mouse - old).normalize_to(20000f32);
                    println!("{:?}", diff);
                    for p in &mut s.simulation.particles {
                        if (p.x - mouse).magnitude2() < 50f32 {
                            p.v += diff;
                        }
                    }
                }
            }
        }
        s.last_mouse_state = mouse;
        renderer.clear(cgmath::vec3(0.0, 0.25, 0.0));
        renderer.set_window_size((s.simulation.parameters.width / s.simulation.parameters.height * height as f32) as i32, height);
        s.simulation.update();
        s.shader.with(|uniforms| {
            let m = cgmath::ortho(0f32, s.simulation.parameters.width, 0f32, s.simulation.parameters.height, 0f32, 1f32); // * cgmath::Matrix4::from_translation(cgmath::vec3(50f32, 50f32, 0f32)) * cgmath::Matrix4::from_scale(10f32);
            // uniforms.set("b", &(renderer.get_time().cos() as f32 * 0.5 + 0.5));
            let mapper: fn(&Particle) -> f32 = |p| -p.rho;
            let min = s.simulation.particles.iter().map(mapper).min_by(|x, y| x.partial_cmp(y).unwrap()).unwrap();
            let max = s.simulation.particles.iter().map(mapper).max_by(|x, y| x.partial_cmp(y).unwrap()).unwrap();
            for particle in &s.simulation.particles {
                uniforms.set("b", &((mapper(particle) - min) / (max - min)));
                // uniforms.set("model", &(m * cgmath::Matrix4::from_translation(cgmath::vec3(particle.x.x, particle.x.y, 0f32)) * cgmath::Matrix4::from_scale(20f32)));
                uniforms.set("model", &(m * cgmath::Matrix4::from_translation(cgmath::vec3(particle.x.x, particle.x.y, 0f32)) * cgmath::Matrix4::from_scale(0.5f32 * s.simulation.parameters.kernel_radius)));
                s.vertex_array.render();
            }
        });
    })
}
