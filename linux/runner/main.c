// #include "my_application.h"

#include <stdlib.h>
#include <stdio.h>
#include <dlfcn.h>
#include <gtk/gtk.h>
#include <flutter_linux/flutter_linux.h>

// int main(int argc, char** argv) {
//   printf("hello chum");
//   return 0;
//   // g_autoptr(MyApplication) app = my_application_new();
//   // return g_application_run(G_APPLICATION(app), argc, argv);
// }

void set_up_app(GtkApplication* app) {
    void *handle = dlopen("libsokol_layer.so", RTLD_LAZY);
    if (!handle) {
        fputs (dlerror(), stderr);
        exit(1);
    }

    int (*set_up)(GtkApplication**) = dlsym(handle, "set_up");
    char *error;
    if ((error = dlerror()) != NULL)  {
        fputs(error, stderr);
        exit(1);
    }

    set_up(&app);
}


static void
activate (GtkApplication* app,
          gpointer        user_data)
{
  // GtkWidget *window;

  // window = gtk_application_window_new (app);
  // gtk_window_set_title (GTK_WINDOW (window), "Window");
  // gtk_window_set_default_size (GTK_WINDOW (window), 800, 600);
  // gtk_widget_show_all (window);

  set_up_app(app);
}

int
main (int    argc,
      char **argv)
{
  GtkApplication *app;
  int status;

  app = gtk_application_new ("org.gtk.example", G_APPLICATION_DEFAULT_FLAGS);
  g_signal_connect (app, "activate", G_CALLBACK (activate), NULL);
  status = g_application_run (G_APPLICATION (app), argc, argv);
  g_object_unref (app);

  return status;
}

// #include <stdlib.h>
// #include <stdio.h>
// #include <dlfcn.h>

// #include <gtk/gtk.h>

// static void
// print_hello (GtkWidget *widget,
//              gpointer   data)
// {
//   g_print ("Hello World\n");
// }

// static void
// activate (GtkApplication *app,
//           gpointer        user_data)
// {
//   GtkWidget *window;
//   // GtkWidget *button;

//   window = gtk_application_window_new (app);
//   gtk_window_set_title (GTK_WINDOW (window), "Hello");
//   gtk_window_set_default_size (GTK_WINDOW (window), 200, 200);

//   // button = gtk_button_new_with_label ("Hello World");
//   // gtk_widget_set_halign(button, GTK_ALIGN_CENTER);
//   // gtk_widget_set_valign(button, GTK_ALIGN_CENTER);
//   // g_signal_connect (button, "clicked", G_CALLBACK (print_hello), NULL);
//   // gtk_window_set_child (GTK_WINDOW (window), button);

//   gtk_window_present (GTK_WINDOW (window));
// }

// int
// main (int    argc,
//       char **argv)
// {
//   GtkApplication *app;
//   int status;

//   app = gtk_application_new ("org.gtk.example", G_APPLICATION_DEFAULT_FLAGS);
//   g_signal_connect (app, "activate", G_CALLBACK (activate), NULL);
//   status = g_application_run (G_APPLICATION (app), argc, argv);
//   g_object_unref (app);

//   return status;
// }

