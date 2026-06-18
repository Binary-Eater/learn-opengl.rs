extern crate glfw;

use glfw::{Context, Window};

fn framebuffer_size_callback(_window: &mut Window, width: i32, height: i32) {
    // TODO figure out why this is safe and write a SAFETY comment
    // TIL it's because the functions are dynamically loaded via the symbol lookup...
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

fn main() {
    use glfw::fail_on_errors;

    let mut glfw = glfw::init(fail_on_errors!()).unwrap();

    glfw.window_hint(glfw::WindowHint::ContextVersionMajor(3));
    glfw.window_hint(glfw::WindowHint::ContextVersionMinor(3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));

    let (mut window, _events) = glfw
        .create_window(800, 600, "Hello this is window", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    /* Implement symbol lookup closure for GL functions */
    gl::load_with(|s| window.get_proc_address(s) as *const _);

    window.make_current(); /* Equivalent to glfwMakeContextCurrent */
    window.set_framebuffer_size_callback(framebuffer_size_callback);

    // render loop
    /* Equivalent of glfwWindowShouldClose */
    while !window.should_close() {
        // input
        process_input(&mut window);

        // rendering commands here
        // TODO write SAFETY comment
        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        // check and call events and swap the buffers
        window.swap_buffers();
        glfw.poll_events();
    }

    /* There is no glfwTerminate call due to Drop trait */
}
