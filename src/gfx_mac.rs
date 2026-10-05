//! macOS: puts canvases on screen as the contents of a Core Animation layer, no OpenGL.
//! Apple's OpenGL runs on a Metal shim that reserves ~200 MB per process; a layer costs only the image.

use claude_companion::canvas::Canvas;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{class, msg_send};
use objc2_app_kit::NSView;
use objc2_core_foundation::{CFData, CFRetained};
use objc2_core_graphics::{
    CGBitmapInfo, CGColorRenderingIntent, CGColorSpace, CGDataProvider, CGImage, CGImageAlphaInfo,
};
use objc2_foundation::NSString;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::error::Error;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};

pub struct Gfx {
    space: CFRetained<CGColorSpace>,
}

pub struct GlWin {
    pub window: Window,
    layer: Retained<AnyObject>,
    premultiplied: Vec<u8>,
}

impl Gfx {
    pub fn new(event_loop: &ActiveEventLoop, attrs: WindowAttributes) -> Result<(Gfx, GlWin), Box<dyn Error>> {
        let gfx = Gfx { space: CGColorSpace::new_device_rgb().ok_or("no RGB color space")? };
        let first = gfx.add_window(event_loop, attrs)?;
        Ok((gfx, first))
    }

    pub fn add_window(&self, event_loop: &ActiveEventLoop, attrs: WindowAttributes) -> Result<GlWin, Box<dyn Error>> {
        let window = event_loop.create_window(attrs)?;
        let RawWindowHandle::AppKit(h) = window.window_handle()?.as_raw() else { return Err("not an AppKit window".into()) };
        let view: &NSView = unsafe { h.ns_view.cast().as_ref() };
        // our own sublayer, so nothing winit draws into the view's layer can cover it
        let layer: Retained<AnyObject> = unsafe {
            view.setWantsLayer(true);
            let host: *mut AnyObject = msg_send![view, layer];
            let layer: Retained<AnyObject> = msg_send![class!(CALayer), new];
            let nearest = NSString::from_str("nearest");
            let _: () = msg_send![&*layer, setMagnificationFilter: &*nearest];
            let _: () = msg_send![&*layer, setAutoresizingMask: 2u32 | 16u32]; // width + height sizable
            let _: () = msg_send![&*layer, setFrame: view.bounds()];
            let _: () = msg_send![host, addSublayer: &*layer];
            layer
        };
        Ok(GlWin { window, layer, premultiplied: Vec::new() })
    }

    pub fn present(&self, w: &mut GlWin, canvas: &Canvas) {
        if canvas.width == 0 || canvas.height == 0 {
            return;
        }
        // Core Animation wants premultiplied alpha
        w.premultiplied.clear();
        w.premultiplied.extend(canvas.data.chunks_exact(4).flat_map(|p| {
            let a = p[3] as u32;
            [(p[0] as u32 * a / 255) as u8, (p[1] as u32 * a / 255) as u8, (p[2] as u32 * a / 255) as u8, p[3]]
        }));
        let Some(data) = (unsafe { CFData::new(None, w.premultiplied.as_ptr(), w.premultiplied.len() as _) }) else { return };
        let Some(provider) = CGDataProvider::with_cf_data(Some(&data)) else { return };
        let image = unsafe {
            CGImage::new(
                canvas.width,
                canvas.height,
                8,
                32,
                canvas.width * 4,
                Some(&self.space),
                CGBitmapInfo(CGImageAlphaInfo::PremultipliedLast.0),
                Some(&provider),
                std::ptr::null(),
                false,
                CGColorRenderingIntent::RenderingIntentDefault,
            )
        };
        let Some(image) = image else { return };
        unsafe {
            // no implicit fade between frames
            let _: () = msg_send![class!(CATransaction), begin];
            let _: () = msg_send![class!(CATransaction), setDisableActions: true];
            let ptr: *const AnyObject = (&*image as *const CGImage).cast();
            let _: () = msg_send![&*w.layer, setContents: ptr];
            let _: () = msg_send![class!(CATransaction), commit];
        }
    }
}
