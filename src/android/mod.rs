use jni::objects::{JClass, JObject, JString, JValue, JValueGen};
use jni::{JNIEnv, JavaVM};
use jni_min_helper::JniClassLoader;
use log::info;
use std::sync::{LazyLock, Mutex};

const NOTIFICATION_ID: i32 = 1337;
const CHANNEL_ID: &str = "my_channel_id";

static SERVICE_MUTEX: LazyLock<Mutex<(bool, flume::Sender<()>, flume::Receiver<()>)>> =
    LazyLock::new(|| {
        let (s, r) = flume::unbounded();
        Mutex::new((false, s, r))
    });

/// JNI entry point called by the Java service's onCreate method.
#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn Java_com_test_foregroundservice_RustService_startRustLogic(
    mut env: JNIEnv,
    _class: JClass,
    service: JObject,
) {
    let mut data = (*SERVICE_MUTEX).lock().expect("expected valid lock");
    if data.0 {
        return;
    }
    // We are running
    data.0 = true;
    let r = data.2.clone();
    drop(data);

    // First, create the notification channel. This is safe to call multiple times.
    if let Err(e) = create_notification_channel(&mut env, &service) {
        log::error!(
            "Failed to create notification channel from service: {:?}",
            e
        );
        return;
    }

    // Next, build the notification object.
    let notification = match build_media_notification(&mut env, &service) {
        Ok(n) => n,
        Err(e) => {
            log::error!("Failed to build notification: {:?}", e);
            return;
        }
    };

    // Finally, call service.startForeground() to make this a foreground service.
    // This is the key step!
    let result = env.call_method(
        &service,
        "startForeground",
        "(ILandroid/app/Notification;)V",
        &[JValue::Int(NOTIFICATION_ID), JValue::Object(&notification)],
    );

    match result {
        Err(e) => {
            log::error!("Failed to call startForeground: {:?}", e);
        }
        Ok(_) => {
            let vm = env.get_java_vm().unwrap();
            let service_ref = env.new_global_ref(service).unwrap();
            std::thread::spawn(move || {
                let mut env = vm.attach_current_thread().unwrap();
                log::info!("Successfully started foreground service from Rust!");

                // Block until we need to stop
                _ = r.recv();
                log::info!("Foreground service is done, removing notification!");
                let mut data = (*SERVICE_MUTEX).lock().expect("expected valid lock");
                data.0 = false;

                env.call_method(
                    &service_ref,
                    "stopForeground",
                    "(Z)V",
                    &[JValue::Bool(true.into())],
                ).unwrap();
            });
        }
    }
}

/// JNI entry point for cleanup.
#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn Java_com_test_foregroundservice_RustService_stopRustLogic(
    mut _env: JNIEnv,
    _class: JClass,
) {
    log::info!("Rust logic cleanup initiated.");

    let data = (*SERVICE_MUTEX).lock().expect("expected valid lock");
    if data.0 {
        return;
    }
    // Notify the thread created in startRustLogic to stop
    _ = data.1.send(());
}

pub fn create_notification_channel(env: &mut JNIEnv, context: &JObject) -> jni::errors::Result<()> {
    const BUILD_VERSION: &str = "android/os/Build$VERSION";
    const NOTIFICATION_CHANNEL: &str = "android/app/NotificationChannel";
    const NOTIFICATION_MANAGER: &str = "android/app/NotificationManager";

    let version_class = env.find_class(BUILD_VERSION)?;
    let sdk_int: i32 = env.get_static_field(version_class, "SDK_INT", "I")?.i()?;

    if sdk_int >= 26 {
        let channel_id: JString = env.new_string("my_channel_id")?;
        let channel_name: JString = env.new_string("Multimedia playback")?;
        let manager_class = env.find_class(NOTIFICATION_MANAGER)?;
        let importance: i32 = env
            .get_static_field(manager_class, "IMPORTANCE_DEFAULT", "I")?
            .i()?;

        let channel_class = env.find_class(NOTIFICATION_CHANNEL)?;
        let channel = env.new_object(
            channel_class,
            "(Ljava/lang/String;Ljava/lang/CharSequence;I)V",
            &[
                JValue::Object(&channel_id),
                JValue::Object(&channel_name),
                JValue::Int(importance),
            ],
        )?;

        let description: JString = env.new_string("Channel for important notifications.")?;
        env.call_method(
            &channel,
            "setDescription",
            "(Ljava/lang/String;)V",
            &[JValue::Object(&description)],
        )?;

        let service_name: JString = env.new_string("notification")?;
        let notification_manager = env
            .call_method(
                context,
                "getSystemService",
                "(Ljava/lang/String;)Ljava/lang/Object;",
                &[JValue::Object(&service_name)],
            )?
            .l()?;

        env.call_method(
            &notification_manager,
            "createNotificationChannel",
            "(Landroid/app/NotificationChannel;)V",
            &[JValue::Object(&channel)],
        )?;

        log::info!("Notification channel created successfully.");
    } else {
        log::info!(
            "Skipping notification channel creation for API level {}.",
            sdk_int
        );
    }

    Ok(())
}

// Not working correctly yet... but this displays the notification
fn build_media_notification<'a>(
    env: &mut JNIEnv<'a>,
    context: &JObject,
) -> jni::errors::Result<JObject<'a>> {
    let resources = env
        .call_method(
            context,
            "getResources",
            "()Landroid/content/res/Resources;",
            &[],
        )?
        .l()?;
    let package_name = env
        .call_method(context, "getPackageName", "()Ljava/lang/String;", &[])?
        .l()?;
    let icon_name = env.new_string("ic_launcher")?;
    let icon_type = env.new_string("drawable")?;
    let small_icon_id = env
        .call_method(
            &resources,
            "getIdentifier",
            "(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)I",
            &[
                JValue::Object(&icon_name),
                JValue::Object(&icon_type),
                JValue::Object(&package_name),
            ],
        )?
        .i()?;

    info!("Icon ID is {}", small_icon_id);

    // --- Create and configure NotificationCompat.Builder ---
    let builder_class = env.find_class("androidx/core/app/NotificationCompat$Builder")?;
    let channel_id_jstr = env.new_string(CHANNEL_ID)?;
    let mut builder = env.new_object(
        builder_class,
        "(Landroid/content/Context;Ljava/lang/String;)V",
        &[JValue::Object(context), JValue::Object(&channel_id_jstr)],
    )?;

    let title = env.new_string("Now Playing")?;
    let text = env.new_string("Song Title - Artist Name")?;
    builder = env
        .call_method(
            &builder,
            "setContentTitle",
            "(Ljava/lang/CharSequence;)Landroidx/core/app/NotificationCompat$Builder;",
            &[JValue::Object(&title)],
        )?
        .l()?;
    builder = env
        .call_method(
            &builder,
            "setContentText",
            "(Ljava/lang/CharSequence;)Landroidx/core/app/NotificationCompat$Builder;",
            &[JValue::Object(&text)],
        )?
        .l()?;
    builder = env
        .call_method(
            &builder,
            "setSmallIcon",
            "(I)Landroidx/core/app/NotificationCompat$Builder;",
            &[JValue::Int(small_icon_id)],
        )?
        .l()?;
    builder = env
        .call_method(
            &builder,
            "setOngoing",
            "(Z)Landroidx/core/app/NotificationCompat$Builder;",
            &[JValue::Bool(true.into())],
        )?
        .l()?;

    // --- Apply MediaStyle ---
    let media_style_class = env.find_class("androidx/media/app/NotificationCompat$MediaStyle")?;
    let media_style = env.new_object(media_style_class, "()V", &[])?;
    builder = env.call_method(&builder, "setStyle", "(Landroidx/core/app/NotificationCompat$Style;)Landroidx/core/app/NotificationCompat$Builder;", &[JValue::Object(&media_style)])?.l()?;

    // --- Build and return the final Notification object ---
    let notification = env
        .call_method(&builder, "build", "()Landroid/app/Notification;", &[])?
        .l()?;
    Ok(notification)
}

pub fn start_foreground_service(app: slint::android::AndroidApp) {
    let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) }.expect("Unable to get JavaVM");
    let mut env = vm.attach_current_thread().expect("Unable to attach thread");

    let activity = unsafe { JObject::from_raw(app.activity_as_ptr().cast()) };

    let service_class = JniClassLoader::app_loader()
        .unwrap()
        .load_class("com.test.foregroundservice/RustService")
        .unwrap();

    let intent_class = env.find_class("android/content/Intent").unwrap();
    let intent = env
        .new_object(
            intent_class,
            "(Landroid/content/Context;Ljava/lang/Class;)V",
            &[JValue::Object(&activity), JValue::Object(&service_class)],
        )
        .unwrap();

    // On modern Android, we must use startForegroundService. The service then has
    // a few seconds to call startForeground() itself.
    env.call_method(
        &activity,
        "startForegroundService",
        "(Landroid/content/Intent;)Landroid/content/ComponentName;",
        &[JValueGen::Object(&intent)],
    )
    .map_err(|e| log::error!("Failed to start service: {:?}", e))
    .ok();
}

pub fn stop_foreground_service(app: slint::android::AndroidApp) {
let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) }.expect("Unable to get JavaVM");
    let mut env = vm.attach_current_thread().expect("Unable to attach thread");

    let activity = unsafe { JObject::from_raw(app.activity_as_ptr().cast()) };

    let service_class = JniClassLoader::app_loader()
        .unwrap()
        .load_class("com.test.foregroundservice/RustService")
        .unwrap();

    let intent_class = env.find_class("android/content/Intent").unwrap();
    let intent = env
        .new_object(
            intent_class,
            "(Landroid/content/Context;Ljava/lang/Class;)V",
            &[JValue::Object(&activity), JValue::Object(&service_class)],
        )
        .unwrap();

    env.call_method(
        &activity,
        "stopService",
        "(Landroid/content/Intent;)Z",
        &[JValueGen::Object(&intent)],
    )
    .map_err(|e| log::error!("Failed to stop service: {:?}", e))
    .ok();
}