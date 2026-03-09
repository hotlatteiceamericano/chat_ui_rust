# Design
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

