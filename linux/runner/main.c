#include <stdlib.h>
#include <stdio.h>
#include <dlfcn.h>
#include <gtk/gtk.h>
#include <flutter_linux/flutter_linux.h>


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
  set_up_app(app);
}

int
main (int    argc,
      char **argv)
{
  GtkApplication *app;
  int status;

  app = gtk_application_new ("co.rizer.flutter-sokol", G_APPLICATION_DEFAULT_FLAGS);
  g_signal_connect (app, "activate", G_CALLBACK (activate), NULL);
  status = g_application_run (G_APPLICATION (app), argc, argv);
  g_object_unref (app);

  return status;
}
