import { invoke } from "@tauri-apps/api";
import { BaseDirectory, exists, writeTextFile } from "@tauri-apps/api/fs";
import { configDir } from "@tauri-apps/api/path";

async function createDatabase() {
  let dbExists = await exists("database.db", { dir: BaseDirectory.AppConfig })
  if (!dbExists) {
    await writeTextFile("database.db", "", { dir: BaseDirectory.AppConfig })
  }
  let databaseDir = await configDir()
  databaseDir += "/Cratis/database.db"
  await invoke('create_database', { databaseDir })
}

async function indexNodes(nodesDir) {
  let databaseDir = await configDir()
  databaseDir += "/Cratis/database.db"
  await invoke('index_nodes', { databaseDir, nodesDir })
}

async function addNode(nodeName) {
  let databaseDir = await configDir()
  databaseDir += "/Cratis/database.db"
  await invoke('add_node', { databaseDir, nodeName })
}

async function updateReferences(nodePath) {
  let databaseDir = await configDir()
  databaseDir += "/Cratis/database.db"
  await invoke('update_references', { databaseDir, nodePath })
}

async function getNodeReferred(nodeName) {
  let databaseDir = await configDir()
  databaseDir += "/Cratis/database.db"
  return await invoke('get_node_referred', { databaseDir, nodeName })
}

async function getSourceContent(sourceNode, cratisDir) {
  return await invoke('get_source_content', { sourceNode, cratisDir })
}

async function getNodes() {
  let databaseDir = await configDir()
  databaseDir += "/Cratis/database.db"
  return await invoke('get_nodes', { databaseDir })
}

async function getReferences() {
  let databaseDir = await configDir()
  databaseDir += "/Cratis/database.db"
  return await invoke('get_references', { databaseDir })
}

async function updateProperties(properties, nodeName) {
  let databaseDir = await configDir()
  databaseDir += "/Cratis/database.db"
  let propertiesObj = JSON.stringify(properties)
  invoke('update_properties', { databaseDir, propertiesObj, nodeName })
}

export { createDatabase, indexNodes, addNode, updateReferences, getNodeReferred, getSourceContent, getNodes, getReferences, updateProperties }
