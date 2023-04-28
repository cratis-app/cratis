<script>
  import { tick } from "svelte";
  import Fragment from "./Fragment.svelte";
  import { editor } from "../../stores/EditorStore";
  import { user } from "../../stores/UserStore"
  import { getNodeReferred, updateReferences } from "../../utils/utils.database.js"
  import { parseContent, saveNode } from "../../utils/utils.editor.js"
  import Reference from "./Reference.svelte";
  import Properties from "./Properties.svelte";

  export let properties = {}
  export let lines
  let references = []
  let fragments = []
  let dragging = false
  export let nodeName
  let nodePath = $editor.isJournal ? $user.config.network_config.location + '/' + $user.config.network_config.name + '/journal/' + nodeName + '.md' : $editor.nodePath

  $: $editor, nodePath = $editor.isJournal ? $user.config.network_config.location + '/' + $user.config.network_config.name + '/journal/' + nodeName + '.md' : $editor.nodePath
  $: lines, lineToFragment()

  let lineToFragment = async () => {
    fragments = new Array(lines.length)
    for (let i = 0;i < lines.length; i++) {
      let parsedContent = await parseContent(lines[i])
      fragments[i] = {
        key: i,
        level: parsedContent.level,
        content: parsedContent.content,
        active: false
      }
      fragments = fragments
    }
  }

  let getReferences = async () => {
    references = []
    references = await getNodeReferred(nodeName)
  }

  $: $editor, getReferences()

</script>

<section id="node">
  <h1>{nodeName}</h1>
  {#if !$editor.showJournal}
    <Properties properties={properties} nodePath={nodePath} nodeName={nodeName} /> 
  {/if}
  {#if fragments[fragments.length - 1]}
    {#each fragments as fragment}
      <div key={fragment.key}>
        <Fragment 
          key={fragment.key} 
          level={fragment.level}
          content={fragment.content}
          active={fragment.active}
          bind:fragments={fragments}
          bind:dragging={dragging}
          saveNode={(fragments) => {saveNode(fragments, nodePath)}}
        />
      </div>
    {/each}
  {/if}
  <section id="references">
    {#if references.length > 0}
      <strong>References</strong>
    {/if}
    {#each references as ref}
      <div>
        <Reference 
          sourceNode={ref}
          targetNode={nodeName}
        />
      </div>
    {/each}

  </section>
</section>

<style lang="scss">
  #node {

    h1 {
      margin-bottom: 16px;
    }

    div {
      pointer-events: none;
      margin-bottom: 0.5em;
      width: 100%;
    }

    #references {
      margin-top: 2em;

      strong {
        font-size: 1.2em;
      }

      div {
        background-color: var(--secondary-bg);
        border-radius: 0.5em;
        margin-top: 1em;
        padding: 0.8em;
      }
    } 
  }
</style>
