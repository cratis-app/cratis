<script>
  import { createNode } from "../../utils/utils.network";
  import { editor } from "../../stores/EditorStore";
  import { user } from "../../stores/UserStore";
  import { isDateFormat, addAttachment } from '../../utils/utils.editor.js'
  import { searchNodes } from "../../utils/utils.editor.js";
  import { convertMarkdown } from '../../utils/utils.editor.js'
  import { tick } from 'svelte'
  export let key
  export let level
  export let content
  export let active
  export let fragments
  export let dragging
  export let saveNode
  let hoverHandle
  let linkSearch = false
  let searchResults = []
  let activeNode = 0
  let tagSearch
  let caretOffset

  let fragContent
  let actionKeys = [
    "ArrowUp",
    "ArrowDown",
    "Tab"
  ]

  let pairKeys = [
    { key: "[", pair: "]" },
    { key: "(", pair: ")" },
    { key: "{", pair: "}"}
  ]

  let getFragContent = async() => {
    fragContent = active ? content : await convertMarkdown(content, $user.config.network_config.location + "/" + $user.config.network_config.name + "/attachments/")
    await tick()
    let links = document.querySelectorAll('.nodeLink')
    Array.from(links).forEach((el) => {
      el.addEventListener('click', handleOpenNode)
    })
    document.getElementsByClassName("active")[0]?.focus()

    if (isInLink() && active) {
      getCaretOffset()
      linkSearch = true 
      handleSearchNodes()
    } else {
      linkSearch = false
    }

    if (extractTag(fragments[key].content) && active) {
      getCaretOffset()
      tagSearch = extractTag(fragments[key].content)
      handleSearchTags()
    } else {
      tagSearch = ""
    }

  }

  let extractTag = (str) => {
    let activeEl = document.getElementsByClassName('active')[0]
    let caretPos = 0
    if (activeEl)
      caretPos = getCaretPos(activeEl).position
    let lastTagIndex = str.lastIndexOf('#', caretPos - 1)
    if (lastTagIndex === -1) return null
    let nextWhitespaceIndex = str.indexOf(' ', lastTagIndex)
    if (nextWhitespaceIndex === -1) {
      let tag = str.slice(lastTagIndex + 1)
      return tag.includes('#') ? null : tag
    }
    if (nextWhitespaceIndex < caretPos) return null
    let tag = str.slice(lastTagIndex + 1, nextWhitespaceIndex)
    return tag.includes('#') ? null : tag
  }

  let handleOpenNode = (e) => {
    let path
    if (isDateFormat(e.target.attributes[0].nodeValue)) {
      path = $user.config.network_config.location + '/' + $user.config.network_config.name + '/journal/'
    } else {
      path = $user.config.network_config.location + '/' + $user.config.network_config.name + '/nodes/'
    }
    createNode(path, e.target.attributes[0].nodeValue)
    $editor = { 
      ...$editor, 
      activeNode: e.target.attributes[0].nodeValue, 
      nodePath: path + e.target.attributes[0].nodeValue + '.md', 
      isJournal: isDateFormat(e.target.attributes[0].nodeValue),
      showJournal: false
    }
  }

  let handleSearchTags = async () => { 
    searchResults = await searchNodes(tagSearch, $user.config.network_config.location + '/' + $user.config.network_config.name)
    if (searchResults[0] !== tagSearch)
      searchResults.splice(0, 0, tagSearch)
  }
  
  let getSearchVal = (caretPos, content) => {
    let openingIndex = content.lastIndexOf("[[", caretPos)
    if (openingIndex === -1) linkSearch = false
    let closingIndex = content.indexOf("]]", caretPos)
    if (closingIndex === -1) linkSearch = false
    if (closingIndex < openingIndex) linkSearch = false
    return content.substring(openingIndex + 2, closingIndex).trim()
  }

  let handleSearchNodes = async () => {
    let caretPos = getCaretPos(document.getElementsByClassName('active')[0]).position
    let searchLinkVal = getSearchVal(caretPos, fragContent)
    if (searchLinkVal) {
      searchResults = await searchNodes(searchLinkVal, $user.config.network_config.location + '/' + $user.config.network_config.name)
    }
    if (searchResults[0] !== searchLinkVal) { 
      searchResults.splice(0, 0, searchLinkVal)
    }

  }

  let handlePairing = async (keyPress, e) => {
    let caretPos = getCaretPos(document.getElementsByClassName('active')[0])
    fragments[key].content = fragContent.substring(0, caretPos.position) + pairKeys.find(p => p.key === keyPress).pair + fragContent.substring(caretPos.position)
    console.log(fragments[key].content)
    fragments = fragments
    await tick()
    setCaretPos(caretPos.position)
  }

  $: fragments, getFragContent()
  // $: fragments, saveNode(fragments)

  let isInLink = () => {
    let activeEl = document.getElementsByClassName('active')[0]
    let caretPos = { position: 0 }
    if (activeEl)
      caretPos = getCaretPos(activeEl)
    let linkReg = /\[\[(.*?)\]\]/g 
    let match
    while ((match = linkReg.exec(fragments[key].content)) !== null) {
      let linkStart = match.index
      let linkEnd = linkStart + match[0].length
      if (linkStart <= caretPos.position && linkEnd >= caretPos.position) return true 
    }
    return false
  }

  let handleKeydown = async (e) => {
    if (extractTag(fragments[key].content)) {
      tagSearch = extractTag(fragments[key].content)
      handleSearchTags()
    } else {
      tagSearch = ""
    }

    if (isInLink()) {
      linkSearch = true 
      handleSearchNodes()
    } else {
      linkSearch = false 
    }

    if (actionKeys.includes(e.code))
      e.preventDefault()
    
    // handle action keys 
    if (e.key === "Enter" && !e.shiftKey && !linkSearch && tagSearch === "") {
      e.preventDefault()
      let caretInfo = getCaretPos(document.getElementsByClassName('active')[0])
      if (caretInfo.position === caretInfo.length && caretInfo.length > 0) {
         let newFrag = {
          key: key + 1,
          level: level,
          content: '',
          active: true
        }
        fragments.splice(key + 1, 0, newFrag)
        fragments[key].active = false
        let i = key + 2 
        for (i; i < fragments.length; i++) {
          fragments[i].key++
        }
        fragments = fragments
      } else {
        if (caretInfo.length > 0) {
          // create new fragment with content following caret
          let elContent = getCaretContent(caretInfo.position)
          let newFrag = {
            key: key + 1,
            level: level,
            content: elContent.afterCaret,
            active: true 
          }
          fragments.splice(key + 1, 0, newFrag)
          fragments[key].active = false 
          fragments[key].content = fragments[key].content.slice(0, caretInfo.position)
          let i = key + 2
          for (i; i < fragments.length; i++) {
            fragments[i].key++
          }
          fragments = fragments
          await tick()
          setCaretPos(0)
        }
      }
    }

    if (e.key === "Backspace") {
      let caretPos = getCaretPos(document.getElementsByClassName('active')[0])
      let prevContent = fragments[key-1].content 
      if (content === "") {
        e.preventDefault()
        fragments.splice(key, 1)
        fragments[key-1].active = true
        let i = key
        for (i; i < fragments.length; i++) {
          fragments[i].key--
        }
      } 
      else if (caretPos.position === 0) {
        e.preventDefault()
        fragments[key-1].content += ' ' + fragments[key].content 
        fragments.splice(key, 1)
        let i = key
        for (i; i < fragments.length; i++) {
          fragments[i].key--
        }
        fragments[key-1].active = true
      }
      fragments = fragments
      await tick()
     
      if (content === "") {
        setCaretPos(prevContent.length)
      } else if (caretPos.position === 0) {
        setCaretPos(prevContent.length + 1)
      }
    }

    if (e.key === "ArrowUp" && (!linkSearch && tagSearch === "")) {
      fragments[key-1].active = true
      fragments[key].active = false
      fragments = fragments 
      await tick()
      setCaretPos(fragments[key-1].content.length)
    }
    if (e.key === "ArrowDown" && (!linkSearch && tagSearch === "")) {
      fragments[key+1].active = true
      fragments[key].active = false
      fragments = fragments
      await tick()
      setCaretPos(fragments[key+1].content.length)
    }

    if (e.key === "Enter" && (linkSearch || tagSearch !== "")) {
      e.preventDefault()
      handleAutoComplete(searchResults[activeNode])
    }

    console.log(activeNode)
    if (e.key === "ArrowUp" && (linkSearch || tagSearch !== "")) {
      if (activeNode > 0)
        activeNode--
    }

    if (e.key === "ArrowDown" && (linkSearch || tagSearch !== "")) {
      if (activeNode < searchResults.length - 1)
        activeNode++
    }

    if (e.code === "Tab") {
      let caretPos = getCaretPos(document.getElementsByClassName('active')[0])

      if (!e.shiftKey) {
        if (fragments[key - 1].level >= fragments[key].level)
          fragments[key].level++
      } else {
        if (fragments[key].level > 0) {
          fragments[key].active = true
          fragments[key].level--
        }
      }
      fragments = fragments
      await tick()
      setCaretPos(caretPos.position)
    }
  }

  let getCaretContent = (caretPos) => {
    let sel = window.getSelection()
    if (sel.rangeCount !== 0) {
      let content = window.getSelection().getRangeAt(0).endContainer.textContent
      let beforeCaret = content.substring(0, caretPos)
      let afterCaret = content.substring(caretPos, content.length)

      return {
        content: content,
        beforeCaret: beforeCaret,
        afterCaret: afterCaret
      }
    }
  }

  let getCaretPos = (el) => {
    let position = 0
    let length = 0
    if (window.getSelection) {
      let selection = window.getSelection()
      if (selection.rangeCount > 0) {
        let range = selection.getRangeAt(0)
        let preCaretRange = range.cloneRange()
        preCaretRange.selectNodeContents(el)
        preCaretRange.setEnd(range.endContainer, range.endOffset)
        position = preCaretRange.toString().length
        length = range.endContainer.textContent.length
      }
    } else if (document.selection && document.selection.type !== 'Control') {
      let textRange = document.selection.createRange()
      let preCaretTextRange = document.body.createTextRange()
      preCaretTextRange.moveToElementText(el)
      preCaretTextRange.setEndPoint('EndToEnd', textRange)
      position = preCaretTextRange.text.length - textRange.text.length
      length = textRange.text.length 
    }

    return {
      position: position,
      length: length
    }
  }

  let setCaretPos = (index) => {
    let el = document.getElementsByClassName('active')[0]
    let sel = window.getSelection()
    let range = document.createRange()
    range.setStart(el.childNodes[0], index)
    range.collapse(true)
    sel.removeAllRanges()
    sel.addRange(range)
  }

  let handleInput = async (e) => {
    if (document.getElementsByClassName('active')[0]) {
      let caretPos = getCaretPos(document.getElementsByClassName('active')[0])
      fragments[key].content = cleanString(e.target.textContent)
      fragments = fragments
      await tick()
      setCaretPos(caretPos.position)
      saveNode(fragments)
    }
    if (pairKeys.find(p => p.key === e.data)) {
      handlePairing(e.data, e)
    }
  }

  let cleanString = (string) => {
    let output = ""
    for (let i=0; i<string.length; i++) {
      if (string.charCodeAt(i) <= 127 || string.charCodeAt(i >= 160) && input.charCodeAt(i) <= 255) {
        output += string.charAt(i)
      }
    }
    return output
  }

  let handleClick = async (type, e) => {
    if (type === "focus") {
      fragments[key].active = true
      fragments = fragments
      let pos = getCaretPos(e.target)
      await tick()
      setCaretPos(pos.position)
    }
    if (type === "blur") {
      fragments[key].active = false
      fragments = fragments 
    }
  }

  let handleDragStart = (e) => {
    // save source element 
    e.dataTransfer.setData('text/plain', e.target.getAttribute('key'))
    dragging = true
  }
  
  let handleDragEnter = (e) => {
  }

  let handleDragEnd = (e) => {
    dragging = false
  }

  let handleDragOver = (e) => {
    return false
  }

  let handleDrop = (e) => {
    // clone drop fragment and move down 1
    let targetKey = parseInt(e.currentTarget.getAttribute('key'))
    let oldKey = parseInt(e.dataTransfer.getData('text'))
    let cloneFrag = {
      key: targetKey + 1,
      level: fragments[targetKey].level,
      content: fragments[targetKey].content,
      active: false 
    }
    fragments.splice(targetKey + 1, 0, cloneFrag)
    let i = targetKey + 2
    for (i; i < fragments.length; i++) {
      fragments[i].key++
    }

    if (targetKey < oldKey) {
     // replace drop fragment with drag fragment
      let oldFrag = {
        key: targetKey,
        level: fragments[targetKey+1].level,
        content: fragments[oldKey+1].content,
        active: true
      }
      fragments[targetKey] = oldFrag
      // delete drag fragment
      fragments.splice(oldKey+1, 1)
      i = oldKey+1
      for (i; i < fragments.length; i++) {
        fragments[i].key--
      }
    } else {
      let dragFrag = {
        key: targetKey,
        level: fragments[targetKey+1].level,
        content: fragments[oldKey].content,
        active: true 
      }
      fragments[targetKey] = dragFrag
      fragments.splice(oldKey, 1)
      i = oldKey
      for (i; i < fragments.length; i++) {
        fragments[i].key--
      }
    }
    fragments = fragments
    saveNode(fragments)
  }
  
  let handleAutoComplete = async (node) => {
    let caretPos = getCaretPos(document.getElementsByClassName('active')[0]).position 
    if (linkSearch) {
      let openingIndex = fragContent.lastIndexOf('[[', caretPos)
      let closingIndex = fragContent.indexOf(']]', caretPos)
      fragments[key].content = fragContent.substring(0, openingIndex + 2) + node + fragContent.substring(closingIndex)
      linkSearch = false
      fragments = fragments
      await tick()
      setCaretPos(openingIndex + 2 + node.length + 2)
    } else {
      let tagIndex = fragContent.lastIndexOf('#', caretPos)
      fragments[key].content = fragContent.substring(0, tagIndex + 1) + node + fragContent.substring(tagIndex + node.length, fragContent.length)
      tagSearch = ""
      fragments = fragments
      await tick()
      setCaretPos(tagIndex + node.length + 1)
      saveNode(fragments)
    }
  }

  let getCaretOffset = () => {
    let caretPos = getCaretPos(document.getElementsByClassName('active')[0]).position
    let measureSpan = document.createElement("span")
    measureSpan.setAttribute("id", "measureSpan")
    let content = fragContent?.substring(0, caretPos)
    measureSpan.textContent = content
    document.body.appendChild(measureSpan)
    caretOffset = measureSpan.getBoundingClientRect().width
    measureSpan.remove()
  }

  let handlePaste = async (e) => {
    if (e.clipboardData.files[0]) {
      let clipData = e.clipboardData.files[0]
      let validImageTypes = ['image/gif', 'image/jpeg', 'image/png']
      if (validImageTypes.includes(clipData.type)) {
        e.preventDefault()
        let reader = new FileReader()
        reader.onload = async function() {
          let attachment = addAttachment(reader.result, $user.config.network_config.location + "/" + $user.config.network_config.name)
          let caretPos = getCaretPos(document.getElementsByClassName('active')[0]).position
          let attachmentLink = `![${attachment.name}](${attachment.path})`
          fragments[key].content = fragContent.substring(0, caretPos) + attachmentLink + fragContent.substring(caretPos + attachmentLink.length) 
          fragments = fragments
          await tick()
          setCaretPos(caretPos + attachmentLink.length)
          saveNode(fragments)
        }
        reader.readAsDataURL(clipData)
      }
    }
  }

</script>

{#if fragContent !== undefined}
  <div 
    id="frag-{key}" 
    key={key}
    class={`fragment ${dragging ? 'dragging' : ''}`}
    on:dragover|preventDefault={handleDragOver}
    on:dragenter|preventDefault={handleDragEnter}
    on:drop={handleDrop}
  >
    <div 
      class={`frag-drag ${hoverHandle ? 'draggable' : ''}`}
      key={key}
      style={`margin-left:calc(1em*${level})`}
      on:dragstart={handleDragStart}
      on:dragend={handleDragEnd}
      draggable="true"
    >
      <div 
        class="handle" 
        on:mouseenter={() => hoverHandle = true}
        on:mouseleave={() => hoverHandle = false}
      >•</div>
      <div
        class={`content ${active ? 'active' : ''}`}
        style={`width:calc(100% - 1em * ${level})`}
        on:click={(e) => handleClick("focus", e)}
        on:blur={(e) => handleClick("blur", e)}
        on:input={handleInput}
        on:keydown={handleKeydown}
        on:paste={handlePaste}
        contenteditable
      >
        {@html fragContent}
      </div>
      {#if linkSearch || tagSearch !== ""}
        <ul id="searchLinkResults" style={`left:${caretOffset}px`}>
          {#each searchResults as node, i}
            <li
              key={node}
              on:click={(e) => handleAutoComplete(e.target.attribute[0].nodeValue)}
              on:keydown|preventDefault
              class={activeNode === i ? 'activeNode' : ''}
              on:mouseover={() => activeNode = i}
            >
              {node}
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  </div>
{/if}
<style lang="scss">
  .frag-drag {
    pointer-events: none;
    display: inline-grid;
    position: relative;
    grid-template-areas: "handle content";
    grid-template-columns: 1em auto;
    width: 100%;

    .handle {
      grid-area: handle;
      align-self: center;
      padding: 0.1em;
      margin-right: 0.5em;
      pointer-events: auto;
    }

    .content {
      grid-area: content;
      display: inline;
      pointer-events: auto;
    }

    #searchLinkResults {
      list-style-type: none;
      padding: 0.5em;
      position: absolute;
      top: 1em;
      background-color: var(--select-bg);

      .activeNode {
        background-color: var(--bg-color);
      }
    }
  }
  :global {
    .content > p {
      display: inline;
      margin: 0;
    }
  }
  .dragging {
    
  }

  .draggable {
    cursor: move;
  }
</style>
