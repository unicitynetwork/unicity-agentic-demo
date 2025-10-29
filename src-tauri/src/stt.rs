#![allow(non_snake_case)]

use tauri::{AppHandle, Emitter};
use tracing::{info, debug, error};
use std::sync::{Mutex, Arc};
use once_cell::sync::Lazy;

// Global instance to avoid creating multiple recognizers
static SPEECH_RECOGNIZER: Lazy<Arc<Mutex<Option<SpeechRecognizerInner>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

pub struct SpeechRecognizer {
    app_handle: AppHandle,
}

struct SpeechRecognizerInner {
    is_active: bool,
}

impl SpeechRecognizer {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }

    pub async fn start_recognition(&self) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            // Check if already running
            let mut recognizer = SPEECH_RECOGNIZER.lock().unwrap();
            if let Some(inner) = recognizer.as_ref() {
                if inner.is_active {
                    info!("🎤 Speech recognition already active");
                    return Ok(());
                }
            }

            unsafe { start_recognition_macos(&self.app_handle) }?;

            // Mark as active
            *recognizer = Some(SpeechRecognizerInner { is_active: true });
            Ok(())
        }
        #[cfg(not(target_os = "macos"))]
        { Err("Speech recognition is only implemented on macOS".into()) }
    }

    pub async fn stop_recognition(&self) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            let mut recognizer = SPEECH_RECOGNIZER.lock().unwrap();
            if let Some(inner) = recognizer.as_ref() {
                if !inner.is_active {
                    info!("🛑 Speech recognition not active");
                    return Ok(());
                }
            }

            unsafe { stop_recognition_macos(); }

            // Mark as inactive
            *recognizer = Some(SpeechRecognizerInner { is_active: false });
            Ok(())
        }
        #[cfg(not(target_os = "macos"))]
        { Ok(()) }
    }

    pub fn is_recognizing(&self) -> bool {
        #[cfg(target_os = "macos")]
        {
            let recognizer = SPEECH_RECOGNIZER.lock().unwrap();
            recognizer.as_ref().map(|r| r.is_active).unwrap_or(false)
        }
        #[cfg(not(target_os = "macos"))]
        { false }
    }
}

// ======================= macOS (Objective‑C bridge) ==========================
#[cfg(target_os = "macos")]
mod mac {
    pub use {
        cocoa_foundation::base::{id, nil, YES, NO},
        cocoa_foundation::foundation::NSAutoreleasePool,
        objc::runtime::Class,
        objc::{msg_send, sel, sel_impl},
        std::ffi::c_void,
        std::ops::Deref,
        std::ptr,
        libc,
    };
}

#[cfg(target_os = "macos")]
use mac::*;

#[cfg(target_os = "macos")]
static mut RECOGNIZER: id = nil;
#[cfg(target_os = "macos")]
static mut AUDIO_ENGINE: id = nil;
#[cfg(target_os = "macos")]
static mut REQUEST: id = nil;
#[cfg(target_os = "macos")]
static mut TASK: id = nil;
#[cfg(target_os = "macos")]
static mut TAP_BLOCK: *mut c_void = std::ptr::null_mut();
#[cfg(target_os = "macos")]
static mut LAST_PARTIAL_LEN: usize = 0;
#[cfg(target_os = "macos")]
static mut RESULT_HANDLER_BLOCK: *mut c_void = std::ptr::null_mut();

#[cfg(target_os = "macos")]
unsafe fn force_load_framework(path: &str) {
    let c = std::ffi::CString::new(path).unwrap();
    let handle = libc::dlopen(c.as_ptr(), libc::RTLD_NOW | libc::RTLD_LOCAL);
    if handle.is_null() {
        error!("Failed to load framework: {}", path);
    } else {
        info!("✅ Loaded framework: {}", path);
    }
}

#[cfg(target_os = "macos")]
unsafe fn start_recognition_macos(app: &AppHandle) -> Result<(), String> {
    let pool = NSAutoreleasePool::new(nil);

    info!("🎤 Starting speech recognition engine");

    // Load frameworks
    force_load_framework("/System/Library/Frameworks/Speech.framework/Speech");
    force_load_framework("/System/Library/Frameworks/AVFoundation.framework/AVFoundation");

    // Get classes
    let cls_SFSpeechRecognizer = Class::get("SFSpeechRecognizer")
        .ok_or("SFSpeechRecognizer not found")?;
    let cls_SFSpeechAudioBufferRecognitionRequest = Class::get("SFSpeechAudioBufferRecognitionRequest")
        .ok_or("SFSpeechAudioBufferRecognitionRequest not found")?;
    let cls_AVAudioEngine = Class::get("AVAudioEngine")
        .ok_or("AVAudioEngine not found")?;
    let cls_AVAudioSession = Class::get("AVAudioSession")
        .ok_or("AVAudioSession not found")?;

    // ============== CRITICAL: CHECK MICROPHONE PERMISSION FIRST ==============
    info!("🎙️ Checking microphone permission");
    let session: id = msg_send![cls_AVAudioSession, sharedInstance];

    // recordPermission returns enum values - these are FourCharCode/u32 values
    let mic_permission: u32 = msg_send![session, recordPermission];

    info!("🎙️ Microphone permission raw value: {} (0x{:X})", mic_permission, mic_permission);

    // Known values (from testing):
    // granted = 1735552628 (0x6772616E) = 'gran' in ASCII
    // denied  = 1684369017 (0x64656E79) = 'deny' in ASCII
    // undetermined = 1970168948 (0x756E6474) = 'undt' in ASCII

    const GRANTED: u32 = 1735552628;     // 0x6772616E = 'gran'
    const DENIED: u32 = 1684369017;       // 0x64656E79 = 'deny'
    const UNDETERMINED: u32 = 1970168948; // 0x756E6474 = 'undt'

    // More defensive check - match on known values
    match mic_permission {
        DENIED => {
            error!("❌ Microphone permission DENIED");
            let _: () = msg_send![pool, drain];
            return Err("Microphone access denied. Please enable in System Settings > Privacy & Security > Microphone.".into());
        }
        GRANTED => {
            info!("✅ Microphone permission GRANTED - continuing...");
            // Continue to speech recognition setup
        }
        UNDETERMINED => {
            info!("📋 Microphone permission UNDETERMINED - requesting...");
            // We need to request permission - this is async and will show a dialog
            let handler_app = app.clone();
            let handler = {
                use block::ConcreteBlock;
                let blk = ConcreteBlock::new(move |granted: bool| {
                    if granted {
                        info!("✅ Microphone permission granted by user");
                        let _ = handler_app.emit("stt://mic-permission", "granted");
                    } else {
                        error!("❌ Microphone permission denied by user");
                        let _ = handler_app.emit("stt://mic-permission", "denied");
                    }
                });
                let copied = blk.copy();
                let leaked: &'static _ = Box::leak(Box::new(copied));
                leaked.deref() as *const _ as *mut c_void
            };

            let _: () = msg_send![session, requestRecordPermission: handler];
            let _: () = msg_send![pool, drain];
            return Err("Microphone permission requested. Please grant permission and try again.".into());
        }
        _ => {
            // Unknown value - maybe macOS returns different values?
            // Be defensive: if we got here and it's not explicitly denied, try to continue
            info!("⚠️ Unknown microphone permission value: {} - attempting to continue anyway", mic_permission);
            // Fall through to continue
        }
    }

    // ============== CHECK SPEECH RECOGNITION PERMISSION ==============
    let speech_auth: isize = msg_send![cls_SFSpeechRecognizer, authorizationStatus];
    info!("🔐 Speech recognition authorization status: {}", speech_auth);

    match speech_auth {
        3 => { // Authorized
            info!("✅ Speech recognition authorized");
        }
        0 => { // NotDetermined
            info!("📋 Requesting speech recognition authorization");
            let handler_app = app.clone();
            let handler = {
                use block::ConcreteBlock;
                let blk = ConcreteBlock::new(move |status: isize| {
                    let status_text = match status {
                        0 => "NotDetermined",
                        1 => "Denied",
                        2 => "Restricted",
                        3 => "Authorized",
                        _ => "Unknown",
                    };
                    info!("🔐 Speech authorization changed to: {}", status_text);
                    let _ = handler_app.emit("stt://speech-auth", status_text);
                });
                let copied = blk.copy();
                let leaked: &'static _ = Box::leak(Box::new(copied));
                leaked.deref() as *const _ as *mut c_void
            };

            let _: () = msg_send![cls_SFSpeechRecognizer, requestAuthorization: handler];
            let _: () = msg_send![pool, drain];
            return Err("Speech recognition authorization requested. Please grant permission and try again.".into());
        }
        1 => { // Denied
            error!("❌ Speech recognition denied");
            let _: () = msg_send![pool, drain];
            return Err("Speech recognition denied. Please enable in System Settings > Privacy & Security > Speech Recognition.".into());
        }
        2 => { // Restricted
            error!("⚠️ Speech recognition restricted");
            let _: () = msg_send![pool, drain];
            return Err("Speech recognition restricted by device policy.".into());
        }
        _ => {
            error!("❓ Unknown authorization status: {}", speech_auth);
            let _: () = msg_send![pool, drain];
            return Err("Unknown authorization status.".into());
        }
    }

    // ============== INITIALIZE SPEECH RECOGNIZER ==============
    info!("🔧 Initializing speech recognizer");
    RECOGNIZER = msg_send![cls_SFSpeechRecognizer, alloc];
    RECOGNIZER = msg_send![RECOGNIZER, init];
    let _: id = msg_send![RECOGNIZER, retain];

    let available: bool = msg_send![RECOGNIZER, isAvailable];
    if !available {
        error!("❌ Speech recognizer not available");
        let _: () = msg_send![RECOGNIZER, release];
        RECOGNIZER = nil;
        let _: () = msg_send![pool, drain];
        return Err("Speech recognizer not available on this device".into());
    }

    // ============== SETUP AUDIO SESSION ==============
    info!("🎙️ Setting up audio session");
    let mut err: id = nil;

    let nsstring_cls = Class::get("NSString").ok_or("NSString not found")?;
    let cat_cstr = std::ffi::CString::new("AVAudioSessionCategoryRecord").unwrap();
    let mode_cstr = std::ffi::CString::new("AVAudioSessionModeMeasurement").unwrap();
    let category: id = msg_send![nsstring_cls, stringWithUTF8String: cat_cstr.as_ptr()];
    let mode: id = msg_send![nsstring_cls, stringWithUTF8String: mode_cstr.as_ptr()];
    let options: u64 = 0;

    let success: bool = msg_send![session,
        setCategory: category
        mode: mode
        options: options
        error: &mut err
    ];

    if !success || err != nil {
        let desc: id = msg_send![err, localizedDescription];
        let cstr: *const i8 = msg_send![desc, UTF8String];
        let msg = std::ffi::CStr::from_ptr(cstr).to_string_lossy().into_owned();
        error!("❌ Failed to set audio category: {}", msg);
        let _: () = msg_send![RECOGNIZER, release];
        RECOGNIZER = nil;
        let _: () = msg_send![pool, drain];
        return Err(format!("Failed to set audio category: {}", msg));
    }

    err = nil;
    let success: bool = msg_send![session, setActive: YES error: &mut err];
    if !success || err != nil {
        let desc: id = msg_send![err, localizedDescription];
        let cstr: *const i8 = msg_send![desc, UTF8String];
        let msg = std::ffi::CStr::from_ptr(cstr).to_string_lossy().into_owned();
        error!("❌ Failed to activate audio session: {}", msg);
        let _: () = msg_send![RECOGNIZER, release];
        RECOGNIZER = nil;
        let _: () = msg_send![pool, drain];
        return Err(format!("Failed to activate audio session: {}", msg));
    }

    // ============== SETUP AUDIO ENGINE ==============
    info!("🔧 Initializing audio engine");
    AUDIO_ENGINE = msg_send![cls_AVAudioEngine, alloc];
    AUDIO_ENGINE = msg_send![AUDIO_ENGINE, init];
    let _: id = msg_send![AUDIO_ENGINE, retain];

    REQUEST = msg_send![cls_SFSpeechAudioBufferRecognitionRequest, alloc];
    REQUEST = msg_send![REQUEST, init];
    let _: id = msg_send![REQUEST, retain];
    let _: () = msg_send![REQUEST, setShouldReportPartialResults: YES];

    // ============== INSTALL TAP ==============
    let input: id = msg_send![AUDIO_ENGINE, inputNode];
    let fmt: id = msg_send![input, outputFormatForBus: 0_u32];

    let tap_block_ptr = {
        use block::ConcreteBlock;
        let req = REQUEST;
        let blk = ConcreteBlock::new(move |buffer: id, _when: id| {
            unsafe {
                let _: () = msg_send![req, appendAudioPCMBuffer: buffer];
            }
        });
        let copied = blk.copy();
        let leaked: &'static _ = Box::leak(Box::new(copied));
        TAP_BLOCK = leaked.deref() as *const _ as *mut c_void;
        TAP_BLOCK
    };

    let _: () = msg_send![input,
        installTapOnBus: 0_u32
        bufferSize: 1024_u32
        format: fmt
        block: tap_block_ptr
    ];

    // ============== START ENGINE ==============
    info!("🚀 Starting audio engine");
    let _: () = msg_send![AUDIO_ENGINE, prepare];
    err = nil;
    let success: bool = msg_send![AUDIO_ENGINE, startAndReturnError: &mut err];
    if !success || err != nil {
        let desc: id = msg_send![err, localizedDescription];
        let cstr: *const i8 = msg_send![desc, UTF8String];
        let msg = std::ffi::CStr::from_ptr(cstr).to_string_lossy().into_owned();
        error!("❌ Failed to start audio engine: {}", msg);

        // Cleanup
        let _: () = msg_send![input, removeTapOnBus: 0_u32];
        let _: () = msg_send![REQUEST, release];
        let _: () = msg_send![AUDIO_ENGINE, release];
        let _: () = msg_send![RECOGNIZER, release];
        RECOGNIZER = nil;
        AUDIO_ENGINE = nil;
        REQUEST = nil;
        TAP_BLOCK = std::ptr::null_mut();

        let _: () = msg_send![pool, drain];
        return Err(format!("Failed to start audio engine: {}", msg));
    }

    // ============== START RECOGNITION ==============
    info!("🎯 Creating recognition task");
    let handler = create_result_handler(app.clone());
    RESULT_HANDLER_BLOCK = handler;

    TASK = msg_send![RECOGNIZER,
        recognitionTaskWithRequest: REQUEST
        resultHandler: handler
    ];

    if TASK != nil {
        let _: id = msg_send![TASK, retain];
    }

    info!("✅ Speech recognition started successfully!");
    let _: () = msg_send![pool, drain];
    Ok(())
}

#[cfg(target_os = "macos")]
unsafe fn stop_recognition_macos() {
    let pool = NSAutoreleasePool::new(nil);

    info!("🛑 Stopping speech recognition");
    LAST_PARTIAL_LEN = 0;

    if AUDIO_ENGINE != nil {
        let input: id = msg_send![AUDIO_ENGINE, inputNode];
        let _: () = msg_send![input, removeTapOnBus: 0_u32];
        let _: () = msg_send![AUDIO_ENGINE, stop];
    }

    if REQUEST != nil {
        let _: () = msg_send![REQUEST, endAudio];
    }

    if TASK != nil {
        let _: () = msg_send![TASK, cancel];
    }

    // Release in reverse order
    if TASK != nil {
        let _: () = msg_send![TASK, release];
        TASK = nil;
    }

    if REQUEST != nil {
        let _: () = msg_send![REQUEST, release];
        REQUEST = nil;
    }

    if AUDIO_ENGINE != nil {
        let _: () = msg_send![AUDIO_ENGINE, release];
        AUDIO_ENGINE = nil;
    }

    if RECOGNIZER != nil {
        let _: () = msg_send![RECOGNIZER, release];
        RECOGNIZER = nil;
    }

    TAP_BLOCK = std::ptr::null_mut();
    RESULT_HANDLER_BLOCK = std::ptr::null_mut();

    info!("✅ Speech recognition stopped");
    let _: () = msg_send![pool, drain];
}

#[cfg(target_os = "macos")]
fn create_result_handler(app: AppHandle) -> *mut c_void {
    use block::ConcreteBlock;

    let blk = ConcreteBlock::new(move |result: id, error: id| {
        unsafe {
            let pool = NSAutoreleasePool::new(nil);

            if error != nil {
                let desc: id = msg_send![error, localizedDescription];
                let cstr: *const i8 = msg_send![desc, UTF8String];
                let msg = std::ffi::CStr::from_ptr(cstr).to_string_lossy().into_owned();
                error!("🔴 Recognition error: {}", msg);
                let _ = app.emit("stt://error", format!("recognition error: {}", msg));
                let _ = app.emit("stt://final", format!("[error] {}", msg));
                let _: () = msg_send![pool, drain];
                return;
            }

            if result != nil {
                let is_final: bool = msg_send![result, isFinal];
                let tr: id = msg_send![result, bestTranscription];
                let s: id = msg_send![tr, formattedString];
                let c: *const i8 = msg_send![s, UTF8String];
                let text = std::ffi::CStr::from_ptr(c).to_string_lossy().into_owned();

                if is_final {
                    LAST_PARTIAL_LEN = 0;
                    if !text.is_empty() {
                        debug!("🎤 Final: {}", text);
                        let _ = app.emit("stt://final", text);
                    }
                } else {
                    if text.len() != LAST_PARTIAL_LEN {
                        LAST_PARTIAL_LEN = text.len();
                        debug!("🎤 Partial: {}", text);
                        let _ = app.emit("stt://partial", text);
                    }
                }
            }

            let _: () = msg_send![pool, drain];
        }
    });

    let copied = blk.copy();
    let leaked: &'static _ = Box::leak(Box::new(copied));
    leaked.deref() as *const _ as *mut c_void
}

// ======================= non‑macOS stubs =====================================
#[cfg(not(target_os = "macos"))]
#[allow(dead_code)]
fn _stubs() {}