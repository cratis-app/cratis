<script>
  import { getSourceContent } from '../../utils/utils.database.js' 
  import { parseContent, isDateFormat, convertMarkdown } from '../../utils/utils.editor.js';
  import { createNode } from '../../utils/utils.network.js'
  import { user } from '../../stores/UserStore.js';
  import { editor } from '../../stores/EditorStore.js';
  export let sourceNode
  export let targetNode
  let refFragments = []
  let filteredFrags = []

  let getRefFragments = async () => {
    let sourceContent = await getSourceContent(sourceNode, $user.config.network_config.location + "/" + $user.config.network_config.name)
    let lines = sourceContent.split(/\r?\n/).filter(l => l !== "")
    refFragments = new Array(lines.length)

    for (let i=0; i<lines.length; i++) {
      let parsedContent = await parseContent(lines[i])
      let htmlContent = await convertMarkdown(parsedContent.content, $user.config.network_config.location + '/' + $user.config.network_config.name + '/attachments/')
      refFragments[i] = {
        key: i,
        level: parsedContent.level,
        content: htmlContent,
        active: false 
      }
      refFragments = refFragments
    }

    // remove non-referencing fragments
    filteredFrags = filterFragments(refFragments)   
  }

  let filterFragments = (fragments) => {
    let filteredFrags = []
    let refLevel = 0
    let refKey = -1
    for (let frag of fragments) {
      if (frag.level <= refLevel) {
        refKey = -1
      }

      if (refKey === -1) {
        let linkRegex = new RegExp(`(?<=\\[\\[)${targetNode}(?=\\]\\])`, "g")
        let tagRegex = new RegExp(`(?<=\\B#)${targetNode}(?=\\b)`, "g")

        let linkMatch = frag.content.match(linkRegex)
        let tagMatch = frag.content.match(tagRegex)

        if (linkMatch || tagMatch) {
          let match
          if (linkMatch) match = linkMatch
          if (tagMatch) match = tagMatch
          if (match[0] === targetNode) {
            filteredFrags.push(frag)
            refLevel = frag.level
            refKey = frag.key
          }
        }
      } else {
        filteredFrags.push(frag)
      }
    }
    return filteredFrags
  }

  let addEvents = () => {
    let links = document.querySelectorAll('.nodeLink')
    Array.from(links).forEach((el) => {
      el.addEventListener('click', handleOpenNode)
    })
  }

  let handleOpenNode = (e) => {
    if (isDateFormat(e.target.attributes[0].nodeValue)) {
      createNode($user.config.network_config.location + '/' + $user.config.network_config.name + '/journal/', e.target.attributes[0].nodeValue)
    } else {
      createNode($user.config.network_config.location + '/' + $user.config.network_config.name + '/nodes/', e.target.attributes[0].nodeValue)
    }

    let nodePath 
    if (isDateFormat(e.target.attributes[0].nodeValue)) {
      nodePath = $user.config.network_config.location + '/' + $user.config.network_config.name + '/journal/' + e.target.attributes[0].nodeValue + '.md'
    } else {
      nodePath = $user.config.network_config.location + '/' + $user.config.network_config.name + '/nodes/' + e.target.attributes[0].nodeValue + '.md'
    }

    $editor = {
      ...$editor,
      activeNode: e.target.attributes[0].nodeValue,
      nodePath: nodePath,
      isJournal: isDateFormat(e.target.attributes[0].nodeValue),
      showJournal: false
    }
  }

  $: $editor, getRefFragments() 
  $: $editor, addEvents()
</script>

<div id="refSource">
  <button key={sourceNode} class="nodeLink">{sourceNode}</button> 
  {#each filteredFrags as frag (frag.key)}
    <div
      class="fragContainer"
      style={`margin-left:calc(1em + 1em*${frag.level});width:calc(100% - 1em*${frag.level} - 1em)`}
    >
      <div class="handle">•</div>
      <div class="content">{@html frag.content}</div>
    </div> 
  {/each}
</div>

<style lang="scss">
  #refSource {
    pointer-events: auto;
    width: 100%;

    .fragContainer {
      display: inline-grid;
      grid-template-areas: "handle content";
      grid-template-columns: 1em auto;
      margin-top: 0.5em;

      .handle {
        grid-area: handle;
        align-self: center;
        padding: 0.1em;
        margin-right: 0.5em;
      }

      .content {
        grid-area: content;
        display: inline;
        pointer-events: auto;
      }
    }
  }
</style>
