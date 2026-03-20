#import <Cocoa/Cocoa.h>

#ifndef AppDelegate_h
#define AppDelegate_h

@interface AppDelegate : NSObject<NSApplicationDelegate> {
    NSWindow *window;
    NSTextView *textView;
    NSTextField *textField;
    NSButton *button;
}

- (void) makeWindow;
- (void) printHello: (id) sender;

@end

#endif /* AppDelegate_h */
