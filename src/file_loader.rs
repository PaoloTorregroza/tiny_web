use std::fs;

pub fn read_html_file(location: &String) -> String {
    if !location.contains(".html") {
        return "".to_string();
    }

    fs::read_to_string(location)
        .expect("Should have been able to read the file")
}
