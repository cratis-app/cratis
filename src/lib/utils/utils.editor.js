import { invoke } from '@tauri-apps/api'
import { updateReferences } from './utils.database.js'

async function convertMarkdown(content) {
  return await invoke('parse_md', { content })
}

async function parseContent(line) {
  let splitArr = line.split('-')
  let level = 0
  if (splitArr.length > 1) {
    level = splitArr[0].length/4
  }
  return {
    level: level,
    content: line.substring(line.indexOf("-")+2, line.length),
  }
}

async function saveNode(fragments, nodePath) {
  console.log(nodePath)
  //write fragments to file with bullet at appropriate level
  let nodeStr = ""
  fragments.forEach(frag => {
    if (frag.content !== "")
      nodeStr += "    ".repeat(frag.level) + "- " + frag.content + "\n"
  })
  
  await invoke('save_node', { nodeStr, nodePath })
  updateReferences(nodePath)
}

async function searchNodes(searchVal, cratisDir) {
  let nodes = await invoke('search_nodes', { searchVal, cratisDir })
  // sort matching nodes by closeness to search value and return top 10
  let filteredNodes = nodes
    .filter((item) => item.toLowerCase().includes(searchVal.toLowerCase()))
    .sort((a, b) => {
      let key = searchVal.toLowerCase()
      let isGoodMatchA = a.toLowerCase().startsWith(key)
      let isGoodMatchB = b.toLowerCase().startsWith(key)

      if (isGoodMatchA ^ isGoodMatchB) {
        return isGoodMatchA ? -1: 1 
      }

      return a.localeCompare(b)
    })

  return filteredNodes.slice(0, 9)
}

function isDateFormat(string) {
  const regex = /^\d{4}-\d{2}-\d{2}$/ 

  return regex.test(string)
}

async function addAttachment(data) {
  console.log(data)
  let extension = data.split(';')[0].split('/')[1]
  console.log(extension)
  return ""
}

export { parseContent, convertMarkdown, saveNode, searchNodes, isDateFormat, addAttachment }
