# Visual Design
╭─────────────────┬──────────────────────────────────╮
│ Conversations   │ Alice                            │
│─────────────────├──────────────────────────────────┤
│                 │                                  │
│ Alice           │ ╭───────────────────╮            │
│                 │ │░ Hey, how are you?│            │
│ Bob             │ ╰───────────────────╯            │
│                 │            ╭───────────────────╮ │
│ Carol           │            │▓ I'm good, thanks!│ │
│                 │            ╰───────────────────╯ │
│ Dave            │                                  │
│                 │ ╭──────────────────────╮         │
│ Eve             │ │░ Want to grab lunch? │         │
│                 │ ╰──────────────────────╯         │
│                 │                                  │
│                 │            ╭───────────────────╮ │
│                 │            │▓ Sure, let's go!  │ │
│                 │            ╰───────────────────╯ │
│                 │                                  │
│                 ├──────────────────────────────────┤
│                 │ Type a message...                │
╰─────────────────┴──────────────────────────────────╯

# New Conversation Design
╭─────────────────┬──────────────────────────────────╮
│ Conversations   │ Alice                            │
│─────────────────├──────────────────────────────────┤
│                 │                                  │
│ Alice           │ ╭───────────────────╮            │
│                 │ │░ Hey, how are you?│            │
│ Bob             │ ╰───╭────────────────────────╮───╯
│                 │    │   New Conversation      │   │
│ Carol           │    │                         │   │
│                 │    │  Recipient ID:          │   │
│ Dave            │    │  ┌─────────────────┐    │   │
│                 │    │  │                 │    │   │
│ Eve             │    │  └─────────────────┘    │   │
│                 │    │                         │   │
│                 │    │  [Cancel]     [Start]   │   │
│                 │    ╰─────────────────────────╯   │
│                 │                                  │
│                 │                                  │
│                 ├──────────────────────────────────┤
│                 │ Type a message...                │
╰─────────────────┴──────────────────────────────────╯

# Async Design
In order to prevent various events blocking the UI responsvesness, this app spawns future tasks to handle events. There are three tasks for now.

### Main Thread
Main thread runs the application, and the application will be waiting for events to arrive from the central channel. Different sources will clone for their own transmitter to ask for the app to respond to it.

### Inbound Message
Spawining a task to handle the message sent from the websocket server. It takes app transmitter's clone and a websocket receiver. Upon new message arrives, it will forward that message to app's central channel via app transmitter. 

### Terminal Event
Spawning a task to handle the event terminal, will only support keyboard event initially. Similar to Inbound Message, it takes app transmitter's clone to forward the messages to the app.

### Outbound Message
Also spawn a task to send the message to websocket server. It creates its own channel and provide its transmitter to the app for app the send the message to the server. 

# Milestones
* able to connect to the local websocket server when starting the app
* [ctrl + q] to quit the application
* render the two main column as the starting display. left to display an empty conversation list; right to display an empty column.
* upon the selection of conversation list item, right half of the application render the chat display, with recipient id as the title
* when pressing [ctrl + n], display a modal ask for recipient's id.
* [enter] to display the chat window on the right half of the application.
* chat window will display recipient's id as the title, chat history under it, and a text area to enter new messages
* typing will enter message in the message text area
* [enter] to send out the message

