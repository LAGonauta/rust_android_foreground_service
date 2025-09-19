#[cfg(target_os = "android")]
mod android;

use log::info;

slint::include_modules!();

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: slint::android::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Trace)
            .with_tag("template"),
    );

    slint::android::init(app.clone()).unwrap();

    let ui = AppWindow::new().unwrap();
    ui.set_is_android(true);

    ui.on_start_foreground_service({
        let app = app.clone();
        move || {
            info!("Starting service");
            crate::android::start_foreground_service(app.clone());
            info!("Service started");
        }
    });
    ui.on_stop_foreground_service({
        let app = app.clone();
        move || {
            info!("Stopping service");
            crate::android::stop_foreground_service(app.clone());
            info!("Service stopped");
        }
    });

    ui.run().unwrap();
}

pub fn main() -> anyhow::Result<()> {
    info!("Loading UI");
    let ui = AppWindow::new().unwrap();
    ui.run()?;
    Ok(())
}
