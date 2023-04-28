<script>
  import { saveProperties } from "../../utils/utils.editor";
  import { updateProperties } from "../../utils/utils.database"

  export let nodePath
  export let nodeName
  export let properties = {}
  let showProperties = false

  let handleSaveProperties = () => {
    let table = document.getElementsByClassName("property-list")[0]
    let obj = {}
    let rows = table.getElementsByTagName('tr')
    for (let i = 0; i < rows.length; i++) {
      let cells = rows[i].getElementsByTagName('td')
      let key = cells[0].getElementsByTagName('input')[0].value
      let value = cells[1].getElementsByTagName('input')[0].value
      obj[key] = value
    }
    properties = obj
    saveProperties(obj, nodePath)
    updateProperties(obj, nodeName)
  }

  let handleKeydown = (e) => {
    if (e.code === "Enter") {
      properties[""] = ""
    }
    if (e.code === "Backspace") {
      if (e.srcElement.classList.contains("prop-input") && e.srcElement.value === "") {
        e.preventDefault()
        delete properties[""]
        properties = properties
        let props = document.querySelectorAll(".input")
        props[props.length - 3].focus()
      }
      if (e.srcElement.classList.contains("val-input") && e.srcElement.value === "") {
        e.preventDefault()
        let props = document.querySelectorAll(".input")
        for (let i = 0; i < props.length; i++) {
          if (props[i+1] === e.srcElement) {
            props[i].focus()
          }
        }
      }
    }
  }

  $: if (Object.keys(properties).length === 0) {
    properties[""] = ""
  }
</script>

<div id="properties">
  <button id="btn-properties" on:click={() => showProperties = !showProperties}>{Object.keys(properties).some(key => key === "") ? "0" : Object.keys(properties).length} Properties {showProperties ? "▲" : "▼"}</button>
  {#if showProperties}
    <table class="property-list">
      {#each Object.entries(properties) as [property, value]}
        <tr>
          <td class="prop"><div><input autofocus class="prop-input input" value={property} on:keydown={handleKeydown} on:input={handleSaveProperties} /></div></td>
          <td class="value"><div><input class="val-input input" value={value} on:keydown={handleKeydown} on:input={handleSaveProperties} /></div></td>
        </tr> 
      {/each}
    </table>
  {/if}
</div>

<style lang="scss">
  #properties {
    margin-bottom: 1em;

    #btn-properties {
      background: none;
      border: none;
      color: var(--link-color);
    }

    .property-list {
      margin-left: 1em;
      table-layout: fixed;
      background-color: var(--secondary-bg);
      padding: 1em;
      border-collapse: collapse;
      width: 100%;

      tr {
        td {
          padding: 0.5em;
          border: none;

          input {
            width: calc(100% - 0.5em);
            background: none;
            border: none;
            padding: 0.5em;
          }
        }
        .prop {
          color: #595959;
          width: 75px;

          input {
            color: inherit;
          }
        }
        
        border-bottom: 1px solid black;
      }

      tr:nth-child(even) {
        background-color: var(--bg-color);
      }
    }
  }
</style>
