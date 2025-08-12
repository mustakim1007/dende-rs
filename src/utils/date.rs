use chrono::Local;

/// Return curent date and time
pub fn timestamp() -> String {
    Local::now().format("%Y/%m/%d %H:%M:%S").to_string()
}