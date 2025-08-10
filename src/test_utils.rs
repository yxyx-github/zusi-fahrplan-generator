use regex::Regex;
use std::fs;
use std::path::Path;

pub fn read_xml_file<P: AsRef<Path>>(path: P) -> String {
    let raw_xml = fs::read_to_string(path).unwrap();
    cleanup_xml(raw_xml)
}

pub fn cleanup_xml(raw_xml: String) -> String {
    let parsed_xml = Regex::new("[ \n\r\t]+").unwrap().replace_all(raw_xml.trim(), " ");
    let parsed_xml = Regex::new("> <").unwrap().replace_all(&parsed_xml, "><");
    let parsed_xml = Regex::new(" >").unwrap().replace_all(&parsed_xml, ">");
    let parsed_xml = Regex::new(" />").unwrap().replace_all(&parsed_xml, "/>");
    parsed_xml.to_string()
}
