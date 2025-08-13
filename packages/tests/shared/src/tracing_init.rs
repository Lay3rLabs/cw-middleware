use std::sync::LazyLock;

static INIT: LazyLock<std::sync::Mutex<bool>> = LazyLock::new(|| std::sync::Mutex::new(false));

// just initialize once for all threads
pub fn tracing_tests_init() {
    let mut init = INIT.lock().unwrap();

    if !*init {
        *init = true;

        tracing_subscriber::fmt::init();
    }
}
