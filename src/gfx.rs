//! Puts canvases on screen: one transparent OpenGL surface per window, the canvas uploaded as a texture.
//! OpenGL is only a transport for per-pixel alpha (software blitters like softbuffer can't do transparency).

use claude_companion::canvas::Canvas;
use glow::HasContext;
use glutin::config::{Config, ConfigTemplateBuilder, GlConfig};
use glutin::context::{
    ContextApi, ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext, PossiblyCurrentGlContext, Version,
};
use glutin::display::{GetGlDisplay, GlDisplay};
use glutin::surface::{GlSurface, Surface, SwapInterval, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasWindowHandle;
use std::error::Error;
use std::num::NonZeroU32;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};

pub struct Gfx {
    config: Config,
    context: PossiblyCurrentContext,
    gl: glow::Context,
    program: glow::Program,
    quad: glow::Buffer,
}

pub struct GlWin {
    pub window: Window,
    surface: Surface<WindowSurface>,
    texture: glow::Texture,
    premultiplied: Vec<u8>,
}

const VERTEX: &str = "
attribute vec2 pos;
varying vec2 uv;
void main() {
    uv = vec2(pos.x * 0.5 + 0.5, 0.5 - pos.y * 0.5);
    gl_Position = vec4(pos, 0.0, 1.0);
}";

const FRAGMENT: &str = "
varying vec2 uv;
uniform sampler2D tex;
void main() { gl_FragColor = texture2D(tex, uv); }";

impl Gfx {
    /// Creates the GL display and context together with the first window.
    pub fn new(event_loop: &ActiveEventLoop, attrs: WindowAttributes) -> Result<(Gfx, GlWin), Box<dyn Error>> {
        let template = ConfigTemplateBuilder::new().with_alpha_size(8).with_transparency(true);
        let (window, config) =
            DisplayBuilder::new().with_window_attributes(Some(attrs)).build(event_loop, template, |configs| {
                // prefer transparency, then the fewest samples
                configs
                    .max_by_key(|c| (c.supports_transparency().unwrap_or(false), std::cmp::Reverse(c.num_samples())))
                    .expect("no GL config")
            })?;
        let window = window.ok_or("window was not created")?;
        if !config.supports_transparency().unwrap_or(false) {
            eprintln!("claude-companion: no transparent GL visual (is a compositor running?)");
        }

        let raw = window.window_handle()?.as_raw();
        let display = config.display();
        let candidates = [
            ContextAttributesBuilder::new().build(Some(raw)),
            ContextAttributesBuilder::new().with_context_api(ContextApi::OpenGl(Some(Version::new(2, 1)))).build(Some(raw)),
            ContextAttributesBuilder::new().with_context_api(ContextApi::Gles(Some(Version::new(2, 0)))).build(Some(raw)),
        ];
        let not_current = candidates
            .iter()
            .find_map(|a| unsafe { display.create_context(&config, a).ok() })
            .ok_or("could not create an OpenGL context")?;
        let surface = create_surface(&config, &window)?;
        let context = not_current.make_current(&surface)?;
        let _ = surface.set_swap_interval(&context, SwapInterval::DontWait);

        let gl = unsafe { glow::Context::from_loader_function_cstr(|s| display.get_proc_address(s)) };
        let (program, quad) = unsafe { setup(&gl)? };
        let gfx = Gfx { config, context, gl, program, quad };
        let first = gfx.wrap(window, surface)?;
        Ok((gfx, first))
    }

    pub fn add_window(&self, event_loop: &ActiveEventLoop, attrs: WindowAttributes) -> Result<GlWin, Box<dyn Error>> {
        let window = glutin_winit::finalize_window(event_loop, attrs, &self.config)?;
        let surface = create_surface(&self.config, &window)?;
        self.wrap(window, surface)
    }

    fn wrap(&self, window: Window, surface: Surface<WindowSurface>) -> Result<GlWin, Box<dyn Error>> {
        let texture = unsafe {
            let t = self.gl.create_texture()?;
            self.gl.bind_texture(glow::TEXTURE_2D, Some(t));
            for (k, v) in [
                (glow::TEXTURE_MIN_FILTER, glow::NEAREST),
                (glow::TEXTURE_MAG_FILTER, glow::NEAREST),
                (glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE),
                (glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE),
            ] {
                self.gl.tex_parameter_i32(glow::TEXTURE_2D, k, v as i32);
            }
            t
        };
        Ok(GlWin { window, surface, texture, premultiplied: Vec::new() })
    }

    pub fn present(&self, w: &mut GlWin, canvas: &Canvas) {
        let size = w.window.inner_size();
        let (Some(sw), Some(sh)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) else { return };
        if self.context.make_current(&w.surface).is_err() {
            return;
        }
        w.surface.resize(&self.context, sw, sh);

        // compositors expect premultiplied alpha
        w.premultiplied.clear();
        w.premultiplied.extend(canvas.data.chunks_exact(4).flat_map(|p| {
            let a = p[3] as u32;
            [(p[0] as u32 * a / 255) as u8, (p[1] as u32 * a / 255) as u8, (p[2] as u32 * a / 255) as u8, p[3]]
        }));

        let gl = &self.gl;
        unsafe {
            gl.viewport(0, 0, sw.get() as i32, sh.get() as i32);
            gl.clear_color(0.0, 0.0, 0.0, 0.0);
            gl.clear(glow::COLOR_BUFFER_BIT);
            gl.use_program(Some(self.program));
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(w.texture));
            gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                canvas.width as i32,
                canvas.height as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(&w.premultiplied)),
            );
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.quad));
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 0, 0);
            gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
        }
        let _ = w.surface.swap_buffers(&self.context);
    }
}

fn create_surface(config: &Config, window: &Window) -> Result<Surface<WindowSurface>, Box<dyn Error>> {
    let attrs = window.build_surface_attributes(Default::default())?;
    Ok(unsafe { config.display().create_window_surface(config, &attrs)? })
}

unsafe fn setup(gl: &glow::Context) -> Result<(glow::Program, glow::Buffer), Box<dyn Error>> {
    let version = gl.version();
    let header = if version.is_embedded { "#version 100\nprecision mediump float;\n" } else { "#version 120\n" };
    let program = gl.create_program()?;
    let mut shaders = Vec::new();
    for (kind, src) in [(glow::VERTEX_SHADER, VERTEX), (glow::FRAGMENT_SHADER, FRAGMENT)] {
        let s = gl.create_shader(kind)?;
        gl.shader_source(s, &format!("{header}{src}"));
        gl.compile_shader(s);
        if !gl.get_shader_compile_status(s) {
            return Err(gl.get_shader_info_log(s).into());
        }
        gl.attach_shader(program, s);
        shaders.push(s);
    }
    gl.bind_attrib_location(program, 0, "pos");
    gl.link_program(program);
    if !gl.get_program_link_status(program) {
        return Err(gl.get_program_info_log(program).into());
    }
    for s in shaders {
        gl.detach_shader(program, s);
        gl.delete_shader(s);
    }
    gl.use_program(Some(program));
    if let Some(loc) = gl.get_uniform_location(program, "tex") {
        gl.uniform_1_i32(Some(&loc), 0);
    }

    // core profiles need a bound vertex array object; legacy and ES 2 contexts may not have one
    if !version.is_embedded && version.major >= 3 {
        if let Ok(vao) = gl.create_vertex_array() {
            gl.bind_vertex_array(Some(vao));
        }
    }
    let quad = gl.create_buffer()?;
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(quad));
    let verts: [f32; 8] = [-1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0, 1.0];
    let bytes = std::slice::from_raw_parts(verts.as_ptr() as *const u8, std::mem::size_of_val(&verts));
    gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytes, glow::STATIC_DRAW);
    Ok((program, quad))
}
