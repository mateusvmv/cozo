use crate::db::engine::Session;
use crate::error::CozoError::LogicError;
use crate::error::{CozoError, Result};
use crate::relation::data::DataKind;
use crate::relation::typing::Typing;
use std::collections::HashSet;
use std::fmt::{Debug, Formatter};

#[derive(Eq, PartialEq, Clone, Copy, Ord, PartialOrd, Hash)]
pub struct TableId {
    pub in_root: bool,
    pub id: i64,
}

impl Debug for TableId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}{}", if self.in_root { 'G' } else { 'L' }, self.id)
    }
}

impl TableId {
    pub fn new(in_root: bool, id: i64) -> Self {
        TableId { in_root, id }
    }
    pub fn is_valid(&self) -> bool {
        self.id >= 0
    }
}

impl From<(bool, usize)> for TableId {
    fn from((in_root, id): (bool, usize)) -> Self {
        Self {
            in_root,
            id: id as i64,
        }
    }
}

impl From<(bool, i64)> for TableId {
    fn from((in_root, id): (bool, i64)) -> Self {
        Self { in_root, id }
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Ord, PartialOrd)]
pub struct ColId {
    pub is_key: bool,
    pub id: i64,
}

impl Debug for ColId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, ".{}{}", if self.is_key { 'K' } else { 'D' }, self.id)
    }
}

impl From<(bool, i64)> for ColId {
    fn from((is_key, id): (bool, i64)) -> Self {
        Self { is_key, id }
    }
}

impl From<(bool, usize)> for ColId {
    fn from((is_key, id): (bool, usize)) -> Self {
        Self {
            is_key,
            id: id as i64,
        }
    }
}

impl Default for TableId {
    fn default() -> Self {
        TableId {
            in_root: false,
            id: -1,
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct TableInfo {
    pub kind: DataKind,
    pub table_id: TableId,
    pub src_table_id: TableId,
    pub dst_table_id: TableId,
    pub data_keys: HashSet<String>,
    pub key_typing: Vec<(String, Typing)>,
    pub val_typing: Vec<(String, Typing)>,
    pub src_key_typing: Vec<(String, Typing)>,
    pub dst_key_typing: Vec<(String, Typing)>,
    pub associates: Vec<TableInfo>,
}

impl<'a> Session<'a> {
    pub fn get_table_info(&self, tbl_name: &str) -> Result<TableInfo> {
        let table_info = match self.resolve(tbl_name)? {
            None => return Err(CozoError::UndefinedType(tbl_name.to_string())),
            Some(tpl) => {
                let mut main_coercer = match tpl.data_kind()? {
                    DataKind::Node => {
                        let key_extractor = Typing::try_from(
                            tpl.get_text(2)
                                .ok_or_else(|| {
                                    CozoError::BadDataFormat(tpl.data.as_ref().to_vec())
                                })?
                                .as_ref(),
                        )?
                        .extract_named_tuple()
                        .ok_or_else(|| CozoError::LogicError("Corrupt data".to_string()))?;
                        let val_extractor = Typing::try_from(
                            tpl.get_text(3)
                                .ok_or_else(|| {
                                    CozoError::BadDataFormat(tpl.data.as_ref().to_vec())
                                })?
                                .as_ref(),
                        )?
                        .extract_named_tuple()
                        .ok_or_else(|| CozoError::LogicError("Corrupt data".to_string()))?;
                        let in_root = tpl.get_bool(0).ok_or_else(|| {
                            CozoError::LogicError("Cannot extract in root".to_string())
                        })?;
                        let table_id = tpl.get_int(1).ok_or_else(|| {
                            CozoError::LogicError("Cannot extract in root".to_string())
                        })?;
                        let table_id = TableId::new(in_root, table_id);

                        TableInfo {
                            kind: DataKind::Node,
                            table_id,
                            src_table_id: Default::default(),
                            dst_table_id: Default::default(),
                            data_keys: val_extractor.iter().map(|(k, _)| k.clone()).collect(),
                            key_typing: key_extractor,
                            val_typing: val_extractor,
                            src_key_typing: vec![],
                            dst_key_typing: vec![],
                            associates: vec![],
                        }
                    }
                    DataKind::Edge => {
                        let other_key_extractor = Typing::try_from(
                            tpl.get_text(6)
                                .ok_or_else(|| {
                                    CozoError::LogicError("Key extraction failed".to_string())
                                })?
                                .as_ref(),
                        )?
                        .extract_named_tuple()
                        .ok_or_else(|| CozoError::LogicError("Corrupt data".to_string()))?;
                        let val_extractor = Typing::try_from(
                            tpl.get_text(7)
                                .ok_or_else(|| {
                                    CozoError::LogicError("Val extraction failed".to_string())
                                })?
                                .as_ref(),
                        )?
                        .extract_named_tuple()
                        .ok_or_else(|| CozoError::LogicError("Corrupt data".to_string()))?;
                        let src_in_root = tpl.get_bool(2).ok_or_else(|| {
                            CozoError::LogicError("Src in root extraction failed".to_string())
                        })?;
                        let src_id = tpl.get_int(3).ok_or_else(|| {
                            CozoError::LogicError("Src id extraction failed".to_string())
                        })?;
                        let src_table_id = TableId::new(src_in_root, src_id);
                        let dst_in_root = tpl.get_bool(4).ok_or_else(|| {
                            CozoError::LogicError("Dst in root extraction failed".to_string())
                        })?;
                        let dst_id = tpl.get_int(5).ok_or_else(|| {
                            CozoError::LogicError("Dst id extraction failed".to_string())
                        })?;
                        let dst_table_id = TableId::new(dst_in_root, dst_id);
                        let src = self.table_data(src_id, src_in_root)?.ok_or_else(|| {
                            CozoError::LogicError("Getting src failed".to_string())
                        })?;
                        let src_key = Typing::try_from(
                            src.get_text(2)
                                .ok_or_else(|| {
                                    CozoError::BadDataFormat(tpl.data.as_ref().to_vec())
                                })?
                                .as_ref(),
                        )?
                        .extract_named_tuple()
                        .ok_or_else(|| CozoError::LogicError("Corrupt data".to_string()))?;
                        let src_key_typing = src_key.into_iter().collect();

                        let dst = self.table_data(dst_id, dst_in_root)?.ok_or_else(|| {
                            CozoError::LogicError("Getting dst failed".to_string())
                        })?;
                        let dst_key = Typing::try_from(
                            dst.get_text(2)
                                .ok_or_else(|| {
                                    CozoError::BadDataFormat(tpl.data.as_ref().to_vec())
                                })?
                                .as_ref(),
                        )?
                        .extract_named_tuple()
                        .ok_or_else(|| CozoError::LogicError("Corrupt data".to_string()))?;
                        let dst_key_typing = dst_key.into_iter().collect();

                        let in_root = tpl.get_bool(0).ok_or_else(|| {
                            CozoError::LogicError("Cannot extract in root".to_string())
                        })?;
                        let table_id = tpl.get_int(1).ok_or_else(|| {
                            CozoError::LogicError("Cannot extract in root".to_string())
                        })?;
                        let table_id = TableId::new(in_root, table_id);

                        TableInfo {
                            kind: DataKind::Edge,
                            table_id,
                            src_table_id,
                            dst_table_id,
                            data_keys: val_extractor.iter().map(|(k, _)| k.clone()).collect(),
                            key_typing: other_key_extractor,
                            val_typing: val_extractor,
                            src_key_typing,
                            dst_key_typing,
                            associates: vec![],
                        }
                    }
                    _ => return Err(LogicError("Cannot insert into non-tables".to_string())),
                };
                let related = self.resolve_related_tables(tbl_name)?;
                for (_n, d) in related {
                    let t = d.get_text(4).ok_or_else(|| {
                        CozoError::LogicError("Unable to extract typing from assoc".to_string())
                    })?;
                    let t = Typing::try_from(t.as_ref())?
                        .extract_named_tuple()
                        .ok_or_else(|| CozoError::LogicError("Corrupt data".to_string()))?;
                    let in_root = d.get_bool(0).ok_or_else(|| {
                        CozoError::LogicError("Cannot extract in root".to_string())
                    })?;
                    let table_id = d.get_int(1).ok_or_else(|| {
                        CozoError::LogicError("Cannot extract in root".to_string())
                    })?;
                    let table_id = TableId::new(in_root, table_id);

                    let coercer = TableInfo {
                        kind: DataKind::Assoc,
                        table_id,
                        src_table_id: Default::default(),
                        dst_table_id: Default::default(),
                        data_keys: t.iter().map(|(k, _)| k.clone()).collect(),
                        key_typing: vec![],
                        val_typing: t,
                        src_key_typing: vec![],
                        dst_key_typing: vec![],
                        associates: vec![],
                    };

                    main_coercer.associates.push(coercer);
                }
                main_coercer
            }
        };
        Ok(table_info)
    }
}
