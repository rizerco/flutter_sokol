#include "my_application.h"

#include <flutter_linux/flutter_linux.h>
#include <stdio.h>
#ifdef GDK_WINDOWING_X11
#include <gdk/gdkx.h>
#endif

#include "flutter/generated_plugin_registrant.h"

struct _MyApplication {
  GtkApplication parent_instance;
  char** dart_entrypoint_arguments;
};

// static struct {
//     sg_pipeline pip;
//     sg_bindings bind;
//     sg_pass_action pass_action;
// } state;

G_DEFINE_TYPE(MyApplication, my_application, GTK_TYPE_APPLICATION)

// Called when first Flutter frame received.
static void first_frame_cb(MyApplication* self, FlView* view) {
  gtk_widget_show(gtk_widget_get_toplevel(GTK_WIDGET(view)));
}

static void do_you_connect(void) {
    // void *handle = dlopen("libsokol_layer.so", RTLD_LAZY);
    // if (!handle) {
    //     fputs (dlerror(), stderr);
    //     return;
    // }

    // int (*launch_app)(void) = dlsym(handle, "launch_app");
    // char *error;
    // if ((error = dlerror()) != NULL)  {
    //     fputs(error, stderr);
    //     return;
    // }

    // launch_app();

    // dlclose(handle);
}

// static void init(void) {
//     sg_desc desc = {};
//     sg_setup(&desc);


//     float vertices[] = {
//          0.0f,  0.5f, 0.5f,     1.0f, 0.0f, 0.0f, 1.0f,
//          0.5f, -0.5f, 0.5f,     0.0f, 1.0f, 0.0f, 1.0f,
//         -0.5f, -0.5f, 0.5f,     0.0f, 0.0f, 1.0f, 1.0f
//     };
//     sg_buffer_desc buffer_desc = {
//         .data = SG_RANGE(vertices),
//     };
//     state.bind.vertex_buffers[0] = sg_make_buffer(&buffer_desc);

//     // state.pip = sg_make_pipeline(&(sg_pipeline_desc){
//     //     .shader = sg_make_shader(triangle_shader_desc(sg_query_backend())),
//     //     .layout = {
//     //         .attrs = {
//     //             [ATTR_triangle_position].format = SG_VERTEXFORMAT_FLOAT3,
//     //             [ATTR_triangle_color0].format = SG_VERTEXFORMAT_FLOAT4
//     //         }
//     //     },
//     // });

//     // state.pass_action = (sg_pass_action) {
//     //     .colors[0] = { .load_action=SG_LOADACTION_CLEAR, .clear_value={0.0f, 0.0f, 0.0f, 1.0f } }
//     // };
// }

 // static gboolean
 //  render (GtkGLArea *area, GdkGLContext *context)
 //  {
 //    // inside this function it's safe to use GL; the given
 //    // `GdkGLContext` has been made current to the drawable
 //    // surface used by the `GtkGLArea` and the viewport has
 //    // already been set to be the size of the allocation

 //    // we can start by clearing the buffer
 //    glClearColor (0, 0, 0.3, 1.0);
 //    glClear (GL_COLOR_BUFFER_BIT);

 //    // draw your object
 //    // draw_an_object ();
 //    // sg_pass pass = {.action = state.pass_action };
 //    // sg_begin_pass(&pass);
 //    // sg_apply_pipeline(state.pip);
 //    // sg_apply_bindings(&state.bind);
 //    // sg_draw(0, 3, 1);
 //    // sg_end_pass();
 //    // sg_commit();

 //    // we completed our drawing; the draw commands will be
 //    // flushed at the end of the signal emission chain, and
 //    // the buffers will be drawn on the window
 //    return TRUE;
 //  }

// Implements GApplication::activate.
static void my_application_activate(GApplication* application) {
    
  MyApplication* self = MY_APPLICATION(application);
  GtkWindow* window =
      GTK_WINDOW(gtk_application_window_new(GTK_APPLICATION(application)));

  // Use a header bar when running in GNOME as this is the common style used
  // by applications and is the setup most users will be using (e.g. Ubuntu
  // desktop).
  // If running on X and not using GNOME then just use a traditional title bar
  // in case the window manager does more exotic layout, e.g. tiling.
  // If running on Wayland assume the header bar will work (may need changing
  // if future cases occur).
  gboolean use_header_bar = TRUE;
#ifdef GDK_WINDOWING_X11
  GdkScreen* screen = gtk_window_get_screen(window);
  if (GDK_IS_X11_SCREEN(screen)) {
    const gchar* wm_name = gdk_x11_screen_get_window_manager_name(screen);
    if (g_strcmp0(wm_name, "GNOME Shell") != 0) {
      use_header_bar = FALSE;
    }
  }
#endif
  if (use_header_bar) {
    GtkHeaderBar* header_bar = GTK_HEADER_BAR(gtk_header_bar_new());
    gtk_widget_show(GTK_WIDGET(header_bar));
    gtk_header_bar_set_title(header_bar, "flutter_sokol");
    gtk_header_bar_set_show_close_button(header_bar, TRUE);
    gtk_window_set_titlebar(window, GTK_WIDGET(header_bar));
  } else {
    gtk_window_set_title(window, "flutter_sokol");
  }

  gtk_window_set_default_size(window, 1280, 720);

  // init();

  g_autoptr(FlDartProject) project = fl_dart_project_new();
  fl_dart_project_set_dart_entrypoint_arguments(
      project, self->dart_entrypoint_arguments);

  FlView* view = fl_view_new(project);
  GdkRGBA background_color;
  // Background defaults to black, override it here if necessary, e.g. #00000000
  // for transparent.
  gdk_rgba_parse(&background_color, "#00000000");
  fl_view_set_background_color(view, &background_color);
  gtk_widget_show(GTK_WIDGET(view));
  // gtk_container_add(GTK_CONTAINER(window), GTK_WIDGET(view));

  // Show the window when Flutter renders.
  // Requires the view to be realized so we can start rendering.
  g_signal_connect_swapped(view, "first-frame", G_CALLBACK(first_frame_cb),
                           self);
  gtk_widget_realize(GTK_WIDGET(view));

  fl_register_plugins(FL_PLUGIN_REGISTRY(view));

  // GtkWidget* button = gtk_button_new_with_label ("Hello World");
  // gtk_widget_set_halign(button, GTK_ALIGN_CENTER);
  // gtk_widget_set_valign(button, GTK_ALIGN_CENTER);
  // // g_signal_connect (button, "clicked", G_CALLBACK (print_hello), NULL);
  // gtk_container_add(GTK_CONTAINER(window), GTK_WIDGET(button));

  // Create drawing area for Sokol rendering
  // GtkWidget *drawing_area = gtk_drawing_area_new();
  // // gtk_container_add(GTK_CONTAINER(window), drawing_area);
  // gtk_widget_show(GTK_WIDGET(drawing_area));
  // g_signal_connect (G_OBJECT (drawing_area), "draw",
  //                   G_CALLBACK (draw_callback), NULL);

  GtkWidget *gl_area = gtk_gl_area_new();
  gtk_widget_show(GTK_WIDGET(gl_area));

  do_you_connect();
  // connect to the "render" signal
  // g_signal_connect (gl_area, "render", G_CALLBACK (render), NULL);

  // g_signal_connect(drawing_area, "draw", G_CALLBACK(draw_callback), self);

  // GtkWidget *button = gtk_button_new_with_label ("Hello World");
  GtkWidget *empty_image = gtk_image_new();
  gtk_widget_show(GTK_WIDGET(empty_image));

  GtkWidget* overlay = gtk_overlay_new();
  // gtk_overlay_add_overlay(GTK_OVERLAY(overlay), GTK_WIDGET(empty_image));
  gtk_overlay_add_overlay(GTK_OVERLAY(overlay), GTK_WIDGET(gl_area));
  gtk_overlay_add_overlay(GTK_OVERLAY(overlay), GTK_WIDGET(view));
  // gtk_overlay_set_overlay_pass_through(GTK_OVERLAY(overlay), GTK_WIDGET(drawing_area), true);

  gtk_widget_show(GTK_WIDGET(overlay));
  gtk_container_add(GTK_CONTAINER(window), GTK_WIDGET(overlay));


  gtk_widget_grab_focus(GTK_WIDGET(view));

  gtk_window_present(window);

}

// Implements GApplication::local_command_line.
static gboolean my_application_local_command_line(GApplication* application,
                                                  gchar*** arguments,
                                                  int* exit_status) {
  MyApplication* self = MY_APPLICATION(application);
  // Strip out the first argument as it is the binary name.
  self->dart_entrypoint_arguments = g_strdupv(*arguments + 1);

  g_autoptr(GError) error = NULL;
  if (!g_application_register(application, NULL, &error)) {
    g_warning("Failed to register: %s", error->message);
    *exit_status = 1;
    return TRUE;
  }

  g_application_activate(application);
  *exit_status = 0;

  return TRUE;
}

// Implements GApplication::startup.
static void my_application_startup(GApplication* application) {
  // MyApplication* self = MY_APPLICATION(object);

  // Perform any actions required at application startup.

  G_APPLICATION_CLASS(my_application_parent_class)->startup(application);
}

// Implements GApplication::shutdown.
static void my_application_shutdown(GApplication* application) {
  // MyApplication* self = MY_APPLICATION(object);

  // Perform any actions required at application shutdown.

  G_APPLICATION_CLASS(my_application_parent_class)->shutdown(application);
}

// Implements GObject::dispose.
static void my_application_dispose(GObject* object) {
  MyApplication* self = MY_APPLICATION(object);
  g_clear_pointer(&self->dart_entrypoint_arguments, g_strfreev);
  G_OBJECT_CLASS(my_application_parent_class)->dispose(object);
}

static void my_application_class_init(MyApplicationClass* klass) {
  G_APPLICATION_CLASS(klass)->activate = my_application_activate;
  G_APPLICATION_CLASS(klass)->local_command_line =
      my_application_local_command_line;
  G_APPLICATION_CLASS(klass)->startup = my_application_startup;
  G_APPLICATION_CLASS(klass)->shutdown = my_application_shutdown;
  G_OBJECT_CLASS(klass)->dispose = my_application_dispose;
}

static void my_application_init(MyApplication* self) {}

MyApplication* my_application_new() {
  // Set the program name to the application ID, which helps various systems
  // like GTK and desktop environments map this running application to its
  // corresponding .desktop file. This ensures better integration by allowing
  // the application to be recognized beyond its binary name.
  g_set_prgname(APPLICATION_ID);

  return MY_APPLICATION(g_object_new(my_application_get_type(),
                                     "application-id", APPLICATION_ID, "flags",
                                     G_APPLICATION_NON_UNIQUE, NULL));
}
