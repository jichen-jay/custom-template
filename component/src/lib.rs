use wasmcloud_component::export;
use wasmcloud_component::wasi::logging::logging::Level;
struct CustomTemplateComponent;

impl CustomTemplateComponent {
    fn log(_level: Level, _context: String, _message: String) {
        // Self::log(Level::Info, "some context".to_string(), "some message".to_string());
    }
}

// export!(CustomTemplateComponent);
