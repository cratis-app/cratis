use std::{fs, io::Read, path::Path};

use rusqlite::{Connection, Result, Batch};
use regex::Regex;
use log::{debug, LevelFilter};
use env_logger::{Env, Builder};
use serde::Serialize;
use serde_json::Value;
use toml::value::Table;
use std::sync::Mutex;
use lazy_static::lazy_static;
use std::io::Write;
use std::fs::ReadDir;
use std::collections::HashMap;
use core::str::Split;

lazy_static! {
    static ref LOGGER: Mutex<Option<Builder>> = Mutex::new(None);
}

fn init_logger() {
    let mut builder = Builder::from_env(Env::default().default_filter_or("debug"));
    builder.format(|buf, record| {
        writeln!(buf, "{} [{}] - {}", record.level(), record.target(), record.args())
    });

    let _ = builder.try_init();
    let _ = LOGGER.lock().unwrap().replace(builder);
}

#[tauri::command]
pub fn create_database(databaseDir: String) {
    let conn = Connection::open(&databaseDir).expect("Could not connect to db");
    // Create tables if they do not exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS nodes (
            id INTEGER PRIMARY KEY,
            name TEXT UNIQUE
        )",
        (),
    ).expect("Could not create table nodes");
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS links (
            source_node TEXT REFERENCES nodes(name),
            target_node TEXT REFERENCES nodes(name)
        )",
        (),
    ).expect("Could not create table links");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS properties (
            node TEXT REFERENCES nodes(name),
            property TEXT NOT NULL,
            value TEXT
        )",
        (),
    ).expect("Could not create table properties");
}

#[derive(Debug, Serialize)]
pub struct Link {
    source_node: String,
    target_node: String
}

#[tauri::command]
pub fn index_nodes(databaseDir: String, nodesDir: String) -> Result<(), String> {
    let mut conn = Connection::open(&databaseDir).expect("Could not connect to db");

    if LOGGER.lock().unwrap().is_none() {
        init_logger();
    }

    // enable logging
    conn.trace(Some(|sql| {
        debug!("Executing SQL statement: {}", sql);
    }));

    let mut nodes_dir = nodesDir.clone();
    nodes_dir.push_str("/nodes/");
    
    let mut journal_dir = nodesDir.clone();
    journal_dir.push_str("/journal/");

    let node_paths = fs::read_dir(nodes_dir).unwrap();
    let journal_paths = fs::read_dir(journal_dir).unwrap();
    // empty tables
    conn.execute_batch(
        "DELETE FROM links;
        DELETE FROM properties;
        DELETE FROM nodes;"
    ).expect("Could not delete tables");

    index_dir(node_paths, &conn);
    index_dir(journal_paths, &conn);

    let mut ref_stmt = conn.prepare("SELECT source_node, target_node FROM links").unwrap();
    let refers = ref_stmt.query_map([], |row| {
        Ok(Link {
            source_node: row.get(0).unwrap(),
            target_node: row.get(1).unwrap()
        })
    }).unwrap();

    for refer in refers {
        println!("REFERENCE: {:?}\n\n", refer.unwrap());
    }

    Ok(())
}

fn index_dir(paths: ReadDir, conn: &Connection) {
    for path_result in paths {
        let path = path_result.unwrap().path();
        let mut node_name = path.file_name().unwrap().to_str().unwrap().to_string();
        node_name.truncate(node_name.len() - 3);
        
        // add to nodes table
        conn.execute(
            "INSERT OR IGNORE INTO nodes (name) VALUES (?1)",
            [&node_name],
        ).expect("Could not insert into nodes table");

        // find references
        let mut file = fs::File::open(&path).unwrap();
        let mut node_content = String::new();
        file.read_to_string(&mut node_content).unwrap();
        let lines = &node_content.split("\n").collect::<Vec<&str>>();
        if lines[0] == "+++" {
            // update properties
            let parts = &node_content.split("+++").collect::<Vec<&str>>();
            let frontmatter = parts[1];
            let props: HashMap<String, String> = toml::from_str(frontmatter).unwrap();

            for (k, v) in props.iter() {
                conn.execute("INSERT INTO properties (node, property, value) VALUES (?1, ?2, ?3)", [&node_name, &k, &v]).expect("Could not insert into properties table"); 
            }
        }

        // update refs
        let re_link = Regex::new(r"\[\[(.+?)\]\]").unwrap();
        let re_tag = Regex::new(r"(?:^|\s)#(\w+)\b").unwrap();
        let mut refs: Vec<String> = Vec::new();

        for link in re_link.captures_iter(&node_content) {
            refs.push(link.get(1).unwrap().as_str().to_string());
        }

        for tag in re_tag.captures_iter(&node_content) {
            refs.push(tag.get(1).unwrap().as_str().to_string());
        }

        for link in refs {
            let sql = r"
                INSERT OR IGNORE INTO nodes (name) VALUES (?2);
                INSERT INTO links (source_node, target_node) VALUES (?1, ?2);
                ";
            let mut batch = Batch::new(&conn, sql);
            while let Some(mut stmt) = batch.next().unwrap() {
                stmt.execute([&node_name, &link]).unwrap();
            }
        }
    }
}

#[tauri::command]
pub fn add_node(databaseDir: String, nodeName: String) {
    let conn = Connection::open(&databaseDir).expect("Could not open db");
    
    conn.execute("
                 INSERT OR IGNORE INTO nodes (name) VALUES (?1)
                 ",
                 [&nodeName]
    ).expect("Could not insert node into table nodes");
}

#[tauri::command]
pub fn update_references(databaseDir: String, nodePath: String) {
    let mut conn = Connection::open(&databaseDir).expect("Could not open db");

    // if LOGGER.lock().unwrap().is_none() {
    //     init_logger();
    // }
    //
    // // enable logging
    // conn.trace(Some(|sql| {
    //     debug!("Executing SQL statement: {}", sql);
    // }));
    
    let mut file = fs::File::open(&nodePath).expect("Could not open file");
    let mut node_content = String::new();
    file.read_to_string(&mut node_content).unwrap();

    let node_path = Path::new(&nodePath);
    let mut node_name = node_path.file_name().unwrap().to_str().unwrap().to_string();
    node_name.truncate(node_name.len() - 3);

    let re_link = Regex::new(r"\[\[(.+?)\]\]").unwrap();
    let re_tag = Regex::new(r"(?:^|\s)#(\w+)\b").unwrap();
    let mut refs: Vec<String> = Vec::new();

    for link in re_link.captures_iter(&node_content) {
        refs.push(link.get(1).unwrap().as_str().to_string());
    }

    for tag in re_tag.captures_iter(&node_content) {
        refs.push(tag.get(1).unwrap().as_str().to_string());
    }

    conn.execute("DELETE FROM links WHERE source_node = ?1", [&node_name]).expect("Could not delete source node references from links table");
    for link in refs {
        println!("SOURCE: {:?}, TARGET: {:?}", &node_name, &link);
        let sql = r"
            INSERT OR IGNORE INTO nodes (name) VALUES (?2), (?1);
            INSERT INTO links (source_node, target_node) VALUES (?1, ?2);
            ";
        let mut batch = Batch::new(&conn, sql);
        while let Some(mut stmt) = batch.next().unwrap() {
            stmt.execute([&node_name, &link]).unwrap();
        }
    }
}

// returns a vector of strings representing the names of nodes that refer to the given node
#[tauri::command]
pub fn get_node_referred(databaseDir: String, nodeName: String) -> Vec<String> {
    let conn = Connection::open(&databaseDir).expect("Could not open db");
    let mut refs: Vec<String> = Vec::new();

    let mut stmt = conn.prepare("SELECT DISTINCT source_node, target_node FROM links WHERE target_node = ?1").unwrap();
    let ref_iter = stmt.query_map([&nodeName], |row| {
        Ok(Link {
            source_node: row.get(0).unwrap(),
            target_node: row.get(1).unwrap()
        })
    }).expect("Could not execute statement");

    for link in ref_iter {
        refs.push(link.unwrap().source_node);    
    }

    refs
}

#[tauri::command]
pub fn get_source_content(sourceNode: String, cratisDir: String) -> String {
    let mut node_path = cratisDir.clone();
    node_path.push_str(&format!("/nodes/{}.md", &sourceNode));
    let in_nodes = Path::new(&node_path).exists();
    if in_nodes {
        fs::read_to_string(node_path).expect("Could not read node")
    } else {
        let mut journal_path = cratisDir.clone();
        journal_path.push_str(&format!("/journal/{}.md", &sourceNode));
        fs::read_to_string(journal_path).expect("Could not read journal")
    }
}

struct Node {
    name: String
}

#[tauri::command]
pub fn get_nodes(databaseDir: String) -> Vec<String> {
    let conn = Connection::open(&databaseDir).expect("Could not open db");

    let mut stmt = conn.prepare("SELECT name FROM nodes").unwrap();
    let node_iter = stmt.query_map([], |row| {
        Ok(Node { name: row.get(0).unwrap() })
    }).expect("Could not iter query");

    let mut nodes: Vec<String> = Vec::new();

    for node in node_iter {
        nodes.push(node.unwrap().name.to_string());
    }

    nodes
}

#[tauri::command]
pub fn get_references(databaseDir: String) -> Vec<Link> {
    let conn = Connection::open(&databaseDir).expect("Could not open db");

    let mut stmt = conn.prepare("SELECT DISTINCT source_node, target_node FROM links").unwrap();
    let link_iter = stmt.query_map([], |row| {
        Ok(Link {
            source_node: row.get(0).unwrap(),
            target_node: row.get(1).unwrap()
        })
    }).expect("Could not iter query");

    let mut references: Vec<Link> = Vec::new();

    for reference in link_iter {
        references.push(reference.unwrap());
    }

    references
}

#[tauri::command]
pub fn update_properties(databaseDir: String, propertiesObj: &str, nodeName: String) {
    let conn = Connection::open(&databaseDir).expect("Could not open db");

    let properties: HashMap<String, String> = serde_json::from_str(propertiesObj).expect("Could not parse properties json");

    conn.execute("DELETE FROM properties WHERE node = ?1", [&nodeName]).expect("Could not delete node properties");
    for (key, value) in properties.iter() {
        conn.execute("INSERT INTO properties (node, property, value) VALUES (?1, ?2, ?3)", [&nodeName, &key, &value]).expect("Could not insert property");
    }
}
