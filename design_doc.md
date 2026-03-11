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
1. whoc should be establishing websocket connection? app itself or the program itself? app itself has dependecy to the channel
Client will have to wait for websocket server response when sending messages out. Because of this, we are designing the terminal UI to have TWO different mpsc channelse. One for handling sending messages to the server, the other one for handling receiving messages from the server.

We will spawn ONE task for the sending receiver, and the websocket receiver to keep waiting for the outbound and inbound messages. Then calling websocket sender to send the message out to the server, and calling receiver sender to send the message to the App.

We are allowing the runtime to decide when to process which tasks and any given momemt, to prevent the blocking of each other.

# To-Study:
┌───────────────────┬─────────────────────────────────────────────────┬─────────────────────────────────────────────────────────┐
│                   │ Two tasks                                       │ Single task + select\!                                   │
├───────────────────┼─────────────────────────────────────────────────┼─────────────────────────────────────────────────────────┤
│ Concurrency model │ True parallelism — tokio can schedule them on   │ Cooperative multiplexing within one task                 │
│                   │ different threads                                │                                                         │
├───────────────────┼─────────────────────────────────────────────────┼─────────────────────────────────────────────────────────┤
│ Readability       │ Each task has a single concern                   │ All logic in one place, but more complex                 │
├───────────────────┼─────────────────────────────────────────────────┼─────────────────────────────────────────────────────────┤
│ Cancellation      │ Independent — reader can outlive writer and     │ Natural — if either side breaks, the loop exits and     │
│                   │ vice versa; need extra signaling (e.g., a       │ both stop                                                │
│                   │ CancellationToken) to coordinate shutdown        │                                                         │
├───────────────────┼─────────────────────────────────────────────────┼─────────────────────────────────────────────────────────┤
│ Performance       │ Negligible difference for this use case          │ Slightly less overhead (one task vs two)                 │
├───────────────────┼─────────────────────────────────────────────────┼─────────────────────────────────────────────────────────┤
│ Error handling    │ Errors are isolated per task                     │ Centralized — one match on which branch fired            │
└───────────────────┴─────────────────────────────────────────────────┴─────────────────────────────────────────────────────────┘

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

