use std::collections::HashMap;
use std::rc::Rc;

use floem::{prelude::*, ViewId};
use editor::text::Document;
use ulid::Ulid;

use crate::common::CommonData;



/// Contains all documents and state for the rooms.
pub struct EditorViewData {
    pub view_id: ViewId,
    pub open_documents: RwSignal<HashMap<Ulid, Rc<dyn Document>>>,
    // TODO: more is needed..
    pub common_data: Rc<CommonData>
}