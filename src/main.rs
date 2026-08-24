extern crate glfw;

use std::ffi::CString;
use std::ptr;

use glfw::{Context, Window};

fn framebuffer_size_callback(_window: &mut Window, width: i32, height: i32) {
    // SAFETY: Assume the functions are dynamically loaded via the symbol lookup.
    unsafe {
        gl::Viewport(0, 0, width, height);
    }
}

fn process_input(window: &mut Window) {
    match window.get_key(glfw::Key::Escape) {
        glfw::Action::Press => window.set_should_close(true),
        _ => (),
    }
}

fn main() -> Result<(), String> {
    use glfw::fail_on_errors;

    let mut glfw = glfw::init(fail_on_errors!()).unwrap();

    glfw.window_hint(glfw::WindowHint::ContextVersionMajor(3));
    glfw.window_hint(glfw::WindowHint::ContextVersionMinor(3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));

    let (mut window, _events) = glfw
        .create_window(800, 600, "LearnOpenGL.rs", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    /* Implement symbol lookup closure for GL functions */
    gl::load_with(|s| window.get_proc_address(s) as *const _);

    window.make_current(); /* Equivalent to glfwMakeContextCurrent */
    window.set_framebuffer_size_callback(framebuffer_size_callback);

    /*
     * A world without element buffer objects for drawing a rectangle.
     *
     * let vertices: [f32; _] = [
     *     /*     x,        y,       z, */
     *     // first triangle
     *      0.5_f32,  0.5_f32, 0.0_f32,  // top right
     *      0.5_f32, -0.5_f32, 0.0_f32,  // bottom right
     *     -0.5_f32,  0.5_f32, 0.0_f32,  // top left
     *     // second triangle
     *      0.5_f32, -0.5_f32, 0.0_f32,  // bottom right
     *     -0.5_f32, -0.5_f32, 0.0_f32,  // bottom left
     *     -0.5_f32,  0.5_f32, 0.0_f32,  // top left
     * ];
     */

    let vertices: [f32; _] = [
        /*     x,        y,       z, */
        // first triangle
         0.5_f32,  0.5_f32, 0.0_f32,  // top right
         0.5_f32, -0.5_f32, 0.0_f32,  // bottom right
        -0.5_f32, -0.5_f32, 0.0_f32,  // bottom left
        -0.5_f32,  0.5_f32, 0.0_f32,  // top left
    ];
    let indices: [u32; _] = [
        /* NOTE: indices start from 0 and end at 3 */
        0, 1, 3,   // first triangle
        1, 2, 3,   // second triangle
    ];

    let mut vbo: u32 = 0;
    let mut vao: u32 = 0;
    let mut ebo: u32 = 0;
    // SAFETY: Assume the functions are dynamically loaded via the symbol lookup.
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::GenBuffers(1, &mut ebo);

        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            std::mem::size_of_val(&vertices)
                .try_into()
                .expect("failed to represent usize value as an i32"),
            /*
             * TODO: Might be interesting to debug incorrect size in RenderDoc.
             *
             * vertices
             *     .len()
             *     .try_into()
             *     .expect("failed to represent usize value as an i32"),
            */
            vertices.as_ptr().cast(), /* cast *const f32 into *const c_void */
            gl::STATIC_DRAW,
        );

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            std::mem::size_of_val(&indices)
                .try_into()
                .expect("failed to represent usize value as an i32"),
            indices.as_ptr().cast(), /* cast *const f32 into *const c_void */
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            (3 * size_of::<f32>())
                .try_into()
                .expect("failed to represent usize value as an i32"),
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);
    }

    let vertex_shader_source = CString::new(include_str!("../assets/shader.vert"))
        .expect("string slice contains an illegal null byte internally");
    let vertex_shader: u32;

    let mut success: i32 = 0;
    let mut info_log: [i8; 512] = [0; 512];
    let info_log_isize: i32 =
        info_log.len().try_into().expect("failed to represent usize value as an i32");
    // SAFETY: Assume the functions are dynamically loaded via the symbol lookup.
    unsafe {
        vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        gl::ShaderSource(
            vertex_shader,
            1,
            &vertex_shader_source.as_ptr(),
            ptr::null(),
        );
        gl::CompileShader(vertex_shader);

        gl::GetShaderiv(
            vertex_shader,
            gl::COMPILE_STATUS,
            ptr::from_mut(&mut success),
        );
        if success == 0 {
            gl::GetShaderInfoLog(
                vertex_shader,
                info_log_isize,
                ptr::null_mut(),
                info_log.as_mut_ptr(),
            );
        }
    }

    if success == 0 {
        // SAFETY: The i8 values in info_log can safely be represented as u8
        // since the byte values represent characters.
        let char_slice: &[u8; 512] = unsafe { std::mem::transmute(&info_log) };

        let error_string = format!(
            "ERROR::SHADER::VERTEX::COMPILATION_FAILED: {}",
            match std::str::from_utf8(char_slice) {
                Ok(s) => String::from(s),
                Err(e) => format!("Invalid UTF-8 found in error message: {}", e),
            },
        ).trim_end_matches("\0").trim_end().to_string();

        return Err(error_string);
    }

    let fragment_shader_source = CString::new(include_str!("../assets/shader.frag"))
        .expect("string slice contains an illegal null byte internally");
    let fragment_shader: u32;

    success = 0;
    // SAFETY: Assume the functions are dynamically loaded via the symbol lookup.
    unsafe {
        fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        gl::ShaderSource(
            fragment_shader,
            1,
            &fragment_shader_source.as_ptr(),
            ptr::null(),
        );
        gl::CompileShader(fragment_shader);

        gl::GetShaderiv(
            fragment_shader,
            gl::COMPILE_STATUS,
            ptr::from_mut(&mut success),
        );
        if success == 0 {
            gl::DeleteShader(vertex_shader);

            gl::GetShaderInfoLog(
                fragment_shader,
                info_log_isize,
                ptr::null_mut(),
                info_log.as_mut_ptr(),
            );
        }
    }

    if success == 0 {
        // SAFETY: The i8 values in info_log can safely be represented as u8
        // since the byte values represent characters.
        let char_slice: &[u8; 512] = unsafe { std::mem::transmute(&info_log) };

        let error_string = format!(
            "ERROR::SHADER::FRAGMENT::COMPILATION_FAILED: {}",
            match std::str::from_utf8(char_slice) {
                Ok(s) => String::from(s),
                Err(e) => format!("Invalid UTF-8 found in error message: {}", e),
            },
        ).trim_end_matches("\0").trim_end().to_string();

        return Err(error_string);
    }

    let shader_program: u32;

    success = 0;
    // SAFETY: Assume the functions are dynamically loaded via the symbol lookup.
    unsafe {
        shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);

        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        gl::GetProgramiv(
            shader_program,
            gl::LINK_STATUS,
            ptr::from_mut(&mut success),
        );
        if success == 0 {
            gl::GetProgramInfoLog(
                shader_program,
                info_log_isize,
                ptr::null_mut(),
                info_log.as_mut_ptr(),
            );
        }
    }

    if success == 0 {
        // SAFETY: The i8 values in info_log can safely be represented as u8
        // since the byte values represent characters.
        let char_slice: &[u8; 512] = unsafe { std::mem::transmute(&info_log) };

        let error_string = format!(
            "ERROR::SHADER::PROGRAM::LINKAGE_FAILED: {}",
            match std::str::from_utf8(char_slice) {
                Ok(s) => String::from(s),
                Err(e) => format!("Invalid UTF-8 found in error message: {}", e),
            },
        ).trim_end_matches("\0").trim_end().to_string();

        return Err(error_string);
    }

    // render loop
    /* Equivalent of glfwWindowShouldClose */
    while !window.should_close() {
        // input
        process_input(&mut window);

        // SAFETY: Assume the functions are dynamically loaded via the symbol lookup.
        unsafe {
            gl::ClearColor(0.2_f32, 0.3_f32, 0.3_f32, 1.0_f32);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::UseProgram(shader_program);
            gl::BindVertexArray(vao);
            /*
             * Can use polygon mode to help with debugging using wireframes
             *
             * wireframe mode: gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
             * default:        gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
             */
            /*
             * ebo is bounded to vao since the vao tracks the last EBO object
             * bound while vao is bound.
             */
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
            gl::BindVertexArray(0);
        }

        // check and call events and swap the buffers
        window.swap_buffers();
        glfw.poll_events();
    }

    // SAFETY: Assume the functions are dynamically loaded via the symbol lookup.
    unsafe {
        gl::DeleteVertexArrays(1, &vao);
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteBuffers(1, &ebo);
        gl::DeleteProgram(shader_program);
    }

    /* There is no glfwTerminate call due to Drop trait */
    Ok(())
}
