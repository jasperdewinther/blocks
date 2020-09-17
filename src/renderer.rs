use glium::{Display, Program, DrawParameters, Frame, VertexBuffer, Surface, glutin, Blend};
use std::time::SystemTime;
use crate::player::Player;
use std::f32::consts::PI;
use glium::backend::glutin::glutin::event_loop::EventLoop;
use crate::constants::{WIDTH, HEIGHT};

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

pub type Color = [f32; 4];


pub struct DrawInfo<'a>{
    pub display: Display,
    pub program: Program,
    pub program_start: SystemTime,
    pub draw_params: DrawParameters<'a>,
}

pub fn draw_vertices(draw_info: &mut DrawInfo, target: &mut Frame, vertex_buffer: &VertexBuffer<Vertex>, player: &Player){
    let utime: f32 = draw_info.program_start.elapsed().unwrap().as_secs_f32();
    let perspective = {
        let (width, height) = target.get_dimensions();
        let aspect_ratio = height as f32 / width as f32;

        let fov: f32 = PI / 3.0;
        let zfar = 1024.0;
        let znear = 0.1;

        let f = 1.0 / (fov / 2.0).tan();

        [
            [f *   aspect_ratio   ,    0.0,              0.0              ,   0.0],
            [         0.0         ,     f ,              0.0              ,   0.0],
            [         0.0         ,    0.0,  (zfar+znear)/(zfar-znear)    ,   1.0],
            [         0.0         ,    0.0, -(2.0*zfar*znear)/(zfar-znear),   0.0],
        ]
    };
    let view = player.get_view_matrix();

    let uniforms = uniform! {
            matrix: [
                [0.1, 0.0, 0.0, 0.0],
                [0.0, 0.1, 0.0, 0.0],
                [0.0, 0.0, 0.1, 0.0],
                [0.0, 0.0, 1.0, 1.0f32]
            ],
            view: view,
            time: utime,
            perspective: perspective
        };
    // drawing a frame
    match target.draw(vertex_buffer, glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList), &draw_info.program, &uniforms, &draw_info.draw_params){
        Ok(_) => (),
        Err(err) => println!("{}", err.to_string())
    }
}

pub fn create_display(event_loop: &EventLoop<()>) -> Display {
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new().with_depth_buffer(23)
        .with_vsync(true)
        .with_multisampling(2);
    return glium::Display::new(wb, cb, &event_loop).unwrap();
}
pub fn gen_program(display: &Display) -> Program {
    let program = program!(display,
        140 => {
            vertex: "
                #version 140
                mat4 rotationX( in float angle ) {
                return mat4(	1.0,		0,			0,			0,
                                0, 	cos(angle),	-sin(angle),		0,
                                0, 	sin(angle),	 cos(angle),		0,
                                0, 			0,			  0, 		1);
                }
                mat4 rotationY( in float angle ) {
                    return mat4(	cos(angle),		0,		sin(angle),	0,
                                            0,		1.0,			 0,	0,
                                    -sin(angle),	0,		cos(angle),	0,
                                            0, 		0,				0,	1);
                }
                mat4 rotationZ( in float angle ) {
                    return mat4(	cos(angle),		-sin(angle),	0,	0,
                                    sin(angle),		cos(angle),		0,	0,
                                            0,				0,		1,	0,
                                            0,				0,		0,	1);
                }
                #define PI 3.1415926535897932384626433832795
                uniform mat4 perspective;
                uniform mat4 matrix;
                uniform mat4 view;
                uniform float time;
                in vec3 position;
                in vec4 color;
                out vec4 vColor;
                void main() {
                    gl_Position = vec4(position, 1.0);
                    gl_Position = perspective * view * gl_Position;
                    vColor = color;
                }
            ",
            fragment: "
                #version 140
                uniform float time;
                in vec4 vColor;
                out vec4 f_color;
                void main() {
                    f_color = vColor;
                }
            "
        },
    ).unwrap();
    return program;
}

pub fn gen_draw_params() -> DrawParameters<'static>{
    glium::DrawParameters {
        depth: glium::Depth {
            test: glium::DepthTest::IfLess,
            write: true,
            .. Default::default()
        },
        backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
        blend: Blend::alpha_blending(),
        .. Default::default()
    }
}