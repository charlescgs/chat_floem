use std::fmt::Display;
use std::str::FromStr;

use serde::Deserialize;
use serde::Serialize;
use ulid::Ulid;


#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Id {
    pub tb: Tb,
    pub id: Ulid
}

impl Id {
    pub fn new(tb: Tb) -> Self {
        Self { tb, id: ulid::Ulid::new() }
    }
}

impl FromStr for Id {
    type Err = ();

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
		let (table, ulid) = s.split_once(':').ok_or_else(|| ())?;
		let tb = table.parse()?;
		let id = Ulid::from_string(ulid).unwrap();
		Ok(Self { tb, id })
    }
}

impl Display for Id {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}:{}", self.tb, self.id)
	}
}


#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Tb {
    Acc,
    Room,
    Msg,
    Com,
    Rea
}

impl Display for Tb {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", match self {
			Tb::Acc => "account",
			Tb::Room => "room",
			Tb::Msg => "msg",
			Tb::Rea => "reaction",
			Tb::Com => "msg_comment",
			// Tb::Inv => "room_invite",
			// Tb::SentTo => "sent_to",
			// Tb::DelTo => "delivered_by",
			// Tb::ViewBy => "viewed_by",
			// Tb::Created => "created",
			// Tb::Member => "member",
			// Tb::Picture => "picture",
			// Tb::Audio => "audio",
			// Tb::File => "file",
		})
	}
}

impl FromStr for Tb {
	type Err = ();

	fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
		match s {
			"account" => Ok(Self::Acc),
			"room" => Ok(Self::Room),
			"msg" => Ok(Self::Msg),
			"reaction" => Ok(Self::Rea),
			"msg_comment" => Ok(Self::Com),
			// "room_invite" => Ok(Self::Inv),
			// "sent_to" => Ok(Self::SentTo),
			// "delivered_by" => Ok(Self::DelTo),
			// "viewed_by" => Ok(Self::ViewBy),
			// "created" => Ok(Self::Created),
			// "member" => Ok(Self::Member),
			// "picture" => Ok(Self::Picture),
			// "audio" => Ok(Self::Audio),
			// "file" => Ok(Self::File),
			_ => Err(())
		}
	}
}