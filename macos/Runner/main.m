#include <stdlib.h>
#include <stdio.h>
#include <dlfcn.h>

#import <Foundation/Foundation.h>
#import "AppDelegate.h"

void load_library(void) {
    void *handle = dlopen("sokol_layer.framework/sokol_layer", RTLD_LAZY);
    if (!handle) {
        fputs (dlerror(), stderr);
        exit(1);
    }

    int (*launch_app)(void) = dlsym(handle, "launch_app");
    char *error;
    if ((error = dlerror()) != NULL)  {
        fputs(error, stderr);
        exit(1);
    }
    launch_app();
}

int main(void) {
    load_library();
    [NSApplication sharedApplication];
    AppDelegate *appDelegate = [AppDelegate new];
    [NSApp setDelegate:appDelegate];
    [NSApp run];
    return 0;
}

