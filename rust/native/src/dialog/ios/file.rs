use std::{
    path::PathBuf,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

use objc2::{DefinedClass, runtime::AnyObject, sel};
use objc2::{
    MainThreadMarker, MainThreadOnly, define_class,
    ffi::{OBJC_ASSOCIATION_RETAIN_NONATOMIC, objc_setAssociatedObject},
    msg_send,
    rc::Retained,
    runtime::ProtocolObject,
};
use objc2_foundation::{NSArray, NSDictionary, NSObject, NSObjectProtocol, NSString, NSURL};
use objc2_ui_kit::{
    UIBarButtonItem, UIBarButtonSystemItem, UIDocumentBrowserViewController,
    UIDocumentBrowserViewControllerDelegate, UIDocumentPickerDelegate,
    UIDocumentPickerViewController, UIImagePickerController, UIImagePickerControllerDelegate,
    UIImagePickerControllerImageURL, UIImagePickerControllerInfoKey,
    UIImagePickerControllerSourceType, UIModalPresentationStyle, UINavigationControllerDelegate,
};
use objc2_uniform_type_identifiers::UTType;

use crate::dialog::file::{FileDialog, FileDialogAction, FileDialogResult};

/// Defines the shared state for the dialogue future.
struct SharedState {
    /// The result.
    result: Option<FileDialogResult>,
    /// The waker for the future.
    waker: Option<Waker>,
}

/// The future for the file picker.
pub struct FileDialogFuture {
    state: Arc<Mutex<SharedState>>,
}

impl Future for FileDialogFuture {
    type Output = FileDialogResult;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.lock().unwrap();
        if let Some(result) = state.result.take() {
            Poll::Ready(result)
        } else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

/// Show an iOS file picker.
pub fn show_file_picker(dialog: &FileDialog) -> FileDialogFuture {
    unsafe {
        let state = Arc::new(Mutex::new(SharedState {
            result: None,
            waker: None,
        }));
        dispatch2::run_on_main(|mtm| {
            let root = super::root_view_controller(mtm);

            let picker = UIDocumentPickerViewController::alloc(mtm);
            match &dialog.action {
                FileDialogAction::Pick { extensions } => {
                    let state_clone = Arc::clone(&state);
                    let picker =
                        DocumentPickerViewController::new(mtm, extensions.clone(), state_clone);
                    root.presentViewController_animated_completion(&picker, true, None);
                }
                FileDialogAction::PickImage { use_camera } => {
                    let state_clone = Arc::clone(&state);
                    let picker = ImagePickerViewController::new(mtm, state_clone);
                    if *use_camera {
                        picker.setSourceType(UIImagePickerControllerSourceType::Camera);
                    }
                    root.presentViewController_animated_completion(&picker, true, None);
                }
                FileDialogAction::Browse => {
                    let state_clone = Arc::clone(&state);
                    let browser = DocumentBrowserViewController::new(mtm, state_clone);
                    browser.setAllowsDocumentCreation(false);
                    browser.setAllowsPickingMultipleItems(false);
                    root.presentViewController_animated_completion(&browser, true, None);
                }
                FileDialogAction::Export { paths } => {
                    let urls: Vec<Retained<NSURL>> = paths
                        .iter()
                        .filter_map(|path| NSURL::from_path(&path, false, None))
                        .collect();
                    let urls = NSArray::from_retained_slice(&urls);
                    let picker = UIDocumentPickerViewController::initForExportingURLs_asCopy(
                        picker, &urls, true,
                    );
                    // let content_types = vec![UTTypeImage];
                    // let content_types = NSArray::from_slice(&content_types);

                    // let picker = UIDocumentPickerViewController::initForOpeningContentTypes(
                    //     picker,
                    //     &content_types,
                    // );
                    picker.setShouldShowFileExtensions(true);

                    let state_clone = Arc::clone(&state);
                    let delegate = Delegate::new(mtm, state_clone);
                    picker.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));

                    // Some Obj-C black magic here so the delegate doesn’t get dropped too soon.
                    objc_setAssociatedObject(
                        &*picker as *const _ as *mut _,
                        "delegate_key" as *const _ as *const _,
                        &*delegate as *const _ as *mut _,
                        OBJC_ASSOCIATION_RETAIN_NONATOMIC,
                    );

                    root.presentViewController_animated_completion(&picker, true, None);
                }
            }
        });

        FileDialogFuture { state }
    }
}

/// Variables for the file picker delegate.
struct DelegateIvars {
    /// The state for the async future.
    state: Arc<Mutex<SharedState>>,
}

define_class!(
    /// The delegate for the file picker.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = DelegateIvars]
    struct Delegate;

    unsafe impl NSObjectProtocol for Delegate {}

    unsafe impl UIDocumentPickerDelegate for Delegate {
        #[unsafe(method(documentPicker:didPickDocumentsAtURLs:))]
        unsafe fn document_picker_did_pick_urls(
            &self,
            _controller: &UIDocumentPickerViewController,
            urls: &NSArray<NSURL>,
        ) {
            unsafe {
                self.did_pick_urls(urls);
            }
        }

        #[unsafe(method(documentPicker:didPickDocumentAtURL:))]
        unsafe fn document_picker_did_pick_url(
            &self,
            _controller: &UIDocumentPickerViewController,
            url: &NSURL,
        ) {
            unsafe {
                self.did_pick_url(url);
            }
        }

        #[unsafe(method(documentPickerWasCancelled:))]
        unsafe fn picker_was_cancelled(&self, _controller: &UIDocumentPickerViewController) {
            println!("cancelled");
            self.send_result(FileDialogResult::Cancelled);
        }
    }
);

impl Delegate {
    /// Creates a new delegate.
    fn new(mtm: MainThreadMarker, state: Arc<Mutex<SharedState>>) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(DelegateIvars { state });
        unsafe { msg_send![super(this), init] }
    }

    /// Sends a result to the future.
    fn send_result(&self, result: FileDialogResult) {
        let state = self.ivars().state.clone();
        let mut state = state.lock().unwrap();
        state.result = Some(result);
        if let Some(waker) = state.waker.take() {
            println!("waking waker");
            waker.wake();
        }
    }

    /// The method called when URLs are selected.
    unsafe fn did_pick_urls(&self, urls: &NSArray<NSURL>) {
        let paths: Vec<PathBuf> = urls
            .iter()
            .filter_map(|url| url.path().map(|path| path.to_string().into()))
            .collect();
        self.send_result(FileDialogResult::Complete {
            selected_paths: paths,
        });
        println!("URLS {:?}", urls);
    }

    /// The method called when a URL is selected.
    unsafe fn did_pick_url(&self, url: &NSURL) {
        let Some(path) = url.path().map(|path| path.to_string().into()) else {
            return;
        };
        self.send_result(FileDialogResult::Complete {
            selected_paths: vec![path],
        });
    }
}

// MARK: - Document browser

define_class!(
    /// The delegate for the file picker.
    #[unsafe(super(UIDocumentBrowserViewController))]
    #[thread_kind = MainThreadOnly]
    #[ivars = DocumentBrowserViewControllerIvars]
    struct DocumentBrowserViewController;

    unsafe impl NSObjectProtocol for DocumentBrowserViewController {}

    unsafe impl UIDocumentBrowserViewControllerDelegate for DocumentBrowserViewController {
        #[unsafe(method(documentBrowser:didPickDocumentsAtURLs:))]
        unsafe fn document_browser_did_pick_urls(
            &self,
            controller: &UIDocumentBrowserViewController,
            urls: &NSArray<NSURL>,
        ) {
            unsafe {
                self.did_pick_urls(urls);
                controller.dismissViewControllerAnimated_completion(true, None);
            }
        }
    }

    impl DocumentBrowserViewController {
        #[unsafe(method(dismiss))]
        fn __dismiss(&self) {
            self.cancel();
        }
    }
);

impl DocumentBrowserViewController {
    /// Creates a new document browser.
    fn new(mtm: MainThreadMarker, state: Arc<Mutex<SharedState>>) -> Retained<Self> {
        unsafe {
            let this = Self::alloc(mtm).set_ivars(DocumentBrowserViewControllerIvars { state });
            let content_types: Option<&NSArray<NSString>> = None;
            let controller: Retained<Self> =
                msg_send![super(this), initForOpeningFilesWithContentTypes: content_types];
            controller.setDelegate(Some(ProtocolObject::from_ref(&*controller)));
            controller.setModalPresentationStyle(UIModalPresentationStyle::FullScreen);
            let cancel_button = UIBarButtonItem::alloc(mtm);
            let cancel_button = UIBarButtonItem::initWithBarButtonSystemItem_target_action(
                cancel_button,
                UIBarButtonSystemItem::Cancel,
                Some(&controller),
                Some(sel!(dismiss)),
            );
            let items = NSArray::from_retained_slice(&[cancel_button]);
            // items.add
            controller.setAdditionalTrailingNavigationBarButtonItems(&items);
            controller
        }
    }

    /// Cancel the document selection.
    fn cancel(&self) {
        self.dismissViewControllerAnimated_completion(true, None);
        self.send_result(FileDialogResult::Cancelled);
    }

    /// The method called when URLs are selected.
    unsafe fn did_pick_urls(&self, urls: &NSArray<NSURL>) {
        let paths: Vec<PathBuf> = urls
            .iter()
            .filter_map(|url| url.path().map(|path| path.to_string().into()))
            .collect();
        self.send_result(FileDialogResult::Complete {
            selected_paths: paths,
        });
    }

    /// Sends a result to the future.
    fn send_result(&self, result: FileDialogResult) {
        let state = self.ivars().state.clone();
        let mut state = state.lock().unwrap();
        state.result = Some(result);
        if let Some(waker) = state.waker.take() {
            waker.wake();
        }
    }
}

/// The document browser view controller variables.
struct DocumentBrowserViewControllerIvars {
    /// The state for the async future.
    state: Arc<Mutex<SharedState>>,
}

// MARK: - Document picker

define_class!(
    /// The file picker.
    #[unsafe(super(UIDocumentPickerViewController))]
    #[thread_kind = MainThreadOnly]
    #[ivars = DocumentPickerViewControllerIvars]
    struct DocumentPickerViewController;

    unsafe impl NSObjectProtocol for DocumentPickerViewController {}

    unsafe impl UIDocumentPickerDelegate for DocumentPickerViewController {
        #[unsafe(method(documentPicker:didPickDocumentsAtURLs:))]
        unsafe fn document_browser_did_pick_urls(
            &self,
            controller: &UIDocumentPickerViewController,
            urls: &NSArray<NSURL>,
        ) {
            unsafe {
                self.did_pick_urls(urls);
                controller.dismissViewControllerAnimated_completion(true, None);
            }
        }
    }

    impl DocumentPickerViewController {
        #[unsafe(method(dismiss))]
        fn __dismiss(&self) {
            self.cancel();
        }
    }
);

impl DocumentPickerViewController {
    /// Creates a new document browser.
    fn new(
        mtm: MainThreadMarker,
        extensions: Vec<&str>,
        state: Arc<Mutex<SharedState>>,
    ) -> Retained<Self> {
        unsafe {
            let this = Self::alloc(mtm).set_ivars(DocumentPickerViewControllerIvars { state });
            let content_types: Vec<Retained<UTType>> = extensions
                .iter()
                .filter_map(|extension| {
                    UTType::typeWithFilenameExtension(&NSString::from_str(&extension))
                })
                .collect();
            // .iter()
            // .filter_map(|path| NSURL::from_path(&path, false, None))
            // .collect();
            // UIDocumentPickerViewController::initForOpeningContentTypes(this, content_types)
            let content_types = NSArray::from_retained_slice(&content_types);
            let content_types: &NSArray<UTType> = content_types.as_ref();
            let controller: Retained<Self> =
                msg_send![super(this), initForOpeningContentTypes: content_types];
            controller.setDelegate(Some(ProtocolObject::from_ref(&*controller)));
            controller
        }
    }

    /// Cancel the document selection.
    fn cancel(&self) {
        self.dismissViewControllerAnimated_completion(true, None);
        self.send_result(FileDialogResult::Cancelled);
    }

    /// The method called when URLs are selected.
    unsafe fn did_pick_urls(&self, urls: &NSArray<NSURL>) {
        let paths: Vec<PathBuf> = urls
            .iter()
            .filter_map(|url| url.path().map(|path| path.to_string().into()))
            .collect();
        self.send_result(FileDialogResult::Complete {
            selected_paths: paths,
        });
    }

    /// Sends a result to the future.
    fn send_result(&self, result: FileDialogResult) {
        let state = self.ivars().state.clone();
        let mut state = state.lock().unwrap();
        state.result = Some(result);
        if let Some(waker) = state.waker.take() {
            waker.wake();
        }
    }
}

/// The document picker view controller variables.
struct DocumentPickerViewControllerIvars {
    /// The state for the async future.
    state: Arc<Mutex<SharedState>>,
}

// MARK: - Image picker

define_class!(
    /// The file picker.
    #[unsafe(super(UIImagePickerController))]
    #[thread_kind = MainThreadOnly]
    #[ivars = ImagePickerViewControllerIvars]
    struct ImagePickerViewController;

    unsafe impl NSObjectProtocol for ImagePickerViewController {}

    unsafe impl UINavigationControllerDelegate for ImagePickerViewController {}

    unsafe impl UIImagePickerControllerDelegate for ImagePickerViewController {
        #[unsafe(method(imagePickerController:didFinishPickingMediaWithInfo:))]
        unsafe fn did_finish_picking_media(
            &self,
            picker: &UIImagePickerController,
            info: &NSDictionary<UIImagePickerControllerInfoKey, AnyObject>,
        ) {
            unsafe {
                if let Some(url) = info.objectForKey(&UIImagePickerControllerImageURL) {
                    let url: Retained<NSURL> = url.downcast().unwrap();
                    self.did_pick_url(&url);
                }
            }
            picker.dismissViewControllerAnimated_completion(true, None);
        }
    }

    impl ImagePickerViewController {
        #[unsafe(method(dismiss))]
        fn __dismiss(&self) {
            self.cancel();
        }
    }
);

impl ImagePickerViewController {
    /// Creates a new image picker.
    fn new(mtm: MainThreadMarker, state: Arc<Mutex<SharedState>>) -> Retained<Self> {
        unsafe {
            let this = Self::alloc(mtm).set_ivars(ImagePickerViewControllerIvars { state });
            let controller: Retained<Self> = msg_send![super(this), init];
            controller.setDelegate(Some(&controller));
            controller
        }
    }

    /// Cancel the image selection.
    fn cancel(&self) {
        self.dismissViewControllerAnimated_completion(true, None);
        self.send_result(FileDialogResult::Cancelled);
    }

    /// The method called when URLs are selected.
    unsafe fn did_pick_url(&self, url: &NSURL) {
        let Some(path) = url.path().map(|path| path.to_string().into()) else {
            return;
        };
        self.send_result(FileDialogResult::Complete {
            selected_paths: vec![path],
        });
    }

    /// Sends a result to the future.
    fn send_result(&self, result: FileDialogResult) {
        let state = self.ivars().state.clone();
        let mut state = state.lock().unwrap();
        state.result = Some(result);
        if let Some(waker) = state.waker.take() {
            waker.wake();
        }
    }
}

/// The image picker view controller variables.
struct ImagePickerViewControllerIvars {
    /// The state for the async future.
    state: Arc<Mutex<SharedState>>,
}
