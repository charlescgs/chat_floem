use std::fmt::Display;

use chrono_lite::Datetime;
use serde::{Deserialize, Deserializer, Serialize};
use serde::de::Error;

use crate::util::{Id, Tb};





/// Holds all data connected to the single message. 
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Msg {
    #[serde(rename = "id")]
    pub msg_id: Id,
    pub room_id: Id,
    pub author: Id,
    pub created: Datetime,
    #[serde(default)]
    pub sent: Option<Datetime>,
    /// Main text of the message with the edits.
    pub text: Text,
    /// Optional media files.
    #[serde(default)]
    pub media: Option<MediaType>,
    /// How many - if any - edits was done on the message.
    #[serde(default)]
    pub edited: Option<u8>,
    /// Message comments - if any.
    #[serde(default)]
    pub comments: Option<Vec<MsgComment>>,
    /// Message reaction emojis - if any.
    #[serde(default)]
    pub reactions: Option<Vec<Reaction>>,
    /// If Msg was delivered to all room members.
    pub delivered_to_all: bool,
    /// If Msg was viewed by all room members.
    pub viewed_by_all: bool,
}


#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgComment {
    #[serde(rename = "id")]
    pub com_id: Id,
    pub author: Id,
    pub parent_id: Id,
    pub room_id: Id,
    pub text: String,
    pub created: Datetime,
    #[serde(default)]
    pub reactions: Option<Vec<Reaction>>,
    #[serde(default)]
    pub updated: Option<Datetime>,
    /// If Comment was delivered to all room members.
    pub delivered_to_all: bool,
    /// If Comment was viewed by all room members.
    pub viewed_by_all: bool
}


#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reaction {
    #[serde(rename = "id")]
    pub rea_id: Id,
    pub author: Id,
    /// Msg.
    #[serde(default)]
    pub grandparent_id: Option<Id>,
    /// Msg/Comment.
    pub parent_id: Id,
    pub room_id: Id,
    #[serde(deserialize_with = "de_emoji")]
    pub emoji: char,
    pub created: Datetime,
    /// If Reaction was delivered to all room members.
    pub delivered_to_all: bool,
    /// If Reaction was viewed by all room members.
    pub viewed_by_all: bool
}


/// Custom deserializer for the emoji encoded in database as string.
fn de_emoji<'de, D: Deserializer<'de>>(deserializer: D) -> Result<char, D::Error> {
    let emoji = String::deserialize(deserializer)?;
    let Some(e) = emoji.chars().next() else {
        return Err(D::Error::custom("Expected single char"))
    };
    Ok(e)
}



/// All media files that can be attached to the message.
/// TODO: change it to struct with enum on media type.
/// TODO: make edit history more that a string?
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaType {
    Picture {
        media_id: Id,
        room_id: Id,
        msg_id: Id,
        stored_on: String,
        data: Vec<u8>,
        data_size: u64,
        name: String,
        media_ext: MediaExt,
        edit_history: Option<Vec<MsgEdit>>,
        last_edited: Option<Datetime>,
        is_upt: Option<Id>
    },
    Audio {
        media_id: Id,
        room_id: Id,
        msg_id: Id,
        stored_on: String,
        data: Vec<u8>,
        data_size: u64,
        name: String,
        media_ext: MediaExt,
        edit_history: Option<Vec<MsgEdit>>,
        last_edited: Option<Datetime>,
        is_upt: Option<Id>
    },
    File {
        media_id: Id,
        room_id: Id,
        msg_id: Id,
        stored_on: String,
        data: Vec<u8>,
        data_size: u64,
        name: String,
        media_ext: MediaExt,
        edit_history: Option<Vec<MsgEdit>>,
        last_edited: Option<Datetime>,
        is_upt: Option<Id>
    },
    // Video {
    //     data: Vec<u8>,
    //     description: Option<String>,
    //     edit_history: Option<Vec<MsgEdit>>,
    //     last_edited: Option<Datetime>
    // },
    // Markdown {
    //     data: Vec<u8>,
    //     description: Option<String>,
    //     edit_history: Option<Vec<MsgEdit>>,
    //     last_edited: Option<Datetime>
    // },
    // CodeBlock { -- code-block
    //     data: Vec<u8>,
    //     description: Option<String>,
    //     edit_history: Option<Vec<MsgEdit>>,
    //     last_edited: Option<Datetime>
    // },
}

impl MediaType {
    pub fn get_id(&self) -> &Id {
        match self {
            MediaType::Picture { media_id, .. } => media_id,
            MediaType::Audio { media_id, .. } => media_id,
            MediaType::File { media_id, .. } => media_id
        }
    }
    
    pub fn get_room_id(&self) -> &Id {
        match self {
            MediaType::Picture { room_id, .. } => room_id,
            MediaType::Audio { room_id, .. } => room_id,
            MediaType::File { room_id, .. } => room_id
        }
    }
    
    pub fn get_msg_id(&self) -> &Id {
        match self {
            MediaType::Picture { msg_id, .. } => msg_id,
            MediaType::Audio { msg_id, .. } => msg_id,
            MediaType::File { msg_id, .. } => msg_id
        }
    }

    pub fn get_path(&self) -> &str {
        match self {
            MediaType::Picture { stored_on, .. } => stored_on,
            MediaType::Audio { stored_on, .. } => stored_on,
            MediaType::File { stored_on, .. } => stored_on
        }
    }

    pub fn update_path(&mut self, new: String) {
        match self {
            MediaType::Picture { stored_on, .. } => *stored_on = new,
            MediaType::Audio { stored_on, .. } => *stored_on = new,
            MediaType::File { stored_on, .. } => *stored_on = new
        }
    }

    pub fn get_size(&self) -> &u64 {
        match self {
            MediaType::Picture { data_size, .. } => data_size,
            MediaType::Audio { data_size, .. } => data_size,
            MediaType::File { data_size, .. } => data_size
        }
    }

    pub fn get_name(&self) -> &str {
        match self {
            MediaType::Picture { name, .. } => name,
            MediaType::Audio { name, .. } => name,
            MediaType::File { name, .. } => name
        }
    }
    
    pub fn get_ext(&self) -> &MediaExt {
        match self {
            MediaType::Picture { media_ext, .. } => media_ext,
            MediaType::Audio { media_ext, .. } => media_ext,
            MediaType::File { media_ext, .. } => media_ext
        }
    }

    pub fn get_edit_history(&self) -> &Option<Vec<MsgEdit>> {
        match self {
            MediaType::Picture { edit_history, .. } => edit_history,
            MediaType::Audio { edit_history, .. } => edit_history,
            MediaType::File { edit_history, .. } => edit_history
        }
    }
   
    pub fn update_edit_history(&mut self, new_edit: MsgEdit) {
        match self {
            MediaType::Picture { edit_history, .. } => {
                match edit_history {
                    Some(edits) => edits.push(new_edit),
                    None => *edit_history = Some(Vec::from([new_edit])),
                }
            },
            MediaType::Audio { edit_history, .. } => {
                match edit_history {
                    Some(edits) => edits.push(new_edit),
                    None => *edit_history = Some(Vec::from([new_edit])),
                }
            },
            MediaType::File { edit_history, .. } => {
                match edit_history {
                    Some(edits) => edits.push(new_edit),
                    None => *edit_history = Some(Vec::from([new_edit])),
                }
            }
        }
    }

    pub fn is_update(&self) -> Option<Id> {
        match self {
            MediaType::Picture { is_upt, .. } => is_upt.clone(),
            MediaType::Audio { is_upt, .. } => is_upt.clone(),
            MediaType::File { is_upt, .. } => is_upt.clone(),
        }
    }

    pub fn get_last_edited(&self) -> &Option<Datetime> {
        match self {
            MediaType::Picture { last_edited, .. } => last_edited,
            MediaType::Audio { last_edited, .. } => last_edited,
            MediaType::File { last_edited, .. } => last_edited
        }
    }
    
    pub fn update_last_edited(&mut self, stamp: Datetime) {
        match self {
            MediaType::Picture { last_edited, .. } => *last_edited = Some(stamp),
            MediaType::Audio { last_edited, .. } => *last_edited = Some(stamp),
            MediaType::File { last_edited, .. } => *last_edited = Some(stamp)
        }
    }

    pub fn get_data(&self) -> &[u8] {
        match self {
            MediaType::Picture { data, ..} => data,
            MediaType::Audio { data, ..} => data,
            MediaType::File { data, ..} => data
        }
    }

    pub fn update_data(&mut self, new_data: Vec<u8>) {
        match self {
            MediaType::Picture { data, ..} => *data = new_data,
            MediaType::Audio { data, ..} => *data = new_data,
            MediaType::File { data, ..} => *data = new_data
        }
    }
}


/// Message Text type to hold current content - as well as - edits history and count.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Text {
    /// Most recent text content.
    pub current: String,
    /// History of the edits.
    pub edits: Option<Vec<MsgEdit>>,
    /// Timestamp of the last text edit.
    pub last_edited: Option<Datetime>
}


/// Allowed media types.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaExt {
    Txt,
    Jpg,
    Png,
    Mp3,
    Flac,
    Wav
}

impl Display for MediaExt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaExt::Txt => f.write_str("txt"),
            MediaExt::Jpg => f.write_str("jpg"),
            MediaExt::Png => f.write_str("png"),
            MediaExt::Mp3 => f.write_str("mp3"),
            MediaExt::Flac => f.write_str("flac"),
            MediaExt::Wav => f.write_str("wav"),
        }
    }
}


/// Holds data of the single edit on the message  text or it's media file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsgEdit {
    /// Edit stamp.
    pub stamp: Datetime,
    /// Previous text of path to previous media attachment.
    pub content: String
}

impl MsgEdit {
    /// Construct an instance of the [MsgEdit] from the building blocks.
    pub fn new(content: &str, stamp: &Datetime) -> Self {
        Self {
            stamp: stamp.clone(),
            content: content.into()
        }
    }
}


/// Struct for read request of the specific message.
pub struct QueryMsg(pub Id);



impl Msg {
    /// Construct new Msg.
    pub fn new(
        msg_id: Id,
        room_id: Id,
        author: Id,
        created: Datetime,
        text: String,
    ) -> Self
    {
        Self {
            msg_id,
            room_id,
            author,
            created,
            media: None,
            comments: None,
            reactions: None,
            delivered_to_all: false,
            viewed_by_all: false,
            text: Text { current: text, edits: None, last_edited: None },
            edited: None,
            sent: None
        }
    }

    /// Construct new test message with random text and specified author.
    pub fn test_msg(room_id: Id, author: Id) -> Self {
        let text = format!("Test Message by {}.", &author);
        Self {
            msg_id: Id::new(Tb::Msg),
            room_id,
            media: None,
            author,
            created: Datetime::default(),
            comments: None,
            reactions: None,
            delivered_to_all: false,
            viewed_by_all: false,
            text: Text { current: text, edits: None, last_edited: None },
            edited: None,
            sent: None
        }
    }

    /// Update [Msg] text and save all version as [MsgEdit].
    pub fn update_text(&mut self, stamp: Datetime, new_text: &str) {
        let old = MsgEdit::new(&self.text.current, &stamp);
        self.text.current = new_text.into();
        self.text.last_edited = Some(stamp);
        if let Some(ref mut edits) = self.text.edits {
            edits.push(old);
        } else {
            self.text.edits = Some(Vec::from([old]))
        }
        if let Some(mut edited) = self.edited {
            edited += 1
        } else {
            self.edited = Some(1)
        }
    }
}