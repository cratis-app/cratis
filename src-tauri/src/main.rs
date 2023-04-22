// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate lazy_static;

use std::{fs::{File, self, OpenOptions}, path::Path, env::current_dir, io::{ErrorKind, Write}};
use base64::{engine::general_purpose, Engine};
use comrak::{markdown_to_html, ComrakOptions};
use directories::ProjectDirs;
use regex::{Regex, Captures};
use serde::{Deserialize, Serialize};
use pulldown_cmark::{Parser, Options, html};

mod database;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![create_network, create_node, open_node, parse_md, save_node, search_nodes, get_journal_entries, database::create_database, database::index_nodes, database::add_node, database::update_references, database::get_node_referred, database::get_source_content, database::get_nodes, database::get_references, add_attachment])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn create_network(location: String, name: String) {
    let mut cratis_dir = location.clone();
    cratis_dir.push_str(&format!("/{}", &name));
    let mut nodes_dir = cratis_dir.clone();
    nodes_dir.push_str("/nodes");
    let mut journal_dir = cratis_dir.clone();
    journal_dir.push_str("/journal");
    let mut attachments_dir = cratis_dir.clone();
    attachments_dir.push_str("/attachments");

    // create network directories
    fs::create_dir_all(cratis_dir).expect("Could not create network directory");
    fs::create_dir_all(nodes_dir).expect("Could not create nodes directory");
    fs::create_dir_all(journal_dir).expect("Could not create journal directory");
    fs::create_dir_all(attachments_dir).expect("Could not create attachments dir");
}

#[tauri::command]
fn create_node(location: String, name: String) {
    let node_path = format!("{location}/{name}.md").to_string();
    if !Path::new(&node_path).exists() {
        let node_file = File::create(node_path).expect("Could not create node");
    }
}

#[tauri::command]
fn open_node(nodePath: String) -> String {
    fs::read_to_string(nodePath).expect("Could not open node")
}

#[tauri::command]
fn parse_md(content: String, attachmentsDir: String) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(&content, options);

    let mut parsed_md = String::new();
    html::push_html(&mut parsed_md, parser);
    // replace links and tags with button
    let btn_html = "<button key=\"$1\" class=\"nodeLink\">$0</button>";
    let link_re = Regex::new(r"\[\[(.+?)\]\]").unwrap();
    let result = link_re.replace_all(&parsed_md, btn_html);
    let tag_re = Regex::new(r"(?:^|\s)#(\w+)\b").unwrap();
    let result2 = tag_re.replace_all(&result, btn_html);
    // replace img src with base64
    let img_re = Regex::new(r#"(<img src=")([^"]*)("[^>]* alt="[^"]*")?( />)"#).unwrap();

    let img_paths: Vec<_> = img_re
        .captures_iter(&result2)
        .map(|cap| cap[2].to_string())
        .collect();

    let name_re = Regex::new(r#"/([^/]+)$"#).unwrap();

    let img_new_paths: Vec<_> = img_paths
        .iter()
        .map(|s| format!("{}{}", &attachmentsDir, name_re.captures(s).unwrap()[1].to_string()))
        .collect();

    let result3 = img_re.replace_all(&result2, |caps: &regex::Captures| {
        let src = caps.get(2).unwrap().as_str();
        let file_name = src.split('/').last().unwrap();
        let new_src = img_new_paths.iter()
            .find(|s| s.contains(file_name))
            .and_then(|path| fs::read(path).ok())
            .map(|bytes| base64::encode(&bytes));

        match new_src {
            Some(new_src) => format!("<img src=\"data:image/png;base64,{}\"{}>", new_src, caps.get(2).unwrap().as_str()),
            None => caps[0].to_string()
        }
    });

    result3.to_string() 
} 

#[tauri::command]
fn save_node(nodeStr: String, nodePath: String) {
    if nodePath != "" {
        let mut file = OpenOptions::new().write(true).open(nodePath).expect("Could not open file");
        file.write_all(nodeStr.as_bytes()).expect("Could not write node");
    }
}    

#[tauri::command]
fn search_nodes(searchVal: String, cratisDir: String) -> Vec<String> {
    let mut nodes = Vec::new();
    let mut nodes_dir = cratisDir.clone();
    nodes_dir.push_str("/nodes/");

    let node_paths = fs::read_dir(nodes_dir).unwrap();

    for path in node_paths {
        // iterate through all nodes and add to nodes vec if matches
        let mut node_name = path.unwrap().file_name().to_str().unwrap().to_string();
        node_name.truncate(node_name.len() - 3);
        
        if node_name.contains(&searchVal) {
            nodes.push(node_name);
        }
    }
    nodes
}

#[derive(Debug, Deserialize, Serialize)]
struct JournalEntry {
    date: String,
    content: String 
}

#[tauri::command]
fn get_journal_entries(journalDir: String) -> Vec<JournalEntry> {
    let mut entries: Vec<JournalEntry> = Vec::new();
    let journal_path = fs::read_dir(journalDir.clone()).unwrap();

    for path in journal_path {
        let mut entry_file = path.unwrap().file_name().to_str().unwrap().to_string();
        let mut entry_path = journalDir.clone();
        entry_path.push_str("/");
        entry_path.push_str(&format!("{}", entry_file));
        let entry_content = open_node(entry_path);
        entry_file.truncate(entry_file.len() - 3);
        let entry = JournalEntry {
            date: entry_file,
            content: entry_content 
        };
        entries.push(entry);
    }

    entries.sort_by(|a, b| a.date.cmp(&b.date));

    entries 
}

#[tauri::command]
fn add_attachment(fileName: String, attachmentsDir: String, imageData: String) -> String {
   let bytes = general_purpose::STANDARD.decode(&imageData).unwrap(); 
   let mut file_dir = attachmentsDir.clone();
   file_dir.push_str(&fileName);
   let mut file = fs::OpenOptions::new()
       .write(true)
       .create_new(true)
       .open(&file_dir).unwrap();
   file.write_all(&bytes);
   return fileName
}
