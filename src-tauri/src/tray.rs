use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{TrayIcon, TrayIconBuilder},
    AppHandle, Manager,
};

fn tray_icon() -> Image<'static> {
    let size = 32usize;
    let mut rgba = Vec::with_capacity(size * size * 4);
    for y in 0..size {
        for x in 0..size {
            let in_vertical = (5..=8).contains(&x);
            let in_top = (4..=15).contains(&y);
            let in_loop_side = (9..=15).contains(&x);
            let in_mid = (16..=19).contains(&y);
            let white = (in_vertical && in_top)
                || (in_loop_side && (in_top || in_mid))
                || (in_vertical && in_mid);
            if white {
                rgba.extend_from_slice(&[0xff, 0xff, 0xff, 0xff]);
            } else {
                rgba.extend_from_slice(&[0x1d, 0x4e, 0xd8, 0xff]);
            }
        }
    }
    Image::new_owned(rgba, size as u32, size as u32)
}

pub fn build_tray(app: &AppHandle) -> tauri::Result<TrayIcon> {
    let open = MenuItem::with_id(app, "open", "Öffnen", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Beenden", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &quit])?;

    let tray = TrayIconBuilder::new()
        .icon(tray_icon())
        .tooltip("Parkplatz")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(tray)
}
