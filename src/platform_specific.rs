use sdl3::sys::{properties::{SDL_CreateProperties, SDL_DestroyProperties, SDL_SetPointerProperty}, video::{SDL_CreateWindowWithProperties, SDL_DestroyWindow, SDL_RaiseWindow, SDL_PROP_WINDOW_CREATE_WIN32_HWND_POINTER}};

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