use epoxy::types::GLint;
use gdk::glib::Propagation;
use gtk::Application;
use gtk::ApplicationWindow;
use gtk::prelude::*;

#[unsafe(no_mangle)]
pub extern "C" fn set_up(app: *const *const gtk::Application) {
    {
        let library = unsafe { libloading::os::unix::Library::new("libepoxy.so.0") }.unwrap();
        epoxy::load_with(|name| {
            unsafe { library.get::<_>(name.as_bytes()) }
                .map(|symbol| *symbol)
                .unwrap_or(std::ptr::null())
        });
    }
    let app = unsafe { gtk::Application::from_glib_ptr_borrow(app as *const *const _) };

    create_window(app);
}

extern "C" fn frame(area: &gtk::GLArea, state_pointer: usize) {
    let mut framebuffer_id: GLint = 0;
    unsafe {
        epoxy::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut framebuffer_id);
    }

    let Some(state) = state_from_pointer(state_pointer) else {
        return;
    };
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
}

fn create_window(app: &Application) {
    let window = ApplicationWindow::new(app);
    window.set_default_size(1000, 600);

    let state = State::default();
    let state = Box::new(state);
    let state_pointer = &*state as *const State as usize;
    POINTER_ADDRESS.store(state_pointer, Ordering::SeqCst);
    // Store the state in the app data so it doesn’t get dropped.
    unsafe { app.set_data("state", state) };

    gtk::init().unwrap();
    let gl_area = gtk::GLArea::new();
    gl_area.set_vexpand(true);
    gl_area.set_hexpand(true);
    gl_area.set_auto_render(true);

    gl_area.connect_realize(move |area| {
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
        init(state_pointer);
    });

    gl_area.connect_render(move |area, _| {
        if !area.is_realized() {
            return Propagation::Stop;
        }
        frame(area, state_pointer);
        Propagation::Proceed
    });

    let flutter_view = flutter::create_flutter_view() as *const gtk::Widget;
    let flutter_view = &flutter_view as *const *const gtk::Widget;
    let flutter_view = unsafe { gtk::Widget::from_glib_ptr_borrow(flutter_view as _) };

    let overlay = gtk::Overlay::new();
    overlay.add_overlay(&gl_area);
    overlay.add_overlay(flutter_view);

    window.add(&overlay);

    window.show_all();
}

extern "C" fn frame(area: &gtk::GLArea, state_pointer: usize) {
    let mut framebuffer_id: GLint = 0;
    unsafe {
        epoxy::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut framebuffer_id);
    }

    let Some(state) = super::state_from_pointer(state_pointer) else {
        return;
    };
    state.swapchain.width = area.allocated_width();
    state.swapchain.height = area.allocated_height();
    state.swapchain.gl = sg::GlSwapchain {
        framebuffer: framebuffer_id as u32,
    };

    super::frame(state_pointer);
}
