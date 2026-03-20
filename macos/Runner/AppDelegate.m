#import "AppDelegate.h"
@implementation AppDelegate

- (void) applicationDidFinishLaunching: (NSNotification *)not{
    [self makeWindow];
}

- (void) makeWindow {
    window = [[NSWindow alloc] initWithContentRect: NSMakeRect(400,400,400,200)
                                         styleMask: NSWindowStyleMaskTitled |
                                                    NSResizableWindowMask |
                                                    NSClosableWindowMask |
                                                    NSMiniaturizableWindowMask
                                           backing: NSBackingStoreBuffered
                                             defer: NO];

    [window setTitle:@"My Window Title"];
    textField = [[NSTextField alloc]initWithFrame:NSMakeRect(20, 70, 200, 22)];
    [[window contentView] addSubview:textField];

    button = [[NSButton alloc] initWithFrame:NSMakeRect(230, 65, 70, 32)];
    [button setTitle:@"print"];

    [button setTarget:self];
    [button setAction:@selector(printHello:)];
    [[window contentView] addSubview:button];

    textView = [[NSTextView alloc] initWithFrame:NSMakeRect(20, 100, 200, 22)];
    [textView setString:@"Hello, me!"];
    [textView setEditable: NO];
    [textView setSelectable: NO];

    [[window contentView] addSubview:textView];

    [window makeKeyAndOrderFront:NSApp];
}

- (void) printHello: (id) sender {
    if([[textView string] isEqual: @"Hello, me!"]){
        [textView setString:@"Heh, what's up?"];
    } else {
        [textView setString: @"Hello, me!"];
    }
}

@end
