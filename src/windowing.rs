use sdl3::{sys::{properties::{SDL_CreateProperties, SDL_DestroyProperties, SDL_SetPointerProperty}, video::{SDL_CreateWindowWithProperties, SDL_DestroyWindow, SDL_RaiseWindow, SDL_PROP_WINDOW_CREATE_WIN32_HWND_POINTER}}, video::Window};
use sdl3_sys::{properties::SDL_GetPointerProperty, video::{SDL_GetWindowProperties, SDL_PROP_WINDOW_WIN32_HWND_POINTER, SDL_SetWindowAlwaysOnTop, SDL_SetWindowFocusable}};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetWindowLongPtrA, SetWindowLongPtrA, GWL_EXSTYLE, WS_EX_APPWINDOW, WS_EX_TOPMOST};

pub fn focus_game_window() {
    if cfg!(windows) {
        unsafe { 
            let hwnd = windows_sys::Win32::UI::WindowsAndMessaging::FindWindowW(windows_sys::w!("YYGameMakerYY"), windows_sys::w!("UNDERTALE"));
            if !hwnd.is_null() {
                let props = SDL_CreateProperties();
                if props == 0 {
                    return;
                }
                SDL_SetPointerProperty(props, SDL_PROP_WINDOW_CREATE_WIN32_HWND_POINTER, hwnd);
                let window = SDL_CreateWindowWithProperties(props);
                if window.is_null() {
                    return;
                }
                _ = SDL_RaiseWindow(window);
                SDL_DestroyWindow(window);
                SDL_DestroyProperties(props);
            }
        }
    }
}

pub fn window_set_focusable(window: &mut Window, focusable: bool) -> bool {
    let raw_window = window.raw();
    if !raw_window.is_null() {
        let main_result = unsafe { SDL_SetWindowFocusable(raw_window, focusable) };

        if cfg!(windows) {
            unsafe {
                // Make sure window icon always shows in Windows taskbar
                let window_hwnd = SDL_GetPointerProperty(SDL_GetWindowProperties(raw_window), SDL_PROP_WINDOW_WIN32_HWND_POINTER, std::ptr::null_mut());
                if !window_hwnd.is_null() {
                    if focusable {
                        SetWindowLongPtrA(window_hwnd, GWL_EXSTYLE, GetWindowLongPtrA(window_hwnd, GWL_EXSTYLE) & !(WS_EX_APPWINDOW as isize | WS_EX_TOPMOST as isize));
                    } else {
                        SetWindowLongPtrA(window_hwnd, GWL_EXSTYLE, GetWindowLongPtrA(window_hwnd, GWL_EXSTYLE) | WS_EX_APPWINDOW as isize | WS_EX_TOPMOST as isize);
                    }
                }
            }
        }

        main_result
    } else {
        false
    }
}

pub fn window_set_always_on_top(window: &mut Window, always_on_top: bool) -> bool {
    let raw_window = window.raw();
    if !raw_window.is_null() {
        let main_result = unsafe { SDL_SetWindowAlwaysOnTop(raw_window, always_on_top) };
        main_result
    } else {
        false
    }
}
