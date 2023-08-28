# Cratis

<img src="https://cdn.josephtalon.ca/images/Cratis/icon.png" width="100px" />

Cratis is a bi-directional outliner built using Tauri and SvelteKit.
You can create notes using markdown, link to other notes, and view all your notes and their connections in a network graph.

## Downloads
[Linux (AppImage)](https://github.com/cratis-app/cratis/releases/download/v0.1.1/cratis_0.1.1_amd64.AppImage)

[Debian](https://github.com/cratis-app/cratis/releases/download/v0.1.1/cratis_0.1.1_amd64.deb)

[Windows](https://github.com/cratis-app/cratis/releases/download/v0.1.1/Cratis_0.1.1_x64_en-US.msi)

## Features

### Outliner capability

Cratis is an outliner. An outliner is just a specialized type of text editor that is used to create/edit outlines, which are text files which follow a tree structure for organization. Each line is rendered as a bullet point that can be nested for structure.

### Bi-directional links

Cratis allows you to use bi-directional links to connect your notes! Type the name of another note between double square brackets or following a hashtag.

Ex/
- ```[[Another note]]```
- ```#Testing```

### Restructuring

Indent your current line by pressing ```Tab``` or shift it back a level with ```Shift+Tab```.

Re-order your points with drag and drop.

### Daily Journal ![Journal](https://cdn.josephtalon.ca/images/Cratis/calendar.svg)

Cratis automatically creates a daily note that goes into your "journal" every day you open the app. From here, you can insert any thoughts you have throughout the day and start building your network.

### View all notes and connections ![Network](https://cdn.josephtalon.ca/images/Cratis/network.svg)

View and navigate all your notes and their connections with the network view.

### See all references to current note

View all references to the current node. This allows you to start building a note from other notes.

### Saved Locally as .md

Every file in the network is saved locally using [markdown](https://github.github.com/gfm/) format.

## Demo Video

Found on YouTube ![here](https://www.youtube.com/watch?v=HvM7Fqb5boQ)

[![Watch the demo](https://img.youtube.com/vi/HvM7Fqb5boQ/maxresdefault.jpg)](https://www.youtube.com/watch?v=HvM7Fqb5boQ)

## Demo Network

Use the demo network to see what Cratis is capable of and view a tutorial on how to use it.

Download the network [here](https://github.com/cratis-app/cratis/releases/download/v0.1.1/demo_network.zip).

### Open the network

1. Unzip the folder to your desired network location
2. Launch Cratis
3. Select the location you unzipped the demo network to (but not the network itself "My Notes")
4. Set the network name to "My Notes" and click the "Create Network" button

![Create Network View](https://cdn.josephtalon.ca/images/Cratis/Craits_create_network.png)
