// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate lazy_static;

use std::{fs::{File, self, OpenOptions}, path::Path, env::current_dir, io::{ErrorKind, Write}};
use comrak::{markdown_to_html, ComrakOptions};
use directories::ProjectDirs;
use regex::{Regex, Captures};
use serde::{Deserialize, Serialize};

mod database;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![create_network, create_node, open_node, parse_md, save_node, search_nodes, get_journal_entries, database::create_database, database::index_nodes, database::add_node, database::update_references, database::get_node_referred, database::get_source_content, database::get_nodes, database::get_references])
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

    // create network directories
    fs::create_dir_all(cratis_dir).expect("Could not create network directory");
    fs::create_dir_all(nodes_dir).expect("Could not create nodes directory");
    fs::create_dir_all(journal_dir).expect("Could not create journal directory");
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
fn parse_md(content: String) -> String {
    let mut opts = ComrakOptions::default();
    opts.extension.strikethrough = true;
    opts.extension.table = true;
    opts.extension.tasklist = true;
    opts.parse.smart = true;
    let parsed_md = markdown_to_html(&content, &opts);
    let btn_html = "<button key=\"$1\" class=\"nodeLink\">$0</button>";
    let link_re = Regex::new(r"\[\[(.+?)\]\]").unwrap();
    let result = link_re.replace_all(&parsed_md, btn_html);
    let tag_re = Regex::new(r"(?:^|\s)#(\w+)\b").unwrap();
    let result2 = tag_re.replace_all(&result, btn_html);
    result2.to_string() 
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
