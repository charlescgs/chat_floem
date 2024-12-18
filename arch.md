## Main views:
1. Side room
2. Msg
3. Editor

## Main view containers:
1. Rooms list
2. Msgs list
3. Editor panel

------------
# Main concepts:
- UI runs on it's own thread and communicates with backend via channels/signals from channels
- UI state/session is a global static from the thread local
- Implement main command system

------------
## Updates flow
Update comes from the server to backend -> to frontend([UISession]) -> notif -> views fetch the data

------------------------------------------

## Chunks system example
| Chunks msgs | Operation | Display msgs |
1. New Msgs:
# # # # # #
2. Clone to display:
# # # # # #     clone->     # # # # # #      
3. Delete last msg:
# # # # #        del->        # # # # # 
4. Tab changed:
# # # # # #      hide-> 
5. Tab active again:
# # # # # #   calc_shown->    - - # # #
6. Load more:
# # # # # #    unhide->       # # # # #


## Chunks cases:
1. Add msg:
    - Chunk < 20: append
    - Chunk > 20: create next Chunk
2. Add msgs:
    - Chunk remaining len < msgs len: fill available, rest append on a new Chunk
    - Chunk remaining len >= msgs len: append
3. Edit msg:
    - Find Chunk, find msg: edit
4. Delete msg:
    - Find Chunk, find msg: if Chunk msg count is 1, remove Chunk, else remove msg

## Display cases:
1. Append msg:
    - fetch last msg from chunks
2. Append msgs:
    - fetch msgs younger than last-on-display from Chunks
3. Msg edited:
    - fetch edited msg and replace it
4. Msg deleted:
    - remove deleted msg
5. Hide older msgs:
    - hide msgs older than 20
6. Load older msgs:
    - fetch another full Chunk

### Chunks & Display implementation concepts:
- [Chunks] is a struct holding vec of [Chunk] with metadata
- [Chunk] is struct holding up to 20 msgs in vec with metadata
- [Display] is:
    Option 1: [Vector] holding msgs
    Option 2: Struct with focus-like capabilites holding msgs and implementing `IntoIterator` trait
    Option 3: [BTreeMap] with msg_id as keys