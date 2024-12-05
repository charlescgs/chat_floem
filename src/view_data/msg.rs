use std::rc::Rc;

use floem::prelude::*;
use floem::ViewId;
use im::Vector;

use crate::views::msg::{ComCtx, ReaCtx};
use crate::common::CommonData;
use crate::cont::acc::Account;
use crate::cont::msg::Msg;
use crate::util::Id;



/// Contains data needed to display msg widget on the msgs list.
pub struct MsgViewData {
    pub view_id: ViewId,
    pub msg_id: Id,
    pub room_id: Id,
    pub author: Rc<Account>,
    pub room_owner: bool,
    pub msg: Rc<Msg>,
    pub com: RwSignal<Vector<ComCtx>>,
    pub rea: RwSignal<Vector<ReaCtx>>,
    pub common_data: Rc<CommonData>
}