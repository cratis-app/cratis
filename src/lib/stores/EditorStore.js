import { writable } from 'svelte/store'
import { invoke } from '@tauri-apps/api/tauri';
import toml from 'toml'

const editor = writable({
  activeNode: "",
  activeFragment: 0,
  nodePath: "",
  isJournal: true,
  showJournal: true,
  showEditor: true,
  frontmatter: {}
});

let openNode = async (nodePath, nodeName) => {
  let nodeStr;
  if (nodeName !== "") {
    nodeStr = await invoke('open_node', { nodePath })  
  }

  let frontmatter = {}
  if (nodeStr.substring(0, 3) === "+++") {
    let splitStr = nodeStr.split("+++")
    let frontmatterStr = splitStr[1]
    nodeStr = splitStr[2]
    frontmatter = toml.parse(frontmatterStr)
  }

  return {
    frontmatter: frontmatter,
    content: nodeStr
  }
}

export { editor, openNode }
