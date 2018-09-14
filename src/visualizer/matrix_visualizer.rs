use failure;
use glium::{glutin, index, texture, Display, Program, Surface, VertexBuffer};
use ndarray::{ArrayBase, Dim, OwnedRepr};
use std::fs::File;
use std::io;
use std::io::prelude::*;

/// 直交座標系(XY座標系)を用いてvisualizeする構造体
pub struct MatrixVisualizer {
    program: Program,
    events_loop: glutin::EventsLoop,
    vertex_buffer: VertexBuffer<Vertex>,
    indices: index::NoIndices,
    display: Display,
}

impl MatrixVisualizer {
    /// MatrixVisualizerインスタンスを生成する
    ///
    /// # Arguments
    /// * `title` - ウィンドウに表示するタイトル
    /// * `vertex_glsl_path` - バーテックスシェーダーのファイルを格納しているpath
    /// * `grafic_glsl_path` - グラフィックシェーダーのファイルを格納しているpath
    ///
    /// # Example
    /// ``````
    /// use my_alife::visualizer::matrix_visualizer::MatrixVisualizer;
    /// let matrix_visualize = MatrixVisualizer::new(
    ///   "Gray Scott",
    ///   "res/shaders/matrix_visualizer_vertex.glsl",
    ///   "res/shaders/matrix_visualizer_fragment.glsl",
    /// ).unwrap();
    ///
    ///
    pub fn new(
        title: &str,
        vertex_glsl_path: &str,
        faragment_glsl_path: &str,
    ) -> Result<MatrixVisualizer, failure::Error> {
        let events_loop = glutin::EventsLoop::new();
        let window = glutin::WindowBuilder::new()
            .with_dimensions((600, 600).into())
            .with_title(title);
        let context = glutin::ContextBuilder::new();
        let display = Display::new(window, context, &events_loop).unwrap();
        let program = Program::from_source(
            &display,
            &Self::glsl(vertex_glsl_path)?,
            &Self::glsl(faragment_glsl_path)?,
            None,
        )?;

        let vertex_buffer = VertexBuffer::new(&display, &Self::shape()).unwrap();
        Ok(MatrixVisualizer {
            program: program,
            events_loop: events_loop,
            vertex_buffer: vertex_buffer,
            indices: index::NoIndices(index::PrimitiveType::TrianglesList),
            display: display,
        })
    }

    fn glsl(path: &str) -> Result<String, io::Error> {
        let mut contents = String::new();
        File::open(path)?.read_to_string(&mut contents)?;
        Ok(contents)
    }

    fn shape() -> Vec<Vertex> {
        let vertex1 = Vertex {
            a_position: [-1.0, -1.0],
            a_texcoord: [0.0, 1.0],
        };
        let vertex2 = Vertex {
            a_position: [1.0, -1.0],
            a_texcoord: [1.0, 1.0],
        };
        let vertex3 = Vertex {
            a_position: [1.0, 1.0],
            a_texcoord: [1.0, 0.0],
        };
        let vertex4 = Vertex {
            a_position: [-1.0, -1.0],
            a_texcoord: [0.0, 1.0],
        };
        let vertex5 = Vertex {
            a_position: [-1.0, 1.0],
            a_texcoord: [0.0, 0.0],
        };
        let vertex6 = Vertex {
            a_position: [1.0, 1.0],
            a_texcoord: [1.0, 0.0],
        };
        vec![vertex1, vertex2, vertex3, vertex4, vertex5, vertex6]
    }

    /// 実際に描画を行う
    ///
    /// # Arguments
    /// * `initail_state` - 初期状態
    /// * `unpdate_fn` - 初期状態をどのように変更するかの関数
    ///
    /// # Example
    /// ```
    /// extern crate ndarray;
    /// extern crate my_alife;
    ///
    /// use my_alife::visualizer::matrix_visualizer::{Matrix, MatrixVisualizer};
    /// use ndarray::Array2;
    ///
    /// let matrix = MatrixVisualizer::new(
    ///     "Gray Scott",
    ///     "res/shaders/matrix_visualizer_vertex.glsl",
    ///     "res/shaders/matrix_visualizer_fragment.glsl",
    /// );
    /// let initial_state = (Array2::<f32>::ones((256, 256)), Array2::<f32>::ones((256, 256)));
    /// fn update_nothing(uv: &mut (Matrix<f32>, Matrix<f32>)) -> &Matrix<f32> {
    ///   &uv.0
    /// }
    ///
    /// matrix.unwrap().draw(initial_state, update_nothing);
    ///
    ///
    /// ```
    pub fn draw<T, F>(mut self, mut initial_state: T, mut update_fn: F) -> Result<(), failure::Error>
    where
        F: FnMut(&mut T) -> &Matrix<f32>,
    {
        let mut closed = false;
        loop {
            if closed {
                break;
            }
            let u = update_fn(&mut initial_state);
            let image = make_texture_image(u);
            let texture = texture::Texture2d::new(&self.display, image).unwrap();
            let mut target = self.display.draw();
            target.clear_color(1.0, 0.0, 0.0, 1.0);
            target.draw(
                &self.vertex_buffer,
                &self.indices,
                &self.program,
                &uniform! {u_texture: texture.sampled()},
                &Default::default(),
            )?;
            target.finish()?;

            self.events_loop.poll_events(|event| {
                if let glutin::Event::WindowEvent { event, .. } = event {
                    if let glutin::WindowEvent::CloseRequested = event {
                        closed = true
                    }
                }
            });
        }
        Ok(())
    }
}

/// 直交座標系(XY座標系)においてどの座標にどんな色(グレースケール)を表示するかを表現する。  
/// 実体は2次元配列
pub type Matrix<T> = ArrayBase<OwnedRepr<T>, Dim<[usize; 2]>>;

#[derive(Copy, Clone)]
struct Vertex {
    a_position: [f32; 2],
    a_texcoord: [f32; 2],
}
implement_vertex!(Vertex, a_position, a_texcoord);


fn make_texture_image<'a>(u: &Matrix<f32>) -> texture::RawImage2d<'a, u8> {
    let mut texture_data = Vec::new();
    for row in u.outer_iter() {
        for e in row.iter() {
            let v = (if *e < 0.0 {
                0.0
            } else if *e > 1.0 {
                1.0
            } else {
                *e
            } * 255.0) as u8;

            texture_data.push(v);
            texture_data.push(v);
            texture_data.push(v);
            texture_data.push(v);
        }
    }
    texture::RawImage2d::from_raw_rgba(texture_data, (256, 256))
}
