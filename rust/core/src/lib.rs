use rand::Rng;
use sokol::gfx as sg;
use sokol::gfx::VertexFormat;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

mod flutter;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
mod shader;

static POINTER_ADDRESS: AtomicUsize = AtomicUsize::new(0);

#[unsafe(no_mangle)]
pub extern "C" fn launch_app() {
    #[cfg(target_os = "macos")]
    macos::launch_app();
}

#[derive(Default, Debug)]
struct State {
    bind: sg::Bindings,
    pip: sg::Pipeline,
    swapchain: sg::Swapchain,
    clear_color: sg::Color,
}

fn state_from_pointer<'a>(state_pointer: usize) -> Option<&'a mut State> {
    unsafe {
        let state = state_pointer as *mut State;
        state.as_mut()
    }
}

extern "C" fn init(state_pointer: usize) {
    let Some(state) = state_from_pointer(state_pointer) else {
        return;
    };

    state.bind.vertex_buffers[0] = sg::make_buffer(&sg::BufferDesc {
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
    state.pip = sg::make_pipeline(&sg::PipelineDesc {
        shader: sg::make_shader(&shader::triangle_shader_desc(sg::query_backend())),
        layout: {
            let mut l = sg::VertexLayoutState::new();
            l.attrs[shader::ATTR_TRIANGLE_POSITION].format = VertexFormat::Float3;
            l.attrs[shader::ATTR_TRIANGLE_COLOR0].format = VertexFormat::Float4;
            l
        },
        ..Default::default()
    });
}

extern "C" fn frame(state_pointer: usize) {
    let Some(state) = state_from_pointer(state_pointer) else {
        return;
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

#[unsafe(no_mangle)]
pub extern "C" fn randomize_clear_color(state_pointer: usize) {
    let Some(state) = state_from_pointer(state_pointer) else {
        return;
    };
    let mut rng = rand::rng();
    state.clear_color.r = rng.random_range(0.0..0.2);
    state.clear_color.g = rng.random_range(0.0..0.2);
    state.clear_color.b = rng.random_range(0.0..0.2);
}

#[unsafe(no_mangle)]
pub extern "C" fn state_pointer() -> usize {
    POINTER_ADDRESS.load(Ordering::SeqCst)
}
