//!  based on https://github.com/LoipesMas/rust-compute-shaders/blob/main/mold/src/main.rs
//!

use std::sync::Arc;

use eframe::{
    glow::{self, Buffer, HasContext, Program},
    CreationContext,
};
use egui::CentralPanel;
use log::info;

pub struct App {
    gl: Arc<eframe::glow::Context>,
    prog: Program,
    data: Buffer,
    res: Option<Vec<f32>>,
    n: usize,
}

static SHADER_SOURCE: &str = include_str!("test.glsl");

// TODO: Use glium abstractions

fn set_buf<T>(gl: &eframe::glow::Context, buf_gl: &Buffer, buf_cpu: &Vec<T>) {
    unsafe {
        let buf_u8_slice: &[u8] = core::slice::from_raw_parts(
            buf_cpu.as_ptr() as *const u8,
            buf_cpu.len() * core::mem::size_of::<T>(),
        );

        gl.bind_buffer(glow::SHADER_STORAGE_BUFFER, Some(*buf_gl));
        gl.buffer_data_u8_slice(
            glow::SHADER_STORAGE_BUFFER,
            buf_u8_slice,
            glow::DYNAMIC_COPY,
        );
    }
}

fn get_buf<T>(gl: &eframe::glow::Context, buf_gl: &Buffer, buf_cpu: &mut Vec<T>) {
    unsafe {
        let buf_u8_slice: &mut [u8] = core::slice::from_raw_parts_mut(
            buf_cpu.as_ptr() as *mut u8,
            buf_cpu.len() * core::mem::size_of::<T>(),
        );

        gl.bind_buffer(glow::SHADER_STORAGE_BUFFER, Some(*buf_gl));
        gl.get_buffer_sub_data(glow::SHADER_STORAGE_BUFFER, 0, buf_u8_slice);
    }
}

const N: usize = 128;

impl App {
    pub fn new(cc: &CreationContext) -> Self {
        info!("Getting GL context");
        let gl = cc.gl.as_ref().unwrap().clone();
        let n = N;

        info!("Setting up GL Buffer");
        let data = unsafe {
            let buf_cpu: Vec<_> = (0..n).map(|i| 1.1 * (i as f32)).collect();
            let buf_gl = gl.create_buffer().unwrap();
            set_buf(&gl, &buf_gl, &buf_cpu);
            buf_gl
        };

        info!("Building GL Prog");
        let prog = unsafe {
            let compute_program = gl.create_program().expect("Cannot create program");

            // compute shaders not actually supported in WebGL it seems, so this fails under WASM
            let shader = gl.create_shader(glow::COMPUTE_SHADER).unwrap();
            gl.shader_source(shader, SHADER_SOURCE);
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                panic!("{}", gl.get_shader_info_log(shader));
            }
            gl.attach_shader(compute_program, shader);
            gl.link_program(compute_program);
            if !gl.get_program_link_status(compute_program) {
                panic!("{}", gl.get_program_info_log(compute_program));
            }
            gl.detach_shader(compute_program, shader);
            gl.delete_shader(shader);

            compute_program
        };

        Self {
            gl,
            prog,
            data,
            res: None,
            n,
        }
    }

    fn compute(&self) {
        let gl = &self.gl;
        unsafe {
            gl.memory_barrier(glow::SHADER_STORAGE_BARRIER_BIT);
            gl.use_program(Some(self.prog));
            gl.bind_buffer_base(glow::SHADER_STORAGE_BUFFER, 0, Some(self.data));
            gl.dispatch_compute(self.n as u32 / 8, 1, 1);
            gl.memory_barrier(glow::SHADER_STORAGE_BARRIER_BIT);
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            if ui.button("test").clicked() {
                self.compute();
                let mut res: Vec<f32> = vec![0.0; self.n];
                get_buf(&self.gl, &self.data, &mut res);
                self.res = Some(res);
            }

            if let Some(res) = &self.res {
                ui.label(format!("{:?}", res).as_str());
            }
        });
    }
}
