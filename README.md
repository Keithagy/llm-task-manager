# housekeeping

## What are we doing here?

This is a server fronting a locally-running ollama instance, and a collection of databases, which provides http endpoint-based interaction with the ollama interaction that augments inputs to the model with relevant retrievals as well as any cookie-attachable context.

Subsequent sections will deal with specific systems and functionality to be added to the base ollama instance.

## Task logging system

- Use telegram for voice-based (with transcription) natural interaction for reading and writing to task logging system
- Interactions can be text based or voice based (text-based first)
- Telegram bot reads in and writes through interactions from ollama instance (telegram here is just a publisher instance, really. How would we handle the authentication aspect, though? Maybe just a simple password for now)
- Possible actions to execute:
  - Retrieve tasks
  - Create tasks
  - Delete tasks
  - Edit task properties
- Task properties:
  - UUID
  - Description
  - Creation date
  - Due date
  - Assignee (string; In the future, this can integrate with the room occupancy system to only allow current crew members)
  - Completion status (enum)
  - Soft delete flag
- Prompt chain:
  1. Incoming message via telegram
  2. Check for password
  3. Extract C/R/U/D message intent
  4. Determine arguments to supply for C/R/U/D function call
  5. If calling by some parameters (e.g. retrieve uncompleted tasks for some crew member) is unsupported, then resolve that way and terminate the flow accordingly.
  6. Call function, pass return value to LLM to tailor answer

## Room Occupancy system

