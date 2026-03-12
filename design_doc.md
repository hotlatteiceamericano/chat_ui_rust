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

# Async Flow
1. outbound task: a task to handle message sending to the websocket server
1. inbound task: a task handle message receiving from the websokcet server
1. keyboard event: a task to listen for user key presses
1. question: should I put the central channel in a task?

my plan:
1. create an app channel, which of course app will receive app_rx
1. create a keyboard task, takes an app_tx clone
1. create a websocket inbound task, takes an app_tx clone to update the conversation list and chat window
1. create a websokcet outbound task, with a ountbound rx. provide an outbound tx's clone to the app

requirements:
app:
1. no task, use main thread
1. takes app_rx
1. takes outbound_tx

inbound task:
1. takes app_tx clone
1. takes ws_receiver

outbound task:
1. takes outbound_rx
1. takes ws_sender

terminal task:
1. takes app_tx clone

summary:
1. inbound messages do not need a channel, but need a task (uses app tx clone instead)
1. terminal events do not need a channel, but need a task (uses app tx clone)
1. outbound messages need both a channel and a task
1. app needs a channel and uses main thread

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

