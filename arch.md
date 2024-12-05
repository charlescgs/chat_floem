# Main structures:
1. Room:
    ```rust
    pub struct RoomViewData {
        pub view_id: ViewId,
        pub room_id: Id,
        pub owner: Account,
        pub members: HashMap<Ulid, Account>,
        pub last_msg: RwSignal<Option<MsgCtx>>,
        pub msgs: RwSignal<RoomMsgChunks>,
        pub unread: RwSignal<bool>,
        pub num_unread: RwSignal<u16>,
        pub description: RwSignal<Option<String>>,
        pub common_data: Rc<CommonData>
    }
    ```
2. Msg:
    ```rust
    pub struct MsgViewData {
        pub view_id: ViewId,
        pub msg_id: Id,
        pub room_id: Id,
        pub author: Rc<Account>,
        pub room_owner: bool,
        pub msg: Rc<Msg>,
        pub com: RwSignal<Option<Vector<ComCtx>>>,
        pub rea: RwSignal<Option<Vector<ReaCtx>>>,
        pub common_data: Rc<CommonData>
    }
    ```
3. Editor:
    ```rust
    pub struct EditorView {
        pub view_id: ViewId,
        pub open_documents: RwSignal<HashMap<usize, Rc<dyn Document>>>,
        pub common_data: Rc<CommonData>
    }
    ```
4. Session:
    ```rust
    pub struct Session {
        pub user: Rc<Account>,
        pub accounts: RwSignal<HashMap<Ulid, Account>>,
        pub rooms: RwSignal<HashMap<usize, RwSignal<RoomViewData>>>,
        pub rooms_tabs: RwSignal<HashMap<Ulid, usize>>,
        pub rooms_tabs_count: Memo<usize>,
        pub active_room: RwSignal<Option<RoomTabIdx>>,
        // pub active_tab: RwSignal<usize>,
        pub common_data: Rc<CommonData>
    }
    ```

## Main views:
1. Side room
2. Msg
3. Editor

## Main containers:
1. Rooms list
2. Msgs list
3. Editor

------------
# Main concepts:
- UI runs on it's own thread and communicates with backend via channels/signals from channels
- UI state/session is a global static from the thread local
- Implement main command system

----------
## Updates flow
Update comes from the server to backend -> to frontend([UISession]) -> notif -> views fetch the data