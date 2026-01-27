use epoxy::types::GLint;
use gdk::glib::Propagation;
use gtk::Application;
use gtk::ApplicationWindow;
use gtk::prelude::*;
use sokol::gfx::VertexFormat;
use sokol::{app as sapp, gfx as sg, glue as sglue};
use std::cell::RefCell;
use std::ffi;

mod flutter;
mod shader;

#[derive(Default)]
struct State {
    bind: sg::Bindings,
    pip: sg::Pipeline,
    swapchain: sg::Swapchain,
    clear_color: sg::Color,
}

unsafe impl Send for State {}
unsafe impl Sync for State {}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

impl Drop for State {
    fn drop(&mut self) {
        println!("State is dropped");
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn set_up(app: *const *const gtk::Application) {
    {
        #[cfg(target_os = "macos")]
        let library = unsafe { libloading::os::unix::Library::new("libepoxy.0.dylib") }.unwrap();
        #[cfg(all(unix, not(target_os = "macos")))]
        let library = unsafe { libloading::os::unix::Library::new("libepoxy.so.0") }.unwrap();
        #[cfg(windows)]
        let library = libloading::os::windows::Library::open_already_loaded("libepoxy-0.dll")
            .or_else(|_| libloading::os::windows::Library::open_already_loaded("epoxy-0.dll"))
            .unwrap();

        epoxy::load_with(|name| {
            unsafe { library.get::<_>(name.as_bytes()) }
                .map(|symbol| *symbol)
                .unwrap_or(std::ptr::null())
        });
    }
    println!("Connect");
    let app = unsafe { gtk::Application::from_glib_ptr_borrow(app as *const *const _) };

    create_window(app);
    // let window = gtk::Window::new(gtk::WindowType::Toplevel);
    // GtkWidget *window;

    // window = gtk_application_window_new (app);
    // gtk_window_set_title (GTK_WINDOW (window), "Window");
    // gtk_window_set_default_size (GTK_WINDOW (window), 800, 600);
    // gtk_widget_show_all (window);
}

extern "C" fn init() {
    // create vertex buffer with triangle vertices
    STATE.with(|state| {
        state.borrow_mut().bind.vertex_buffers[0] = sg::make_buffer(&sg::BufferDesc {
            #[rustfmt::skip]
        data: sg::value_as_range::<[f32; _]>(&[
             // positions    colors
             0.0,  0.5, 0.5, 1.0, 0.0, 0.0, 1.0,
             0.5, -0.5, 0.5, 0.0, 1.0, 0.0, 1.0,
            -0.5, -0.5, 0.5, 0.0, 0.0, 1.0, 1.0,
        ]),
            ..Default::default()
        });

        // create a shader and pipeline object
        state.borrow_mut().pip = sg::make_pipeline(&sg::PipelineDesc {
            shader: sg::make_shader(&shader::triangle_shader_desc(sg::query_backend())),
            layout: {
                let mut l = sg::VertexLayoutState::new();
                l.attrs[shader::ATTR_TRIANGLE_POSITION].format = VertexFormat::Float3;
                l.attrs[shader::ATTR_TRIANGLE_COLOR0].format = VertexFormat::Float4;
                l
            },
            ..Default::default()
        });
    });
}

extern "C" fn frame(area: &gtk::GLArea) {
    // let state = unsafe { &mut *(user_data as *mut State) };

    let mut framebuffer_id: GLint = 0;
    unsafe {
        epoxy::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut framebuffer_id);
    }

    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.swapchain.width = area.allocated_width();
        state.swapchain.height = area.allocated_height();
        state.swapchain.gl = sg::GlSwapchain {
            framebuffer: framebuffer_id as u32,
        };

        let mut pass_action = sg::PassAction::new();
        pass_action.colors[0].load_action = sg::LoadAction::Clear;
        pass_action.colors[0].clear_value = state.clear_color;

        sg::begin_pass(&sg::Pass {
            action: pass_action,
            swapchain: state.swapchain,
            ..Default::default()
        });
        sg::apply_pipeline(state.pip);
        sg::apply_bindings(&state.bind);
        sg::draw(0, 3, 1);
        sg::end_pass();
        sg::commit();
    });
}

fn create_window(app: &Application) {
    let window = ApplicationWindow::new(app);
    window.set_default_size(1000, 600);

    gtk::init().unwrap();
    let gl_area = gtk::GLArea::new();
    gl_area.set_vexpand(true);
    gl_area.set_hexpand(true);
    gl_area.set_auto_render(true);

    gl_area.connect_realize(|area| {
        area.make_current();

        sg::setup(&sg::Desc {
            environment: sg::Environment {
                defaults: sg::EnvironmentDefaults {
                    color_format: sg::PixelFormat::Rgba8,
                    depth_format: sg::PixelFormat::None,
                    ..Default::default()
                },
                ..Default::default()
            },
            logger: sg::Logger {
                func: Some(sokol::log::slog_func),
                ..Default::default()
            },
            ..Default::default()
        });
        assert!(sg::isvalid());
        init();
    });

    gl_area.connect_render(move |area, _context| {
        if !area.is_realized() {
            return Propagation::Stop;
        }
        frame(area);
        Propagation::Proceed
    });

    let button = gtk::Button::with_label("Click me!");
    button.set_halign(gtk::Align::Start);
    button.set_valign(gtk::Align::Start);
    button.set_margin_start(8);
    button.set_margin_top(8);
    // button.connect_clicked(move |_| {
    //     randomize_clear_color();
    // });

    let flutter_view = flutter::create_flutter_view() as *const gtk::Widget;
    let flutter_view = &flutter_view as *const *const gtk::Widget;
    let flutter_view = unsafe { gtk::Widget::from_glib_ptr_borrow(flutter_view as _) };

    let overlay = gtk::Overlay::new();
    overlay.add_overlay(&gl_area);
    overlay.add_overlay(flutter_view);
    overlay.add_overlay(&button);

    window.add(&overlay);

    window.show_all();
}

extern "C" fn cleanup(user_data: *mut ffi::c_void) {
    sg::shutdown();

    let _ = unsafe { Box::from_raw(user_data as *mut State) };
}

#[unsafe(no_mangle)]
pub extern "C" fn launch_app() -> u8 {
    let app = Application::builder()
        .application_id("co.rizer.test")
        .build();

    app.connect_activate(|app| {
        // We create the main window.
        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(320)
            .default_height(200)
            .title("Hello, World!")
            .build();

        // Show the window.
        window.present();
    });

    println!("Boomy");
    2
    // app.run().get()

    // let state = Box::new(State {
    //     pass_action: sg::PassAction::new(),
    // });

    // let user_data = Box::into_raw(state) as *mut ffi::c_void;

    // flutter_link::add_flutter_view();

    // sapp::run(&sapp::Desc {
    //     init_userdata_cb: Some(init),
    //     frame_userdata_cb: Some(frame),
    //     cleanup_userdata_cb: Some(cleanup),
    //     user_data,
    //     window_title: c"clear.rs".as_ptr(),
    //     width: 800,
    //     height: 600,
    //     sample_count: 4,
    //     logger: sapp::Logger {
    //         func: Some(sokol::log::slog_func),
    //         ..Default::default()
    //     },
    //     icon: sapp::IconDesc {
    //         sokol_default: true,
    //         ..Default::default()
    //     },
    //     ..Default::default()
    // });
}

#[unsafe(no_mangle)]
pub extern "C" fn link_flutter() {
    println!("link_flutter called");
    // flutter_link::add_flutter_view();
}
